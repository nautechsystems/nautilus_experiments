use pyo3::prelude::*;
use std::ffi::CString;
use pyo3_test::{export_value_ptr, get_value, print_value_ptr_address, set_value};

fn main() {
    let root = env!("CARGO_MANIFEST_DIR");
    let code = std::fs::read_to_string(format!("{}/test.py", root)).unwrap();
    let code = CString::new(code).unwrap();
    let filename = CString::new("test".to_string()).unwrap();
    let module = CString::new("test".to_string()).unwrap();

    pyo3::prepare_freethreaded_python();
    println!("{}", get_value());
    set_value(1);
    println!("{}", get_value());
    
    print_value_ptr_address();
    let ptr = export_value_ptr();

    Python::with_gil(|py| {
        let pymod = PyModule::from_code(py, &code, &filename, &module).unwrap();
        let test_class = pymod.getattr("Test").unwrap();
        let test_instance = test_class.call0().unwrap();
        
        // Set the static variable to the pointer
        let set_val_method = pymod.getattr("set_value_from_ptr").unwrap();
        set_val_method.call1((ptr,)).unwrap();

        // Check the static variable is updated
        let get_val_method = pymod.getattr("get_value").unwrap();
        let val = get_val_method.call0().unwrap();
        println!("{}", val);
        
        let do_method = test_instance.getattr("do").unwrap();
        do_method.call0().unwrap();
    });
    
    println!("{}", get_value());
}
