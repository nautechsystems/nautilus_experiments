use log::{debug, error, info, set_boxed_logger, set_max_level, warn};
use pyo3::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    ffi::{c_char, CStr},
    io::{self, Write},
};

pub fn time_since_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error calling `SystemTime::now.duration_since`")
        .as_nanos() as u64
}

#[derive(Debug)]
pub struct Logger;

impl Logger {
    fn new() -> Self {
        Self {}
    }

    pub fn initialize() {
        let _ = set_boxed_logger(Box::new(Logger::new())).unwrap();
        let _ = set_max_level(log::LevelFilter::Debug);
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        io::stdout()
            .write_fmt(format_args!(
                "{} {} {}\n",
                time_since_epoch(),
                record.level(),
                record.args()
            ))
            .unwrap();
        io::stdout().flush().unwrap();
    }

    fn flush(&self) {
        println!("Flushing");
    }
}

#[pyfunction]
pub fn set_global_log_collector() {
    Logger::initialize();
}

#[pyclass]
pub struct TempLogger {
    component: String,
}

#[pymethods]
impl TempLogger {
    #[new]
    pub fn new(component: String) -> Self {
        TempLogger { component }
    }

    pub fn info(slf: PyRef<'_, Self>, message: String) {
        info!("{}: {}", &slf.component, message);
    }
}

/// Loaded as nautilus_pyo3.common
#[pymodule]
pub fn core(_: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<TempLogger>()?;
    m.add_function(wrap_pyfunction!(set_global_log_collector, m)?)?;
    Ok(())
}

#[no_mangle]
pub extern "C" fn logger_init() {
    Logger::initialize()
}

#[no_mangle]
pub unsafe extern "C" fn logger_info(message: *const c_char) {
    let message = CStr::from_ptr(message).to_str().unwrap().to_string();
    dbg!("cython info");
    dbg!(&message);
    log::logger().flush();
    info!("{}", message);
}
