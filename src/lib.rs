use std::{cell::UnsafeCell, collections::HashMap, ffi::c_char, sync::OnceLock};

use indexmap::IndexMap;
use pyo3::prelude::*;
use ustr::Ustr;

extern "C" {
    pub fn ustr_from_str(s: *const c_char, len: usize) -> Ustr;
    pub fn debug_function() -> i32;
}

// #[no_mangle]
// pub extern "C" fn ustr_from_str(s: *const c_char, len: usize) -> Ustr {
//     panic!("This should be overridden by LD_PRELOAD");
// }

// #[no_mangle]
// pub extern "C" fn debug_function() -> i32 {
//     panic!("This should be overridden by LD_PRELOAD");
// }

// // Add this type alias
// type UstrFromStrFn = extern "C" fn(*const c_char, usize) -> Ustr;

// Modify your wrapper function
// pub fn ustr_from(s: &str) -> Ustr {
//     // Get function pointer - this level of indirection helps with symbol resolution
//     let func: UstrFromStrFn = ustr_from_str;
//     func(s.as_ptr() as *const c_char, s.len())
// }

pub fn ustr_from(s: &str) -> Ustr {
    // Get function pointer - this level of indirection helps with symbol resolution
    println!("Ptr: {:p}, len: {}", s.as_ptr(), s.len());
    unsafe { ustr_from_str(s.as_ptr() as *const c_char, s.len()) }
}

pub struct SharedVal(Box<UnsafeCell<IndexMap<Ustr, PyObject>>>);

// SAFETY: Cannot be sent across thread boundaries
#[allow(unsafe_code)]
unsafe impl Send for SharedVal {}
#[allow(unsafe_code)]
unsafe impl Sync for SharedVal {}

#[no_mangle]
pub static VALUE: OnceLock<SharedVal> = OnceLock::new();

fn get_value_ref() -> &'static SharedVal {
    VALUE.get_or_init(|| SharedVal(Box::new(UnsafeCell::new(IndexMap::new()))))
}

#[pyfunction]
pub fn print_value_ptr_address() {
    let val = get_value_ref();
    let ptr = val.0.get();
    println!("Value pointer address: {:p}", ptr);
}

#[pyfunction]
pub fn export_value_ptr() -> u64 {
    let val = get_value_ref();
    // Return pointer to the UnsafeCell, not its contents
    (&*val.0) as *const UnsafeCell<IndexMap<Ustr, PyObject>> as u64
}

#[pyfunction]
pub fn set_value_from_ptr(addr: u64) -> PyResult<()> {
    if addr != 0 {
        let ptr = addr as *mut UnsafeCell<IndexMap<Ustr, PyObject>>;
        if !ptr.is_null() {
            let _ = VALUE.set(SharedVal(unsafe { Box::from_raw(ptr) }));
        };
    }
    Ok(())
}

#[pyfunction]
pub fn get_value(key: &str) -> Option<PyObject> {
    let val = get_value_ref();
    let key = ustr_from(key);
    Python::with_gil(|py| unsafe { (*val.0.get()).get(&key).map(|v| v.clone_ref(py)) })
}

#[pyfunction]
pub fn append_value(key: &str, val: PyObject) {
    let shared_val = get_value_ref();
    unsafe {
        (*shared_val.0.get()).insert(ustr_from(key), val);
    }
}

#[pyfunction]
pub fn print_hello_world(prefix: &str) {
    println!("{} hello world", prefix);
}

/// We modify sys modules so that submodule can be loaded directly as
/// import supermodule.submodule
///
/// Also re-exports all submodule attributes so they can be imported directly from `nautilus_pyo3`
/// refer: <https://github.com/PyO3/pyo3/issues/2644>
#[pymodule]
fn pyo3_test(_: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(print_hello_world, m)?)?;
    m.add_function(wrap_pyfunction!(get_value, m)?)?;
    m.add_function(wrap_pyfunction!(append_value, m)?)?;
    m.add_function(wrap_pyfunction!(export_value_ptr, m)?)?;
    m.add_function(wrap_pyfunction!(set_value_from_ptr, m)?)?;
    m.add_function(wrap_pyfunction!(print_value_ptr_address, m)?)?;

    // let sys = PyModule::import(py, "sys")?;
    // let modules = sys.getattr("modules")?;
    // let sys_modules: &Bound<'_, PyAny> = modules.downcast()?;
    // let module_name = "nautilus_experiments";

    // // Set pyo3_nautilus to be recognized as a subpackage
    // sys_modules.set_item(module_name, m)?;

    Ok(())
}
