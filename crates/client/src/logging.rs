use directories::ProjectDirs;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

const DEFAULT_FILTER: &str = "warn,scanfill=info,shared=info";
const KEEP_DAYS: usize = 14;

pub fn init() -> Option<WorkerGuard> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let terminal = fmt::layer().with_writer(std::io::stderr);

    let appender = ProjectDirs::from("dev", "AeonLogics", "ScanFill").and_then(|dirs| {
        RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_prefix("scanfill")
            .filename_suffix("log")
            .max_log_files(KEEP_DAYS)
            .build(dirs.data_local_dir().join("logs"))
            .ok()
    });

    let Some(appender) = appender else {
        tracing_subscriber::registry()
            .with(filter)
            .with(terminal)
            .init();
        tracing::warn!("log file unavailable, logging to the terminal only");
        return None;
    };

    let (writer, guard) = tracing_appender::non_blocking(appender);
    let file = fmt::layer().with_ansi(false).with_writer(writer);

    tracing_subscriber::registry()
        .with(filter)
        .with(terminal)
        .with(file)
        .init();

    Some(guard)
}
