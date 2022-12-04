use std::ffi::c_void;
use std::rc::Rc;
use std::slice;

use pyo3::types::PyString;
use pyo3::{ffi, FromPyPointer, IntoPyPointer, Py, Python};
use std::fmt::{Debug, Display, Formatter, Result};
use uuid::Uuid;

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq)]
#[allow(clippy::box_collection)] // C ABI compatibility
pub struct UUID4 {
    value: Box<String>,
}

impl UUID4 {
    pub fn new() -> UUID4 {
        let uuid = Uuid::new_v4();
        UUID4 {
            value: Box::new(uuid.to_string()),
        }
    }
}

impl From<&str> for UUID4 {
    fn from(s: &str) -> Self {
        let uuid = Uuid::parse_str(s).unwrap();
        UUID4 {
            value: Box::new(uuid.to_string()),
        }
    }
}

/// Returns an owned string from a valid Python object pointer.
///
/// # Safety
///
/// - `ptr` must be borrowed from a valid Python UTF-8 `str`.
#[inline(always)]
pub unsafe fn pystr_to_string(ptr: *mut ffi::PyObject) -> String {
    Python::with_gil(|py| PyString::from_borrowed_ptr(py, ptr).to_string())
}

/// Returns a pointer to a valid Python UTF-8 string.
///
/// # Safety
///
/// - Assumes that since the data is originating from Rust, the GIL does not need
/// to be acquired.
/// - Assumes you are immediately returning this pointer to Python.
#[inline(always)]
pub unsafe fn string_to_pystr(s: &str) -> *mut ffi::PyObject {
    let py = Python::assume_gil_acquired();
    let pystr: Py<PyString> = PyString::new(py, s).into();
    pystr.into_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn uuid4_from_pystr(ptr: *mut ffi::PyObject) -> UUID4 {
    UUID4 {
        value: Box::new(pystr_to_string(ptr)),
    }
}

/// Returns a pointer to a valid Python UTF-8 string.
///
/// # Safety
///
/// - Assumes that since the data is originating from Rust, the GIL does not need
/// to be acquired.
/// - Assumes you are immediately returning this pointer to Python.
#[no_mangle]
pub unsafe extern "C" fn uuid4_to_pystr(uuid: &UUID4) -> *mut ffi::PyObject {
    string_to_pystr(uuid.value.as_str())
}

#[no_mangle]
pub extern "C" fn uuid4_free(uuid4: UUID4) {
    drop(uuid4); // Memory freed here
}

#[no_mangle]
pub extern "C" fn uuid4_new() -> UUID4 {
    UUID4::new()
}
