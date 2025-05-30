use std::{
    ffi::c_char,
    slice,
};
use ustr::Ustr;

#[unsafe(no_mangle)]
pub extern "C" fn ustr_from_str(s: *const c_char, len: usize) -> Ustr {
    println!("Ptr: {:p}, len: {}", s, len);
    if s.is_null() {
        println!("Found null pointer");
        panic!("Do no pass null pointer");
    }
    let str = unsafe { str::from_utf8_unchecked(slice::from_raw_parts(s as *const u8, len)) };
    Ustr::from(str)
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_function() -> i32 {
    println!("debug_function");
    0
}
