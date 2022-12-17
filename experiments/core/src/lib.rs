use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter, Result};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use pyo3::prelude::*;
use pyo3::types::{PyCapsule, PyString};
use pyo3::{
    ffi, pyfunction, FromPyPointer, IntoPyPointer, Py, PyObject, PyResult, Python, ToPyObject,
};

use std::{ffi::c_void, ptr::null};
use uuid::Uuid;

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[allow(clippy::box_collection)] // C ABI compatibility
#[allow(clippy::redundant_allocation)] // C ABI compatibility
pub struct UUID4 {
    pub value: Box<Rc<String>>,
}

impl UUID4 {
    pub fn new() -> UUID4 {
        let uuid = Uuid::new_v4();
        UUID4 {
            value: Box::new(Rc::new(uuid.to_string())),
        }
    }
}

impl From<&str> for UUID4 {
    fn from(s: &str) -> Self {
        let uuid = Uuid::try_parse(s).expect("invalid UUID string");
        UUID4 {
            value: Box::new(Rc::new(uuid.to_string())),
        }
    }
}
impl Display for UUID4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.value)
    }
}

////////////////////////////////////////////////////////////////////////////////
// C API
////////////////////////////////////////////////////////////////////////////////
#[no_mangle]
pub extern "C" fn uuid4_new() -> UUID4 {
    UUID4::new()
}

#[no_mangle]
pub extern "C" fn uuid4_clone(uuid4: &UUID4) -> UUID4 {
    uuid4.clone()
}

#[no_mangle]
pub extern "C" fn uuid4_free(uuid4: UUID4) {
    drop(uuid4); // Memory freed here
}

/// Returns a `UUID4` from a valid Python object pointer.
///
/// # Safety
/// - Assumes `ptr` is borrowed from a valid Python UTF-8 `str`.
#[no_mangle]
pub unsafe extern "C" fn uuid4_from_pystr(ptr: *mut ffi::PyObject) -> UUID4 {
    UUID4::from(
        { Python::with_gil(|py| PyString::from_borrowed_ptr(py, ptr).to_string()) }.as_str(),
    )
}

/// Returns a pointer to a valid Python UTF-8 string.
///
/// # Safety
/// - Assumes that since the data is originating from Rust, the GIL does not need
/// to be acquired.
/// - Assumes you are immediately returning this pointer to Python.
#[no_mangle]
pub unsafe extern "C" fn uuid4_to_pystr(uuid: &UUID4) -> *mut ffi::PyObject {
    let s = uuid.value.as_str();
    Python::with_gil(|py| {
        let pystr: Py<PyString> = PyString::new(py, s).into();
        pystr.into_ptr()
    })
}

#[no_mangle]
pub extern "C" fn uuid4_eq(lhs: &UUID4, rhs: &UUID4) -> u8 {
    (lhs == rhs) as u8
}

#[no_mangle]
pub extern "C" fn uuid4_hash(uuid: &UUID4) -> u64 {
    let mut h = DefaultHasher::new();
    uuid.hash(&mut h);
    h.finish()
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CVec {
    pub ptr: *mut c_void,
    pub len: usize,
    pub cap: usize,
}

unsafe impl Send for CVec {}

impl CVec {
    pub fn default() -> Self {
        CVec {
            ptr: null() as *const bool as *mut c_void,
            len: 0,
            cap: 0,
        }
    }
}

/// Consumes and leaks the Vec, returning a mutable pointer to the contents as
/// a 'CVec'. The memory has been leaked and now exists for the lifetime of the
/// program unless dropped manually.
/// Note: drop the memory by reconstructing the vec using from_raw_parts method
/// as shown in the test below.
impl<T> From<Vec<T>> for CVec {
    fn from(data: Vec<T>) -> Self {
        if data.is_empty() {
            CVec::default()
        } else {
            let len = data.len();
            let cap = data.capacity();
            CVec {
                ptr: &mut data.leak()[0] as *mut T as *mut c_void,
                len,
                cap,
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// C API
////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "C" fn cvec_new() -> CVec {
    CVec::default()
}

#[no_mangle]
/// Specialize to UUID4 for test
pub extern "C" fn cvec_free(cvec: CVec) {
    let CVec { ptr, len, cap } = cvec;
    let data: Vec<UUID4> = unsafe { Vec::from_raw_parts(ptr as *mut UUID4, len, cap) };
    drop(data) // Memory freed here
}

////////////////////////////////////////////////////////////////////////////////
// Python API
////////////////////////////////////////////////////////////////////////////////

#[pyfunction]
fn generate_data(len: u32) -> PyObject {
    let data: CVec = (0..len)
        .into_iter()
        .map(|_| UUID4::new())
        .collect::<Vec<UUID4>>()
        .into();

    Python::with_gil(|py| {
        let a = PyCapsule::new(py, data, None).unwrap();
        a.to_object(py)
    })
}

#[pymodule]
fn core(_: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_data, m)?)?;
    Ok(())
}
