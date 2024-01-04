use log::{debug, error, info, log, set_boxed_logger, set_max_level, warn};
use pyo3::prelude::*;
use std::{
    ffi::{c_char, CStr},
    io::{self, BufWriter, Stdout, Write},
    ops::Deref,
    sync::{
        self,
        atomic::{self, AtomicBool, AtomicU64, Ordering},
        mpsc::SyncSender,
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};
use std::{
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Atomic clock stores the last recorded time in nanoseconds
///
/// It uses AtomicU64 to atomically update the value using only immutable
/// references.
///
/// AtomicClock can act as a live clock and static clock based on its mode.
#[derive(Debug, Clone)]
#[pyclass]
pub struct AtomicTime {
    /// Atomic clock is operating in live or static mode
    mode: Arc<AtomicBool>,
    /// The last recorded time in nanoseconds for the clock
    timestamp_ns: Arc<AtomicU64>,
}

impl Deref for AtomicTime {
    type Target = AtomicU64;

    fn deref(&self) -> &Self::Target {
        &self.timestamp_ns
    }
}

#[pymethods]
impl AtomicTime {
    fn live(&mut self) {
        self.mode.store(true, Ordering::Relaxed)
    }

    fn static_mode(&mut self) {
        self.mode.store(false, Ordering::Relaxed)
    }

    /// Increments current time with a delta and returns the updated time
    pub fn increment_time(&self, delta: u64) -> u64 {
        self.fetch_add(delta, Ordering::Relaxed) + delta
    }
}

impl AtomicTime {
    /// New atomic clock set with the given time
    pub fn new(mode: bool, time: u64) -> Self {
        AtomicTime {
            mode: Arc::new(AtomicBool::new(mode)),
            timestamp_ns: Arc::new(AtomicU64::new(time)),
        }
    }

    /// Get time in nanoseconds.
    ///
    /// * Live mode returns current wall clock time since UNIX epoch (unique and monotonic)
    /// * Static mode returns currently stored time.
    pub fn get_time_ns(&self) -> u64 {
        match self.mode.load(Ordering::Relaxed) {
            true => self.time_since_epoch(),
            false => self.timestamp_ns.load(Ordering::Relaxed),
        }
    }

    /// Stores and returns current time
    pub fn time_since_epoch(&self) -> u64 {
        // increment by 1 nanosecond to keep increasing time
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Error calling `SystemTime::now.duration_since`")
            .as_nanos() as u64
            + 1;
        let last = self.load(Ordering::SeqCst) + 1;
        let new = now.max(last);
        self.store(new, Ordering::SeqCst);
        new
    }
}

// impl FormatTime for AtomicTime {
//     fn format_time(&self, w: &mut format::Writer<'_>) -> std::fmt::Result {
//         let timestamp_ns = self.get_time_ns();
//         let dt = DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_nanos(timestamp_ns));
//         write!(w, "{}", dt.to_rfc3339_opts(SecondsFormat::Nanos, true))
//     }
// }

pub struct LoggerHandle {
    handle: JoinHandle<()>,
}

impl LoggerHandle {
    pub fn shutdown(self) {
        log::logger().flush();
        let _ = self.handle.join().unwrap();
    }
}

#[derive(Debug)]
pub struct Logger {
    time: AtomicTime,
    tx: SyncSender<LogEvent>,
}

struct LogEvent {
    ts: u64,
    level: log::Level,
    data: String,
}

impl Logger {
    fn new(tx: SyncSender<LogEvent>) -> Self {
        Self {
            time: AtomicTime::new(true, 0),
            tx,
        }
    }

    pub fn initialize() -> LoggerHandle {
        let (tx, rx) = sync::mpsc::sync_channel::<LogEvent>(0);
        let handle = thread::spawn(move || {
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

            dbg!("quitting writer thread");
            let _ = writer.flush().unwrap();
        });
        let logger = Self::new(tx);
        let logger_handle = LoggerHandle { handle };
        let _ = set_boxed_logger(Box::new(logger)).unwrap();
        let _ = set_max_level(log::LevelFilter::Debug);
        logger_handle
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let target = record
            .key_values()
            .get("component".into())
            .map(|v| v.to_string())
            .unwrap_or_else(|| record.metadata().target().to_string());
        self.tx
            .send(LogEvent {
                ts: self.time.get_time_ns(),
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

    pub fn flush(slf: PyRef<'_, Self>) {
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
