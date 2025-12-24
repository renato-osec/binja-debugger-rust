// rust binja dbg bindings

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use binaryninja::binary_view::BinaryView;
use std::ffi::{c_void, CStr, CString};
use std::fmt;
use std::sync::Arc;

pub mod ffi;

//common types
pub use ffi::{
    BNDebugAdapterConnectionStatus, BNDebugAdapterTargetStatus, BNDebugStopReason,
    BNDebuggerEventType, BNFunctionGraphType,
};

struct DebuggerControllerInner {
    handle: *mut ffi::BNDebuggerController,
}

impl Drop for DebuggerControllerInner {
    fn drop(&mut self) {
        unsafe {
            ffi::BNDebuggerFreeController(self.handle);
        }
    }
}

// SAFETY: thread-safe in Binary Ninja core
unsafe impl Send for DebuggerControllerInner {}
unsafe impl Sync for DebuggerControllerInner {}

#[derive(Clone)]
pub struct DebuggerController {
    inner: Arc<DebuggerControllerInner>,
}

impl DebuggerController {
    /// get or create a debugger controller for a binary view
    pub fn new(bv: &BinaryView) -> Option<Self> {
        let handle = unsafe { ffi::BNGetDebuggerController(bv.handle as *mut _) };
        if handle.is_null() {
            None
        } else {
            Some(Self {
                inner: Arc::new(DebuggerControllerInner { handle }),
            })
        }
    }

    /// Check if a debugger controller exists for a binary view
    pub fn exists(bv: &BinaryView) -> bool {
        unsafe { ffi::BNDebuggerControllerExists(bv.handle as *mut _) }
    }

    /// Get the underlying handle
    fn handle(&self) -> *mut ffi::BNDebuggerController {
        self.inner.handle
    }

    /// Get the live BinaryView used during debugging.
    ///
    /// This returns the rebased view with actual runtime addresses
    /// (e.g., 0x555555554000 instead of 0x0 for PIE binaries).
    ///
    /// This is equivalent to Python's `dbg.data`.
    ///
    /// # Safety
    /// This uses unsafe transmutation internally because the binaryninja
    /// crate's BinaryView constructor is private. The returned reference
    /// is properly reference-counted.
    pub fn data(&self) -> Option<binaryninja::rc::Ref<BinaryView>> {
        let handle = unsafe { ffi::BNDebuggerGetData(self.handle()) };
        if handle.is_null() {
            return None;
        }

        // Increment reference count
        let handle = unsafe { binaryninjacore_sys::BNNewViewReference(handle as *mut _) };
        if handle.is_null() {
            return None;
        }

        // Transmute to BinaryView - safe because BinaryView is just a wrapper around *mut BNBinaryView
        let bv: BinaryView = unsafe { std::mem::transmute(handle) };

        Some(bv.to_owned())
    }

    /// Get the raw BinaryView handle for advanced usage.
    ///
    /// The returned handle has its reference count incremented.
    /// Should be cleaned up with BNFreeBinaryView when done.
    pub fn data_handle(&self) -> *mut ffi::BNBinaryView {
        let handle = unsafe { ffi::BNDebuggerGetData(self.handle()) };
        if handle.is_null() {
            return std::ptr::null_mut();
        }
        // Increment ref count before returning
        unsafe { binaryninjacore_sys::BNNewViewReference(handle as *mut _) as *mut _ }
    }

    pub fn is_connected(&self) -> bool {
        unsafe { ffi::BNDebuggerIsConnected(self.handle()) }
    }

    pub fn is_running(&self) -> bool {
        unsafe { ffi::BNDebuggerIsRunning(self.handle()) }
    }

    pub fn connection_status(&self) -> BNDebugAdapterConnectionStatus {
        unsafe { ffi::BNDebuggerGetConnectionStatus(self.handle()) }
    }

    pub fn target_status(&self) -> BNDebugAdapterTargetStatus {
        unsafe { ffi::BNDebuggerGetTargetStatus(self.handle()) }
    }

    pub fn adapter_type(&self) -> String {
        unsafe {
            let ptr = ffi::BNDebuggerGetAdapterType(self.handle());
            if ptr.is_null() {
                return String::new();
            }
            let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            ffi::BNDebuggerFreeString(ptr);
            s
        }
    }

    pub fn set_adapter_type(&self, adapter: &str) {
        let adapter_cstr = CString::new(adapter).unwrap();
        unsafe { ffi::BNDebuggerSetAdapterType(self.handle(), adapter_cstr.as_ptr()) }
    }

    pub fn executable_path(&self) -> String {
        unsafe {
            let ptr = ffi::BNDebuggerGetExecutablePath(self.handle());
            if ptr.is_null() {
                return String::new();
            }
            let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            ffi::BNDebuggerFreeString(ptr);
            s
        }
    }

    pub fn set_executable_path(&self, path: &str) {
        let path_cstr = CString::new(path).unwrap();
        unsafe { ffi::BNDebuggerSetExecutablePath(self.handle(), path_cstr.as_ptr()) }
    }

    pub fn working_directory(&self) -> String {
        unsafe {
            let ptr = ffi::BNDebuggerGetWorkingDirectory(self.handle());
            if ptr.is_null() {
                return String::new();
            }
            let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            ffi::BNDebuggerFreeString(ptr);
            s
        }
    }

    pub fn set_working_directory(&self, path: &str) {
        let path_cstr = CString::new(path).unwrap();
        unsafe { ffi::BNDebuggerSetWorkingDirectory(self.handle(), path_cstr.as_ptr()) }
    }

    pub fn command_line_arguments(&self) -> String {
        unsafe {
            let ptr = ffi::BNDebuggerGetCommandLineArguments(self.handle());
            if ptr.is_null() {
                return String::new();
            }
            let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            ffi::BNDebuggerFreeString(ptr);
            s
        }
    }

    pub fn set_command_line_arguments(&self, args: &str) {
        let args_cstr = CString::new(args).unwrap();
        unsafe { ffi::BNDebuggerSetCommandLineArguments(self.handle(), args_cstr.as_ptr()) }
    }

    /// non blocking

    pub fn launch(&self) -> bool {
        unsafe { ffi::BNDebuggerLaunch(self.handle()) }
    }

    pub fn go(&self) -> bool {
        unsafe { ffi::BNDebuggerGo(self.handle()) }
    }

    pub fn pause(&self) {
        unsafe { ffi::BNDebuggerPause(self.handle()) }
    }

    pub fn quit(&self) {
        unsafe { ffi::BNDebuggerQuit(self.handle()) }
    }

    pub fn restart(&self) {
        unsafe { ffi::BNDebuggerRestart(self.handle()) }
    }

    pub fn detach(&self) {
        unsafe { ffi::BNDebuggerDetach(self.handle()) }
    }

    pub fn step_into(&self, il: BNFunctionGraphType) -> bool {
        unsafe { ffi::BNDebuggerStepInto(self.handle(), il) }
    }

    pub fn step_over(&self, il: BNFunctionGraphType) -> bool {
        unsafe { ffi::BNDebuggerStepOver(self.handle(), il) }
    }

    pub fn step_return(&self) -> bool {
        unsafe { ffi::BNDebuggerStepReturn(self.handle()) }
    }


    pub fn launch_and_wait(&self) -> BNDebugStopReason {
        unsafe { ffi::BNDebuggerLaunchAndWait(self.handle()) }
    }

    pub fn go_and_wait(&self) -> BNDebugStopReason {
        unsafe { ffi::BNDebuggerGoAndWait(self.handle()) }
    }

    pub fn pause_and_wait(&self) -> BNDebugStopReason {
        unsafe { ffi::BNDebuggerPauseAndWait(self.handle()) }
    }

    pub fn quit_and_wait(&self) {
        unsafe { ffi::BNDebuggerQuitAndWait(self.handle()) }
    }

    pub fn step_into_and_wait(&self, il: BNFunctionGraphType) -> BNDebugStopReason {
        unsafe { ffi::BNDebuggerStepIntoAndWait(self.handle(), il) }
    }

    pub fn step_over_and_wait(&self, il: BNFunctionGraphType) -> BNDebugStopReason {
        unsafe { ffi::BNDebuggerStepOverAndWait(self.handle(), il) }
    }

    pub fn step_return_and_wait(&self) -> BNDebugStopReason {
        unsafe { ffi::BNDebuggerStepReturnAndWait(self.handle()) }
    }

    pub fn run_to(&self, addresses: &[u64]) -> bool {
        unsafe { ffi::BNDebuggerRunTo(self.handle(), addresses.as_ptr(), addresses.len()) }
    }

    pub fn run_to_and_wait(&self, addresses: &[u64]) -> BNDebugStopReason {
        unsafe { ffi::BNDebuggerRunToAndWait(self.handle(), addresses.as_ptr(), addresses.len()) }
    }

    /// regs

    pub fn ip(&self) -> u64 {
        unsafe { ffi::BNDebuggerGetIP(self.handle()) }
    }

    pub fn last_ip(&self) -> u64 {
        unsafe { ffi::BNDebuggerGetLastIP(self.handle()) }
    }

    pub fn set_ip(&self, address: u64) -> bool {
        unsafe { ffi::BNDebuggerSetIP(self.handle(), address) }
    }

    pub fn stack_pointer(&self) -> u64 {
        unsafe { ffi::BNDebuggerGetStackPointer(self.handle()) }
    }

    pub fn registers(&self) -> Vec<DebugRegister> {
        let mut count = 0usize;
        let ptr = unsafe { ffi::BNDebuggerGetRegisters(self.handle(), &mut count) };
        if ptr.is_null() {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(count);
        unsafe {
            let slice = std::slice::from_raw_parts(ptr, count);
            for reg in slice {
                result.push(DebugRegister::from_raw(reg));
            }
            ffi::BNDebuggerFreeRegisters(ptr, count);
        }
        result
    }

    pub fn get_register_value(&self, name: &str) -> Vec<u8> {
        let name_cstr = CString::new(name).unwrap();
        let mut buffer = [0u8; 64];
        unsafe {
            ffi::BNDebuggerGetRegisterValue(self.handle(), name_cstr.as_ptr(), buffer.as_mut_ptr());
        }
        buffer.to_vec()
    }

    pub fn set_register_value(&self, name: &str, value: &[u8]) -> bool {
        let name_cstr = CString::new(name).unwrap();
        unsafe { ffi::BNDebuggerSetRegisterValue(self.handle(), name_cstr.as_ptr(), value.as_ptr()) }
    }

    /// mem

    pub fn read_memory(&self, address: u64, size: usize) -> Option<Vec<u8>> {
        let ptr = unsafe { ffi::BNDebuggerReadMemory(self.handle(), address, size) };
        if ptr.is_null() {
            return None;
        }

        // Use binaryninjacore-sys functions to read from DataBuffer
        let data = unsafe {
            let len = binaryninjacore_sys::BNGetDataBufferLength(ptr as *mut _);
            let data_ptr = binaryninjacore_sys::BNGetDataBufferContents(ptr as *mut _);
            let slice = std::slice::from_raw_parts(data_ptr as *const u8, len);
            let result = slice.to_vec();
            binaryninjacore_sys::BNFreeDataBuffer(ptr as *mut _);
            result
        };

        Some(data)
    }

    pub fn write_memory(&self, address: u64, data: &[u8]) -> bool {
        unsafe {
            let buffer = binaryninjacore_sys::BNCreateDataBuffer(data.as_ptr() as *const _, data.len());
            let result = ffi::BNDebuggerWriteMemory(self.handle(), address, buffer as *mut _);
            binaryninjacore_sys::BNFreeDataBuffer(buffer);
            result
        }
    }

    /// threads

    pub fn threads(&self) -> Vec<DebugThread> {
        let mut count = 0usize;
        let ptr = unsafe { ffi::BNDebuggerGetThreads(self.handle(), &mut count) };
        if ptr.is_null() {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(count);
        unsafe {
            let slice = std::slice::from_raw_parts(ptr, count);
            for thread in slice {
                result.push(DebugThread::from_raw(thread));
            }
            ffi::BNDebuggerFreeThreads(ptr, count);
        }
        result
    }

    pub fn active_thread(&self) -> DebugThread {
        unsafe { DebugThread::from_raw(&ffi::BNDebuggerGetActiveThread(self.handle())) }
    }

    pub fn set_active_thread(&self, thread: &DebugThread) {
        unsafe { ffi::BNDebuggerSetActiveThread(self.handle(), thread.to_raw()) }
    }

    pub fn suspend_thread(&self, tid: u32) -> bool {
        unsafe { ffi::BNDebuggerSuspendThread(self.handle(), tid) }
    }

    pub fn resume_thread(&self, tid: u32) -> bool {
        unsafe { ffi::BNDebuggerResumeThread(self.handle(), tid) }
    }

    /// stacks

    pub fn frames_of_thread(&self, tid: u32) -> Vec<DebugFrame> {
        let mut count = 0usize;
        let ptr = unsafe { ffi::BNDebuggerGetFramesOfThread(self.handle(), tid, &mut count) };
        if ptr.is_null() {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(count);
        unsafe {
            let slice = std::slice::from_raw_parts(ptr, count);
            for frame in slice {
                result.push(DebugFrame::from_raw(frame));
            }
            ffi::BNDebuggerFreeFrames(ptr, count);
        }
        result
    }

    /// mods

    pub fn modules(&self) -> Vec<DebugModule> {
        let mut count = 0usize;
        let ptr = unsafe { ffi::BNDebuggerGetModules(self.handle(), &mut count) };
        if ptr.is_null() {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(count);
        unsafe {
            let slice = std::slice::from_raw_parts(ptr, count);
            for module in slice {
                result.push(DebugModule::from_raw(module));
            }
            ffi::BNDebuggerFreeModules(ptr, count);
        }
        result
    }

    /// bps

    pub fn breakpoints(&self) -> Vec<DebugBreakpoint> {
        let mut count = 0usize;
        let ptr = unsafe { ffi::BNDebuggerGetBreakpoints(self.handle(), &mut count) };
        if ptr.is_null() {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(count);
        unsafe {
            let slice = std::slice::from_raw_parts(ptr, count);
            for bp in slice {
                result.push(DebugBreakpoint::from_raw(bp));
            }
            ffi::BNDebuggerFreeBreakpoints(ptr, count);
        }
        result
    }

    pub fn add_breakpoint(&self, address: u64) {
        unsafe { ffi::BNDebuggerAddAbsoluteBreakpoint(self.handle(), address) }
    }

    pub fn delete_breakpoint(&self, address: u64) {
        unsafe { ffi::BNDebuggerDeleteAbsoluteBreakpoint(self.handle(), address) }
    }

    pub fn enable_breakpoint(&self, address: u64) {
        unsafe { ffi::BNDebuggerEnableAbsoluteBreakpoint(self.handle(), address) }
    }

    pub fn disable_breakpoint(&self, address: u64) {
        unsafe { ffi::BNDebuggerDisableAbsoluteBreakpoint(self.handle(), address) }
    }

    pub fn contains_breakpoint(&self, address: u64) -> bool {
        unsafe { ffi::BNDebuggerContainsAbsoluteBreakpoint(self.handle(), address) }
    }

    pub fn add_relative_breakpoint(&self, module: &str, offset: u64) {
        let module_cstr = CString::new(module).unwrap();
        unsafe { ffi::BNDebuggerAddRelativeBreakpoint(self.handle(), module_cstr.as_ptr(), offset) }
    }

    pub fn delete_relative_breakpoint(&self, module: &str, offset: u64) {
        let module_cstr = CString::new(module).unwrap();
        unsafe {
            ffi::BNDebuggerDeleteRelativeBreakpoint(self.handle(), module_cstr.as_ptr(), offset)
        }
    }

    /// stopping

    pub fn stop_reason(&self) -> BNDebugStopReason {
        unsafe { ffi::BNDebuggerGetStopReason(self.handle()) }
    }

    pub fn exit_code(&self) -> u32 {
        unsafe { ffi::BNDebuggerGetExitCode(self.handle()) }
    }

    /// events

    //this should be reviewed
    pub fn register_event_callback<F>(&self, name: &str, callback: F) -> usize
    where
        F: Fn(&DebuggerEvent) + Send + Sync + 'static,
    {
        let name_cstr = CString::new(name).unwrap();
        let boxed: Box<Box<dyn Fn(&DebuggerEvent) + Send + Sync>> = Box::new(Box::new(callback));
        let ctx = Box::into_raw(boxed) as *mut c_void;

        unsafe extern "C" fn trampoline(ctx: *mut c_void, event: *mut ffi::BNDebuggerEvent) {
            if ctx.is_null() || event.is_null() {
                return;
            }
            let callback = &*(ctx as *const Box<dyn Fn(&DebuggerEvent) + Send + Sync>);
            let event_wrapped = DebuggerEvent::from_raw(&*event);
            callback(&event_wrapped);
        }

        unsafe {
            ffi::BNDebuggerRegisterEventCallback(
                self.handle(),
                Some(trampoline),
                name_cstr.as_ptr(),
                ctx,
            )
        }
    }

    pub fn remove_event_callback(&self, index: usize) {
        unsafe { ffi::BNDebuggerRemoveEventCallback(self.handle(), index) }
    }

    //invoke backend (useful for watchpoints)

    pub fn invoke_backend_command(&self, cmd: &str) -> String {
        let cmd_cstr = CString::new(cmd).unwrap();
        unsafe {
            let ptr = ffi::BNDebuggerInvokeBackendCommand(self.handle(), cmd_cstr.as_ptr());
            if ptr.is_null() {
                return String::new();
            }
            let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            ffi::BNDebuggerFreeString(ptr);
            s
        }
    }
}

//data types

#[derive(Debug, Clone)]
pub struct DebugThread {
    pub tid: u32,
    pub rip: u64,
    pub is_frozen: bool,
}

impl DebugThread {
    fn from_raw(raw: &ffi::BNDebugThread) -> Self {
        Self {
            tid: raw.m_tid,
            rip: raw.m_rip,
            is_frozen: raw.m_isFrozen,
        }
    }

    fn to_raw(&self) -> ffi::BNDebugThread {
        ffi::BNDebugThread {
            m_tid: self.tid,
            m_rip: self.rip,
            m_isFrozen: self.is_frozen,
        }
    }
}

impl fmt::Display for DebugThread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Thread {} @ 0x{:x}{}",
            self.tid,
            self.rip,
            if self.is_frozen { " (frozen)" } else { "" }
        )
    }
}

#[derive(Debug, Clone)]
pub struct DebugFrame {
    pub index: usize,
    pub pc: u64,
    pub sp: u64,
    pub fp: u64,
    pub function_name: String,
    pub function_start: u64,
    pub module: String,
}

impl DebugFrame {
    fn from_raw(raw: &ffi::BNDebugFrame) -> Self {
        Self {
            index: raw.m_index,
            pc: raw.m_pc,
            sp: raw.m_sp,
            fp: raw.m_fp,
            function_name: if raw.m_functionName.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(raw.m_functionName) }
                    .to_string_lossy()
                    .into_owned()
            },
            function_start: raw.m_functionStart,
            module: if raw.m_module.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(raw.m_module) }
                    .to_string_lossy()
                    .into_owned()
            },
        }
    }
}

impl fmt::Display for DebugFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{} 0x{:x} in {} ({})",
            self.index, self.pc, self.function_name, self.module
        )
    }
}

#[derive(Debug, Clone)]
pub struct DebugModule {
    pub name: String,
    pub short_name: String,
    pub address: u64,
    pub size: usize,
    pub loaded: bool,
}

impl DebugModule {
    fn from_raw(raw: &ffi::BNDebugModule) -> Self {
        Self {
            name: if raw.m_name.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(raw.m_name) }
                    .to_string_lossy()
                    .into_owned()
            },
            short_name: if raw.m_short_name.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(raw.m_short_name) }
                    .to_string_lossy()
                    .into_owned()
            },
            address: raw.m_address,
            size: raw.m_size,
            loaded: raw.m_loaded,
        }
    }
}

impl fmt::Display for DebugModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} @ 0x{:x} (size: 0x{:x})",
            self.short_name, self.address, self.size
        )
    }
}

#[derive(Debug, Clone)]
pub struct DebugRegister {
    pub name: String,
    pub value: Vec<u8>,
    pub width: usize,
    pub register_index: usize,
    pub hint: String,
}

impl DebugRegister {
    fn from_raw(raw: &ffi::BNDebugRegister) -> Self {
        // Cap width at buffer size (64 bytes)
        let width = raw.m_width.min(64);
        Self {
            name: if raw.m_name.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(raw.m_name) }
                    .to_string_lossy()
                    .into_owned()
            },
            value: raw.m_value[..width].to_vec(),
            width: raw.m_width,  // Keep original width for display
            register_index: raw.m_registerIndex,
            hint: if raw.m_hint.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(raw.m_hint) }
                    .to_string_lossy()
                    .into_owned()
            },
        }
    }

    /// Get value as u64 (for registers <= 8 bytes)
    pub fn value_u64(&self) -> u64 {
        let mut buf = [0u8; 8];
        let len = self.value.len().min(8);
        buf[..len].copy_from_slice(&self.value[..len]);
        u64::from_le_bytes(buf)
    }
}

impl fmt::Display for DebugRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.width <= 8 {
            write!(f, "{} = 0x{:x}", self.name, self.value_u64())
        } else {
            write!(f, "{} = {:02x?}", self.name, self.value)
        }
    }
}

#[derive(Debug, Clone)]
pub struct DebugBreakpoint {
    pub module: String,
    pub offset: u64,
    pub address: u64,
    pub enabled: bool,
}

impl DebugBreakpoint {
    fn from_raw(raw: &ffi::BNDebugBreakpoint) -> Self {
        Self {
            module: if raw.module.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(raw.module) }
                    .to_string_lossy()
                    .into_owned()
            },
            offset: raw.offset,
            address: raw.address,
            enabled: raw.enabled,
        }
    }
}

impl fmt::Display for DebugBreakpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Breakpoint @ 0x{:x}{}",
            self.address,
            if self.enabled { "" } else { " (disabled)" }
        )
    }
}

#[derive(Debug, Clone)]
pub struct DebugProcess {
    pub pid: u32,
    pub name: String,
}

impl DebugProcess {
    fn from_raw(raw: &ffi::BNDebugProcess) -> Self {
        Self {
            pid: raw.m_pid,
            name: if raw.m_processName.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(raw.m_processName) }
                    .to_string_lossy()
                    .into_owned()
            },
        }
    }
}

//DebuggerEvent

#[derive(Debug, Clone)]
pub struct DebuggerEvent {
    pub event_type: BNDebuggerEventType,
    pub stop_reason: Option<BNDebugStopReason>,
    pub exit_code: Option<u64>,
    pub error_message: Option<String>,
    pub address: Option<u64>,
    pub message: Option<String>,
}

impl DebuggerEvent {
    fn from_raw(raw: &ffi::BNDebuggerEvent) -> Self {
        let mut event = Self {
            event_type: raw.r#type,
            stop_reason: None,
            exit_code: None,
            error_message: None,
            address: None,
            message: None,
        };

        match raw.r#type {
            BNDebuggerEventType::TargetStoppedEventType
            | BNDebuggerEventType::AdapterStoppedEventType => {
                event.stop_reason = Some(raw.data.targetStoppedData.reason);
                event.exit_code = Some(raw.data.targetStoppedData.exitCode as u64);
            }
            BNDebuggerEventType::TargetExitedEventType
            | BNDebuggerEventType::AdapterTargetExitedEventType => {
                event.exit_code = Some(raw.data.exitData.exitCode);
            }
            BNDebuggerEventType::ErrorEventType => {
                if !raw.data.errorData.error.is_null() {
                    event.error_message = Some(
                        unsafe { CStr::from_ptr(raw.data.errorData.error) }
                            .to_string_lossy()
                            .into_owned(),
                    );
                }
            }
            BNDebuggerEventType::AbsoluteBreakpointAddedEvent
            | BNDebuggerEventType::AbsoluteBreakpointRemovedEvent
            | BNDebuggerEventType::AbsoluteBreakpointEnabledEvent
            | BNDebuggerEventType::AbsoluteBreakpointDisabledEvent => {
                event.address = Some(raw.data.absoluteAddress);
            }
            BNDebuggerEventType::StdoutMessageEventType
            | BNDebuggerEventType::BackendMessageEventType => {
                if !raw.data.messageData.message.is_null() {
                    event.message = Some(
                        unsafe { CStr::from_ptr(raw.data.messageData.message) }
                            .to_string_lossy()
                            .into_owned(),
                    );
                }
            }
            _ => {}
        }

        event
    }
}

// Helper functions

pub fn stop_reason_string(reason: BNDebugStopReason) -> String {
    unsafe {
        let ptr = ffi::BNDebuggerGetStopReasonString(reason);
        if ptr.is_null() {
            return format!("{:?}", reason);
        }
        let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
        ffi::BNDebuggerFreeString(ptr);
        s
    }
}

//Available debug adapters

pub fn available_debug_adapters(bv: &BinaryView) -> Vec<String> {
    let mut count = 0usize;
    let ptr = unsafe { ffi::BNGetAvailableDebugAdapterTypes(bv.handle as *mut _, &mut count) };
    if ptr.is_null() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(count);
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, count);
        for &s in slice {
            if !s.is_null() {
                result.push(CStr::from_ptr(s).to_string_lossy().into_owned());
            }
        }
        ffi::BNDebuggerFreeStringList(ptr, count);
    }
    result
}
