use pyo3::prelude::*;
use std::str::FromStr;
use tracing::{debug, error, info, warn, Level};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{fmt::Layer, prelude::*, EnvFilter, Registry};

/// Guards the log collector and flushes it when dropped
///
/// This struct must be dropped when the application has completed operation
/// it ensures that the any pending log lines are flushed before the application
/// closes.
#[pyclass]
pub struct LogGuard {
    guards: Vec<WorkerGuard>,
}

/// Sets the global log collector
///
/// stdout_level: Set the level for the stdout writer
/// stderr_level: Set the level for the stderr writer
/// file_level: Set the level, the directory and the prefix for the file writer
///
/// It also configures a top level filter based on module/component name.
/// The format for the string is component1=info,component2=debug.
/// For e.g. network=error,kernel=info
///
/// # Safety
/// Should only be called once during an applications run, ideally at the
/// beginning of the run.
#[pyfunction]
pub fn set_global_log_collector(
    stdout_level: Option<String>,
    stderr_level: Option<String>,
    file_level: Option<(String, String, String)>,
) -> LogGuard {
    let mut guards = Vec::new();
    let stdout_sub_builder = stdout_level.map(|stdout_level| {
        let stdout_level = Level::from_str(&stdout_level).unwrap();
        let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());
        guards.push(guard);
        Layer::default().with_writer(non_blocking.with_max_level(stdout_level))
    });
    let stderr_sub_builder = stderr_level.map(|stderr_level| {
        let stderr_level = Level::from_str(&stderr_level).unwrap();
        let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());
        guards.push(guard);
        Layer::default().with_writer(non_blocking.with_max_level(stderr_level))
    });
    let file_sub_builder = file_level.map(|(dir_path, file_prefix, file_level)| {
        let file_level = Level::from_str(&file_level).unwrap();
        let rolling_log = RollingFileAppender::new(Rotation::NEVER, dir_path, file_prefix);
        let (non_blocking, guard) = tracing_appender::non_blocking(rolling_log);
        guards.push(guard);
        Layer::default().with_writer(non_blocking.with_max_level(file_level))
    });

    Registry::default()
        .with(stderr_sub_builder)
        .with(stdout_sub_builder)
        .with(file_sub_builder)
        .with(EnvFilter::from_default_env())
        .init();

    LogGuard { guards }
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
        debug!(message, component = slf.component.clone());
    }

    pub fn info(slf: PyRef<'_, Self>, message: String) {
        info!(message, component = slf.component.clone());
    }

    pub fn warn(slf: PyRef<'_, Self>, message: String) {
        warn!(message, component = slf.component.clone());
    }

    pub fn error(slf: PyRef<'_, Self>, message: String) {
        error!(message, component = slf.component.clone());
    }
}

/// Loaded as nautilus_pyo3.common
#[pymodule]
pub fn core(_: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<LogGuard>()?;
    m.add_class::<TempLogger>()?;
    m.add_function(wrap_pyfunction!(set_global_log_collector, m)?)?;
    Ok(())
}
