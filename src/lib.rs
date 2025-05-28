use std::{cell::RefCell, rc::Rc, sync::OnceLock};

use pyo3::prelude::*;

pub struct SharedVal(Rc<RefCell<u64>>);

// SAFETY: Cannot be sent across thread boundaries
#[allow(unsafe_code)]
unsafe impl Send for SharedVal {}
#[allow(unsafe_code)]
unsafe impl Sync for SharedVal {}


pub static VALUE: OnceLock<SharedVal> = OnceLock::new();

#[pyfunction]
pub fn get_value() -> u64 {
    *VALUE.get_or_init(|| {
        SharedVal(Rc::new(RefCell::new(0)))
    }).0.borrow()
}

#[pyfunction]
pub fn set_value(val: u64) {
    let var = VALUE.get().unwrap();
    *(var.0.borrow_mut()) = val;
}

#[pyfunction]
fn print_hello_world() {
    println!("Hello, world!");
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
    m.add_function(wrap_pyfunction!(set_value, m)?)?;

    // let sys = PyModule::import(py, "sys")?;
    // let modules = sys.getattr("modules")?;
    // let sys_modules: &Bound<'_, PyAny> = modules.downcast()?;
    // let module_name = "nautilus_experiments";

    // // Set pyo3_nautilus to be recognized as a subpackage
    // sys_modules.set_item(module_name, m)?;

    Ok(())
}

fn re_export_module_attributes(
    parent_module: &Bound<'_, PyModule>,
    submodule_name: &str,
) -> PyResult<()> {
    let submodule = parent_module.getattr(submodule_name)?;
    for item_name in submodule.dir()? {
        let item_name_str: &str = item_name.extract()?;
        if let Ok(attr) = submodule.getattr(item_name_str) {
            parent_module.add(item_name_str, attr)?;
        }
    }

    Ok(())
}
