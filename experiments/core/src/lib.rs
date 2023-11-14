use pyo3::prelude::*;
use std::str::FromStr;
use time::{
    format_description::well_known::{
        self,
        iso8601::{Config, EncodedConfig},
        Iso8601,
    },
    macros::format_description,
};
use tracing::{debug, error, info, warn, Level};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_core::{Event, Subscriber};
use tracing_subscriber::fmt::{
    format::{self, FormatEvent, FormatFields},
    FmtContext, FormattedFields,
};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{
    fmt,
    fmt::time::{ChronoUtc, UtcTime},
    prelude::*,
    EnvFilter, Registry,
};

/// Guards the log collector and flushes it when dropped
///
/// This struct must be dropped when the application has completed operation
/// it ensures that the any pending log lines are flushed before the application
/// closes.
#[pyclass]
pub struct LogGuard {
    guards: Vec<WorkerGuard>,
}

const TIME_FORMAT_CONFIG: EncodedConfig = Config::DEFAULT
    .set_formatted_components(well_known::iso8601::FormattedComponents::DateTime)
    .encode();
// const TIME_FORMAT_CONFIG: EncodedConfig = Config::DEFAULT.encode();

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
    // let timer = UtcTime::new(format_description!(
    //     "[year]-[month]-[day]T[hour repr:24]:[minute]:[second].[subsecond digits:9]Z"
    // ));
    // let timer = UtcTime::rfc_3339();
    let timer = ChronoUtc::rfc_3339();
    let timer = ChronoUtc::new("%FT%T.%9f%:z".to_string());
    // let timer = UtcTime::new(Iso8601::DEFAULT);
    // let time_format = Iso8601<TIME_FORMAT_CONFIG>{};
    // let timer = UtcTime::new(Iso8601::<TIME_FORMAT_CONFIG> {});
    let stdout_sub_builder = stdout_level.map(|stdout_level| {
        let stdout_level = Level::from_str(&stdout_level).unwrap();
        let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());
        guards.push(guard);
        fmt::Layer::default()
            .with_timer(timer)
            // .event_format(MyFormatter)
            .with_writer(non_blocking.with_max_level(stdout_level))
    });
    let stderr_sub_builder = stderr_level.map(|stderr_level| {
        let stderr_level = Level::from_str(&stderr_level).unwrap();
        let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());
        guards.push(guard);
        fmt::Layer::default().with_writer(non_blocking.with_max_level(stderr_level))
    });
    let file_sub_builder = file_level.map(|(dir_path, file_prefix, file_level)| {
        let file_level = Level::from_str(&file_level).unwrap();
        let rolling_log = RollingFileAppender::new(Rotation::NEVER, dir_path, file_prefix);
        let (non_blocking, guard) = tracing_appender::non_blocking(rolling_log);
        guards.push(guard);
        fmt::Layer::default().with_writer(non_blocking.with_max_level(file_level))
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

struct MyFormatter;

impl<S, N> FormatEvent<S, N> for MyFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        // Format values from the event's's metadata:
        let metadata = event.metadata();
        if let Some(value) = metadata.fields().field("component") {
            write!(&mut writer, "{} {}: ", metadata.level(), value.to_string())?;
        } else {
            write!(&mut writer, "{} {}: ", metadata.level(), metadata.target())?;
        }

        // Format all the spans in the event's span context.
        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                write!(writer, "{}", span.name())?;

                // `FormattedFields` is a formatted representation of the span's
                // fields, which is stored in its extensions by the `fmt` layer's
                // `new_span` method. The fields will have been formatted
                // by the same field formatter that's provided to the event
                // formatter in the `FmtContext`.
                let ext = span.extensions();
                let fields = &ext
                    .get::<FormattedFields<N>>()
                    .expect("will never be `None`");

                // Skip formatting the fields if the span had no fields.
                if !fields.is_empty() {
                    write!(writer, "{{{}}}", fields)?;
                }
                write!(writer, ": ")?;
            }
        }

        // Write fields on the event
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}
