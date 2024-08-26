use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::{create_exception, PyTypeCheck};

create_exception!(inner, MyError, PyException, "Some description.");

#[pyfunction]
fn raise_myerror() -> PyResult<()> {
    let err = MyError::new_err("Some error happened.");
    Err(err)
}

#[pymodule]
pub fn inner(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add(
        <MyError as PyTypeCheck>::NAME,
        m.py().get_type_bound::<MyError>(),
    )?;
    m.add_function(wrap_pyfunction!(raise_myerror, m)?)?;
    Ok(())
}
