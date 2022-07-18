mod appender;

use sn_interface::LogFormatter;
use sn_node::node::Config;

use eyre::{Context, Result};

use tracing::Level;
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
    let env_filter = env_filter(config.verbose())?;
    let mut guard: Option<WorkerGuard> = None;

    guard = if let Some(log_dir) = config.log_dir() {
        println!("Starting logging to directory: {:?}", log_dir);

        let (non_blocking, guard) = appender::file_rotater(
            log_dir,
            config.logs_max_bytes,
            config.logs_max_lines,
            config.logs_retained,
            config.logs_uncompressed,
        );

        let builder = tracing_subscriber::fmt()
            // eg : RUST_LOG=my_crate=info,my_crate::my_mod=debug,[my_span]=trace
            .with_env_filter(env_filter)
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
            .with_env_filter(env_filter)
            .with_target(false)
            .event_format(LogFormatter::default())
            .init();

        None
    };

    Ok(guard)
}

fn env_filter(level_filter: Level) -> Result<EnvFilter> {
    let filter = match EnvFilter::try_from_env("RUST_LOG") {
        Ok(filter) => filter,
        // If we have an error (ie RUST_LOG not set or otherwise), we check the verbosity flags
        Err(_) => {
            // we manually determine level filter instead of using tracing EnvFilter.
            let module_filter = format!("{}={}", MODULE_NAME, level_filter)
                .parse()
                .wrap_err("BUG: invalid module filter constructed")?;
            EnvFilter::from_default_env().add_directive(module_filter)
        }
    };

    Ok(filter)
}
