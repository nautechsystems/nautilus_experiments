use chrono::{
    prelude::{DateTime, Utc},
    SecondsFormat,
};
use log::{debug, error, info, log, set_boxed_logger, warn};
use pyo3::prelude::*;
use std::{
    io::{self, BufWriter, Stdout, Write},
    ops::Deref,
    sync::{
        atomic::{self, AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use std::{
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use time::{
    format_description::well_known::{
        self,
        iso8601::{Config, EncodedConfig},
        Iso8601,
    },
    macros::format_description,
    OffsetDateTime, UtcOffset,
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

#[derive(Clone, Debug)]
pub struct Logger {
    time: AtomicTime,
    writer: Arc<Mutex<BufWriter<Stdout>>>,
}

impl Logger {
    fn new() -> Self {
        Self {
            time: AtomicTime::new(true, 0),
            writer: Arc::new(Mutex::new(BufWriter::new(io::stdout()))),
        }
    }

    pub fn initialize() {
        let logger = Self::new();
        set_boxed_logger(Box::new(logger));
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
        self.writer.lock().unwrap().write_fmt(format_args!(
            "{} {} {}",
            self.time.get_time_ns(),
            target,
            record.args()
        )).unwrap();
    }

    fn flush(&self) {
        self.writer.lock().unwrap().flush().unwrap();
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

    // pub fn debug(slf: PyRef<'_, Self>, message: String) {
    //     debug!(component = slf.component.clone(); message);
    // }

    // pub fn info(slf: PyRef<'_, Self>, message: String) {
    //     info!(message, component = slf.component.clone());
    // }

    // pub fn warn(slf: PyRef<'_, Self>, message: String) {
    //     warn!(message, component = slf.component.clone());
    // }

    // pub fn error(slf: PyRef<'_, Self>, message: String) {
    //     error!(message, component = slf.component.clone());
    // }
}

/// Loaded as nautilus_pyo3.common
#[pymodule]
pub fn core(_: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<TempLogger>()?;
    m.add_function(wrap_pyfunction!(set_global_log_collector, m)?)?;
    Ok(())
}
