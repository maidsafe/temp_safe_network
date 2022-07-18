mod appender;

use sn_interface::LogFormatter;
use sn_node::node::Config;

use eyre::Result;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::{EnvFilter, Targets};
use tracing_subscriber::layer::Layer;
use tracing_subscriber::prelude::*;

/// Inits node logging, returning the global node guard if required.
/// This guard should be held for the life of the program.
///
/// Logging should be instantiated only once.
pub fn init_node_logging(config: &Config) -> Result<Option<WorkerGuard>> {
    let mut layers = vec![];
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_ansi(false);

    #[cfg(feature = "tokio-console")]
    {
        let console_layer = console_subscriber::spawn();
        layers.push(console_layer.boxed());
    }

    let mut guard: Option<WorkerGuard> = None;
    if let Some(log_dir) = config.log_dir() {
        println!("Starting logging to directory: {:?}", log_dir);

        let (non_blocking, worker_guard) = appender::file_rotater(
            log_dir,
            config.logs_max_bytes,
            config.logs_max_lines,
            config.logs_retained,
            config.logs_uncompressed,
        );
        guard = Some(worker_guard);

        let fmt_layer = fmt_layer.with_writer(non_blocking);

        if config.json_logs {
            layers.push(fmt_layer.json().boxed());
        } else {
            layers.push(fmt_layer.event_format(LogFormatter::default()).boxed());
        }
    } else {
        println!("Starting logging to stdout");

        let fmt_layer = fmt_layer
            .with_target(false)
            .event_format(LogFormatter::default());
        layers.push(fmt_layer.boxed());
    };

    // Create filter to log only from certain modules. Either from `RUST_LOG` or a default level for current crate.
    let target_filter = if let Ok(f) = EnvFilter::try_from_default_env() {
        f.boxed()
    } else {
        Targets::new()
            .with_target(current_crate_str(), Level::INFO)
            .boxed()
    };

    tracing_subscriber::registry()
        .with(layers)
        .with(target_filter)
        .init();

    Ok(guard)
}

/// Get current root module name (e.g. "sn_node")
fn current_crate_str() -> &'static str {
    // Grab root from module path ("sn_node::log::etc" -> "sn_node")
    let m = module_path!();
    &m[..m.find(':').unwrap_or(m.len())]
}
