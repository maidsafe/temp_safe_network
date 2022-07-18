mod appender;

use sn_interface::LogFormatter;
use sn_node::node::Config;

use eyre::{Context, Result};
use file_rotate::{
    compression::Compression,
    suffix::{AppendTimestamp, FileLimit},
    ContentLimit,
};

pub use appender::FileRotateAppender;


#[cfg(not(feature = "tokio-console"))]
use tracing_appender::non_blocking::WorkerGuard;

#[cfg(not(feature = "tokio-console"))]
use tracing_subscriber::filter::EnvFilter;

#[cfg(not(feature = "tokio-console"))]
const MODULE_NAME: &str = "sn_node";

/// Inits node logging, returning the global node guard if required.
/// This guard should be held for the life of the program.
///
/// Logging should be instantiated only once.
#[cfg(not(feature = "tokio-console"))]
pub fn init_node_logging(config: Config) -> Result<Option<WorkerGuard>> {
    // ==============
    // Set up logging
    // ==============

    let mut _optional_guard: Option<WorkerGuard> = None;

    let filter = match EnvFilter::try_from_env("RUST_LOG") {
        Ok(filter) => filter,
        // If we have an error (ie RUST_LOG not set or otherwise), we check the verbosity flags
        Err(_) => {
            // we manually determine level filter instead of using tracing EnvFilter.
            let level_filter = config.verbose();
            let module_filter = format!("{}={}", MODULE_NAME, level_filter)
                .parse()
                .wrap_err("BUG: invalid module filter constructed")?;
            EnvFilter::from_default_env().add_directive(module_filter)
        }
    };

    _optional_guard = if let Some(log_dir) = config.log_dir() {
        println!("Starting logging to directory: {:?}", log_dir);

        let mut content_limit = ContentLimit::BytesSurpassed(config.logs_max_bytes);
        if config.logs_max_lines > 0 {
            content_limit = ContentLimit::Lines(config.logs_max_lines);
        }

        // FileRotate crate changed `0 means for all` to `0 means only original`
        // Here set the retained value to be same as uncompressed in case of 0.
        let logs_retained = if config.logs_retained == 0 {
            config.logs_uncompressed
        } else {
            config.logs_retained
        };
        let file_appender = FileRotateAppender::make_rotate_appender(
            log_dir,
            "sn_node.log",
            AppendTimestamp::default(FileLimit::MaxFiles(logs_retained)),
            content_limit,
            Compression::OnRotate(config.logs_uncompressed),
        );

        // configure how tracing non-blocking works: https://tracing.rs/tracing_appender/non_blocking/struct.nonblockingbuilder#method.default
        let non_blocking_builder = tracing_appender::non_blocking::NonBlockingBuilder::default();

        let (non_blocking, guard) = non_blocking_builder
            // lose lines and keep perf, or exert backpressure?
            .lossy(false)
            // optionally change buffered lines limit
            // .buffered_lines_limit(buffered_lines_limit)
            .finish(file_appender);

        let builder = tracing_subscriber::fmt()
            // eg : RUST_LOG=my_crate=info,my_crate::my_mod=debug,[my_span]=trace
            .with_env_filter(filter)
            .with_thread_names(true)
            .with_ansi(false)
            .with_writer(non_blocking);

        if config.json_logs {
            builder.json().init();
        } else {
            builder.event_format(LogFormatter::default()).init();
        }

        Some(guard)
    } else {
        println!("Starting logging to stdout");

        tracing_subscriber::fmt()
            .with_thread_names(true)
            .with_ansi(false)
            .with_env_filter(filter)
            .with_target(false)
            .event_format(LogFormatter::default())
            .init();

        None
    };

    Ok(_optional_guard)
}
