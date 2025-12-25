// Vtable detection utilities for Rust binaries
//
// Rust vtable layout:
//   offset 0x00: drop fn pointer (or null)
//   offset 0x08: size (usize)
//   offset 0x10: alignment (usize)
//   offset 0x18+: trait method pointers

use binaryninja::binary_view::{BinaryView, BinaryViewBase, BinaryViewExt};
use binaryninja::headless::Session;
use binaryninja::section::Semantics;
use std::env;

/// Rust vtable header size (drop + size + align)
pub const VTABLE_HEADER_SIZE: u64 = 24;

/// Information about a detected vtable
#[derive(Debug, Clone)]
pub struct VtableInfo {
    pub address: u64,
    pub size: u64,
    pub drop_fn: u64,
    pub type_size: u64,
    pub type_align: u64,
    pub method_count: usize,
    pub methods: Vec<u64>,
}

impl VtableInfo {
    /// Check if this vtable has a drop function
    pub fn has_drop(&self) -> bool {
        self.drop_fn != 0
    }

    /// Check if this is a zero-sized type (like closures for fmt::Display)
    pub fn is_zst(&self) -> bool {
        self.type_size == 0
    }
}



/// Check if an address could be a valid vtable header
fn is_valid_vtable_header(bv: &BinaryView, addr: u64) -> bool {
    let Some(drop_fn) = read_u64(bv, addr) else {
        return false;
    };
    let Some(size) = read_u64(bv, addr + 8) else {
        return false;
    };
    let Some(align) = read_u64(bv, addr + 16) else {
        return false;
    };

    // drop_fn must be null or valid code pointer
    if drop_fn != 0 && !is_code_ptr(bv, drop_fn) {
        return false;
    }

    // size must be reasonable
    if size > 0x100000 {
        return false;
    }

    // align must be power of 2 and reasonable
    if align == 0 || align > 4096 || (align & (align - 1)) != 0 {
        return false;
    }

    true
}

/// Determine vtable size by scanning for valid method pointers
///
/// Scans from the header until a non-code pointer is found or max_size is reached
pub fn get_vtable_size(bv: &BinaryView, addr: u64, max_size: u64) -> u64 {
    let mut size = VTABLE_HEADER_SIZE;
    let mut offset = VTABLE_HEADER_SIZE;

    while offset < max_size {
        let Some(ptr) = read_u64(bv, addr + offset) else {
            break;
        };

        if !is_code_ptr(bv, ptr) {
            break;
        }

        offset += 8;
        size = offset;
    }

    size
}

/// Parse a vtable at the given address
pub fn parse_vtable(bv: &BinaryView, addr: u64, max_size: u64) -> Option<VtableInfo> {
    if !is_valid_vtable_header(bv, addr) {
        return None;
    }

    let drop_fn = read_u64(bv, addr)?;
    let type_size = read_u64(bv, addr + 8)?;
    let type_align = read_u64(bv, addr + 16)?;

    // Collect method pointers
    let mut methods = Vec::new();
    let mut offset = VTABLE_HEADER_SIZE;

    while offset < max_size {
        let Some(ptr) = read_u64(bv, addr + offset) else {
            break;
        };

        if !is_code_ptr(bv, ptr) {
            break;
        }

        methods.push(ptr);
        offset += 8;
    }

    let size = VTABLE_HEADER_SIZE + (methods.len() as u64 * 8);

    Some(VtableInfo {
        address: addr,
        size,
        drop_fn,
        type_size,
        type_align,
        method_count: methods.len(),
        methods,
    })
}

/// Compute vtable sizes given a list of known vtable addresses
///
/// Uses neighbor addresses as upper bounds, then validates via pointer scanning
pub fn compute_vtable_sizes(bv: &BinaryView, addrs: &[u64]) -> Vec<VtableInfo> {
    if addrs.is_empty() {
        return Vec::new();
    }

    let mut sorted: Vec<u64> = addrs.to_vec();
    sorted.sort_unstable();

    let mut results = Vec::new();

    for (i, &addr) in sorted.iter().enumerate() {
        // Upper bound from next vtable or reasonable max
        let max_size = if i + 1 < sorted.len() {
            sorted[i + 1] - addr
        } else {
            0x200 // 512 bytes max for last vtable
        };

        if let Some(info) = parse_vtable(bv, addr, max_size) {
            results.push(info);
        }
    }

    results
}

/// Scan a memory range for potential vtables
///
/// Returns addresses that look like valid vtable headers
pub fn scan_for_vtables(bv: &BinaryView, start: u64, end: u64) -> Vec<u64> {
    let mut candidates = Vec::new();
    let mut addr = start;

    // Vtables are pointer-aligned
    let align = 8u64;
    addr = (addr + align - 1) & !(align - 1);

    while addr + VTABLE_HEADER_SIZE <= end {
        if is_valid_vtable_header(bv, addr) {
            // Additional check: must have at least one method pointer
            if let Some(first_method) = read_u64(bv, addr + VTABLE_HEADER_SIZE) {
                if is_code_ptr(bv, first_method) {
                    candidates.push(addr);
                    // Skip past this vtable's header to avoid false positives
                    addr += VTABLE_HEADER_SIZE;
                    continue;
                }
            }
        }
        addr += align;
    }

    candidates
}

/// Find vtables in .data.rel.ro section
pub fn find_vtables_in_relro(bv: &BinaryView) -> Vec<VtableInfo> {
    // Find .data.rel.ro section
    let sections = bv.sections();
    let section = sections.iter().find(|s| {
        let name = s.name();
        let name_str = name.to_string_lossy();
        name_str.contains("data.rel.ro") || name_str == ".data.rel.ro"
    });

    let Some(section) = section else {
        return Vec::new();
    };

    let start = section.start();
    let end = start + section.len() as u64;

    let candidates = scan_for_vtables(bv, start, end);
    compute_vtable_sizes(bv, &candidates)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: {} <binary>", args[0]);
        std::process::exit(1);
    }

    let binary_path = &args[1];

    let session = match Session::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to init binja: {:?}", e);
            std::process::exit(1);
        }
    };

    let bv = match session.load(binary_path) {
        Some(bv) => bv,
        None => {
            eprintln!("failed to load {}", binary_path);
            std::process::exit(1);
        }
    };

    println!("scanning {} for vtables...", binary_path);

    let vtables = find_vtables_in_relro(&bv);

    if vtables.is_empty() {
        println!("no vtables found in .data.rel.ro");
        return;
    }

    println!("found {} vtables:\n", vtables.len());

    for vt in &vtables {
        println!("vtable @ 0x{:x}", vt.address);
        println!("  size:    {} bytes ({} methods)", vt.size, vt.method_count);
        println!("  type:    size={}, align={}", vt.type_size, vt.type_align);
        if vt.has_drop() {
            println!("  drop:    0x{:x}", vt.drop_fn);
        } else {
            println!("  drop:    (none)");
        }
        for (i, &method) in vt.methods.iter().enumerate() {
            println!("  [{}]:     0x{:x}", i, method);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_of_two() {
        assert!((1u64 & (1 - 1)) == 0); // 1 is power of 2
        assert!((8u64 & (8 - 1)) == 0); // 8 is power of 2
        assert!((3u64 & (3 - 1)) != 0); // 3 is not power of 2
    }
}
