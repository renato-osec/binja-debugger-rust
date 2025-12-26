#[macro_export]
macro_rules! curr_func {
    ($dbg:expr) => {
        curr_func!($dbg, "no function at IP")
    };
    ($dbg:expr, $msg:expr) => {{
        use binaryninja::binary_view::BinaryViewExt;
        let ip = $dbg.ip();
        let bv = $dbg.data().expect("no debugger data");
        bv.functions_containing(ip)
            .iter()
            .next()
            .map(|f| f.to_owned())
            .expect($msg)
    }};
}

#[macro_export]
macro_rules! comment {
    ($targets:expr) => {{
        let inner = $targets
            .iter()
            .map(|x| format!("0x{:x}", x))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{{{}}}", inner)
    }};
}
