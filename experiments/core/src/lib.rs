use std::ptr::null;

use pyo3::types::PyString;
use pyo3::{ffi, FromPyPointer, Python};

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[allow(clippy::box_collection)] // C ABI compatibility
pub struct Symbol {
    value: Box<String>,
}

impl Symbol {
    fn print_debug_info(mut self) -> Self {
        let ptr = self.value.as_ptr();
        println!("string value {:p}", ptr);
        unsafe {
            println!(
                "create new symbol: {}{}{}",
                *ptr as char,
                *ptr.offset(1) as char,
                *ptr.offset(2) as char
            );
        }

        let ptr = Box::into_raw(self.value);
        println!("symbol_new: rust sting box value {:p}", ptr);
        self.value = unsafe { Box::from_raw(ptr) };
        self
    }
}

////////////////////////////////////////////////////////////////////////////////
// C API
////////////////////////////////////////////////////////////////////////////////

/// Returns a Nautilus identifier from a valid Python object pointer.
///
/// # Safety
/// - Assumes `ptr` is borrowed from a valid Python UTF-8 `str`.
#[no_mangle]
pub unsafe extern "C" fn symbol_new(ptr: *mut ffi::PyObject) -> Symbol {
    let v = Python::with_gil(|py| PyString::from_borrowed_ptr(py, ptr).to_string());
    let value = Box::new(v);
    let v = Symbol { value };
    Symbol::print_debug_info(v)
}

#[no_mangle]
pub extern "C" fn symbol_copy(symbol: &Symbol) -> Symbol {
    symbol.clone()
}

/// Frees the memory for the given `symbol` by dropping.
#[no_mangle]
pub extern "C" fn symbol_free(mut symbol: Symbol) {
    let ptr: *const String = Box::into_raw(symbol.value);
    if ptr != null() {
        symbol.value = unsafe { Box::from_raw(ptr as *mut String) };
        let symbol = Symbol::print_debug_info(symbol);
        drop(symbol); // Memory freed here
    } else {
        println!("symbol value is null");
    }
}
