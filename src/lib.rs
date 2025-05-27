use pyo3::{
    prelude::*,
    types::{PyDict, PyString},
};

use inner::inner;

#[pymodule]
fn outer(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    let sys = PyModule::import(py, "sys")?;
    let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;
    let module_name = "outer";

    // Set pyo3_nautilus to be recognized as a subpackage
    sys_modules.set_item(module_name, m)?;

    let n = "inner";
    let submodule = pyo3::wrap_pymodule!(inner);
    m.add_wrapped(submodule)?;
    sys_modules.set_item(format!("{module_name}.{n}"), m.getattr(n)?)?;
    re_export_module_attributes(m, n)?;

    Ok(())
}

fn re_export_module_attributes(parent_module: &PyModule, submodule_name: &str) -> PyResult<()> {
    let submodule = parent_module.getattr(submodule_name)?;
    for item in submodule.dir() {
        let item_name: &PyString = item.extract()?;
        if let Ok(attr) = submodule.getattr(item_name) {
            parent_module.add(item_name.to_str()?, attr)?;
        }
    }

    Ok(())
}
