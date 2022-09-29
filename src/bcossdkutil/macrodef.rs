#![allow(
    clippy::unreadable_literal,
    clippy::upper_case_acronyms,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    overflowing_literals,
    unused_variables,
    unused_assignments
)]
//将rust的str转成C指针
#[macro_export]
macro_rules! str2p {
    ($x:expr) => {
        CString::new($x).unwrap().as_ptr() as *const c_char
    };
}

/// Global flag of enabling debug output.
pub static mut ENABLE_BCOSSDK_DEBUG_PRINTLNEX: bool = false;

/// Prints debug output that can be disabled by setting a global flag.
#[macro_export]
macro_rules! printlnex {
            () => ( print!("\n"));
            ($($arg:tt)*) => {
            if crate::bcossdkutil::macrodef::is_debugprint() {
                      print!("{}:{}:", file!(), line!());
                      println!($($arg)*);
            }
     };
}

pub fn is_debugprint() -> bool {
    unsafe { ENABLE_BCOSSDK_DEBUG_PRINTLNEX }
}
pub fn set_debugprint(isprint: bool) {
    unsafe { ENABLE_BCOSSDK_DEBUG_PRINTLNEX = isprint }
}
