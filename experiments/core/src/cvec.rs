use pyo3::pyclass;
use std::{ffi::c_void, ptr::null};

/// CVec is a C compatible struct that stores an opaque pointer to a block of
/// memory, it's length and the capacity of the vector it was allocated from.
///
/// NOTE: Changing the values here may lead to undefined behaviour when the
/// memory is dropped.
#[repr(C)]
#[pyclass]
#[derive(Clone, Copy)]
pub struct CVec {
    /// Opaque pointer to block of memory storing elements to access the
    /// elements cast it to the underlying type.
    pub ptr: *mut c_void,
    /// The number of elements in the block.
    #[pyo3(get, set)]
    pub len: usize,
    /// The capacity of vector from which it was allocated.
    /// Used when deallocating the memory
    pub cap: usize,
}

/// Empty derivation for Send to satisfy `pyclass` requirements
/// however this is only designed for single threaded use for now
unsafe impl Send for CVec {}

impl CVec {
    pub fn default() -> Self {
        CVec {
            // explicitly type cast the pointer to some type
            // to satisfy the compiler. Since the pointer is
            // null it works for any type.
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
/// # Safety
/// - Assumes `chunk` is a valid `ptr` pointer to a contiguous byte array
/// Default drop assumes the chunk is byte buffer that came from a Vec<u8>
pub extern "C" fn cvec_free(cvec: CVec) {
    let CVec { ptr, len, cap } = cvec;
    let data: Vec<u8> = unsafe { Vec::from_raw_parts(ptr as *mut u8, len, cap) };
    drop(data) // Memory freed here
}
