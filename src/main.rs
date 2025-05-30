use pyo3::prelude::*;
use pyo3_test::{export_value_ptr, get_value, print_value_ptr_address};
use std::ffi::CString;

fn main() {
    // Your actual main code here
    println!("Starting program");

    let root = env!("CARGO_MANIFEST_DIR");
    let code = std::fs::read_to_string(format!("{}/test.py", root)).unwrap();
    let code = CString::new(code).unwrap();
    let filename = CString::new("test".to_string()).unwrap();
    let module = CString::new("test".to_string()).unwrap();

    pyo3::prepare_freethreaded_python();
    println!("{:?}", get_value("1"));
    println!("{:?}", get_value("2"));

    print_value_ptr_address();
    let ptr = export_value_ptr();

    Python::with_gil(|py| {
        let pymod = PyModule::from_code(py, &code, &filename, &module).unwrap();

        // Set the static variable to the pointer
        let set_val_method = pymod.getattr("set_value_from_ptr").unwrap();
        set_val_method.call1((ptr,)).unwrap();

        // Set the static variable to the pointer
        let set_val_method = pymod.getattr("print_value_ptr_address").unwrap();
        set_val_method.call0().unwrap();

        // Check the static variable is updated
        let get_val_method = pymod.getattr("get_value").unwrap();
        let val = get_val_method.call1(("2",)).unwrap();
        println!("Got value from python get_value method: {}", val);
    });

    println!("{:?}", get_value("2"));
}
