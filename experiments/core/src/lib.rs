use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter, Result};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use pyo3::types::{PyString, PyUnicode};
use pyo3::{ffi, FromPyPointer, IntoPyPointer, Py, Python};

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
