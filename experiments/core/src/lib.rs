use std::ffi::c_void;
use std::rc::Rc;
use std::slice;

use cvec::CVec;
use pyo3::types::PyString;
use pyo3::{ffi, FromPyPointer, Python};

pub mod cvec;

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[allow(clippy::box_collection)] // C ABI compatibility
pub struct Symbol {
    value: Box<Rc<String>>,
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
    Symbol {
        value: Box::new(Rc::new(v)),
    }
}

#[no_mangle]
pub extern "C" fn symbol_clone_void(symbol: *const c_void) -> Symbol {
    unsafe { &*(symbol as *const Symbol) }.clone()
}

#[no_mangle]
pub extern "C" fn symbol_clone(symbol: &Symbol) -> Symbol {
    symbol.clone()
}

/// Frees the memory for the given `symbol` by dropping.
#[no_mangle]
pub extern "C" fn symbol_free(symbol: Symbol) {
    drop(symbol); // Memory freed here
}

#[no_mangle]
pub extern "C" fn symbol_vector_test(data: *mut c_void, len: usize) {
    let data: &[Symbol] = unsafe { slice::from_raw_parts(data as *const Symbol, len) };
    let v = &data[len - 1];
    dbg!(Rc::strong_count(&v.value));
    dbg!(len, &data[len - 1]);
}

#[no_mangle]
pub extern "C" fn symbol_cvec_test(data: cvec::CVec) {
    let CVec { ptr, len, cap: _ } = data;
    let data: &[Symbol] = unsafe { slice::from_raw_parts(ptr as *const Symbol, len) };
    let v = &data[len - 1];
    dbg!(Rc::strong_count(&v.value));
    dbg!(len, &data[len - 1]);
}
