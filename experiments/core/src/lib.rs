use log::{debug, error, info, set_boxed_logger, set_max_level, warn};
use pyo3::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    ffi::{c_char, CStr},
    io::{self, BufWriter, Write},
    sync::{self, mpsc::SyncSender},
    thread::{self},
};

pub fn time_since_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error calling `SystemTime::now.duration_since`")
        .as_nanos() as u64
}

#[derive(Debug)]
pub struct Logger {
    tx: SyncSender<LogEvent>,
}

struct LogEvent {
    ts: u64,
    level: log::Level,
    data: String,
}

impl Logger {
    fn new(tx: SyncSender<LogEvent>) -> Self {
        Self { tx }
    }

    pub fn initialize() {
        let (tx, rx) = sync::mpsc::sync_channel::<LogEvent>(0);
        let _handle = thread::spawn(move || {
            let mut writer = BufWriter::new(io::stdout());
            while let Ok(LogEvent { ts, level, data }) = rx.recv() {
                if ts == u64::MAX {
                    break;
                }
                writer
                    .write_fmt(format_args!("{} {} {}\n", ts, level, data))
                    .unwrap();
                let _ = writer.flush().unwrap();
            }
        });
        let logger = Self::new(tx);
        let _ = set_boxed_logger(Box::new(logger)).unwrap();
        let _ = set_max_level(log::LevelFilter::Debug);
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        self.tx
            .send(LogEvent {
                ts: time_since_epoch(),
                level: record.level(),
                data: format_args!("{}", record.args()).to_string(),
            })
            .unwrap();
    }

    fn flush(&self) {
        self.tx
            .send(LogEvent {
                ts: u64::MAX,
                level: log::Level::Debug,
                data: "".to_string(),
            })
            .unwrap();
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

    pub fn debug(slf: PyRef<'_, Self>, message: String) {
        debug!("{}: {}", &slf.component, message);
    }

    pub fn info(slf: PyRef<'_, Self>, message: String) {
        info!("{}: {}", &slf.component, message);
    }

    pub fn warn(slf: PyRef<'_, Self>, message: String) {
        warn!("{}: {}", &slf.component, message);
    }

    pub fn error(slf: PyRef<'_, Self>, message: String) {
        error!("{}: {}", &slf.component, message);
    }

    pub fn flush(_slf: PyRef<'_, Self>) {
        log::logger().flush();
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
pub unsafe extern "C" fn logger_debug(message: *const c_char) {
    let message = CStr::from_ptr(message).to_str().unwrap().to_string();
    debug!("{}", message);
}

#[no_mangle]
pub unsafe extern "C" fn logger_info(message: *const c_char) {
    let message = CStr::from_ptr(message).to_str().unwrap().to_string();
    info!("{}", message);
}

#[no_mangle]
pub unsafe extern "C" fn logger_error(message: *const c_char) {
    let message = CStr::from_ptr(message).to_str().unwrap().to_string();
    error!("{}", message);
}

#[no_mangle]
pub unsafe extern "C" fn logger_warn(message: *const c_char) {
    let message = CStr::from_ptr(message).to_str().unwrap().to_string();
    warn!("{}", message);
}
