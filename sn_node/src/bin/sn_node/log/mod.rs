mod appender;

use sn_interface::LogFormatter;
use sn_node::node::Config;

use eyre::Result;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::{EnvFilter, Targets};
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::Filter;
use tracing_subscriber::{prelude::*, Registry};

#[cfg(feature = "otlp")]
macro_rules! otlp_layer {
    () => {{
        use opentelemetry::sdk::Resource;
        use opentelemetry::KeyValue;
        use opentelemetry_otlp::WithExportConfig;

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    // Derive endpoints etc. from environment variables like `OTEL_EXPORTER_OTLP_ENDPOINT`
                    .with_env(),
            )
            .with_trace_config(
                opentelemetry::sdk::trace::config().with_resource(Resource::new(vec![
                    KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                        current_crate_str(),
                    ),
                    KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_INSTANCE_ID,
                        std::process::id().to_string(),
                    ),
                ])),
            )
            .install_batch(opentelemetry::runtime::Tokio);

        match tracer {
            Ok(t) => Ok(tracing_opentelemetry::layer().with_tracer(t).with_filter(EnvFilter::from_env("RUST_LOG_OTLP"))),
            Err(e) => Err(e),
        }
    }};
}

macro_rules! fmt_layer {
    ($config:expr) => {{
        // Filter by log level either from `RUST_LOG` or default to crate only.
        let target_filter: Box<dyn Filter<Registry> + Send + Sync> =
            if let Ok(f) = EnvFilter::try_from_default_env() {
                Box::new(f)
            } else {
                Box::new(Targets::new().with_target(current_crate_str(), $config.verbose()))
            };
        let mut guard: Option<WorkerGuard> = None;
        let fmt_layer: Layer<Registry> = tracing_subscriber::fmt::layer()
            .with_thread_names(true)
            .with_ansi(false);

        let fmt_layer = if let Some(log_dir) = $config.log_dir() {
            println!("Starting logging to directory: {:?}", log_dir);

            let (non_blocking, worker_guard) = appender::file_rotater(
                log_dir,
                $config.logs_max_bytes,
                $config.logs_max_lines,
                $config.logs_retained,
                $config.logs_uncompressed,
            );
            guard = Some(worker_guard);

            let fmt_layer = fmt_layer.with_writer(non_blocking);

            if $config.json_logs {
                fmt_layer.json().with_filter(target_filter).boxed()
            } else {
                fmt_layer
                    .event_format(LogFormatter::default())
                    .with_filter(target_filter)
                    .boxed()
            }
        } else {
            println!("Starting logging to stdout");

            fmt_layer
                .with_target(false)
                .event_format(LogFormatter::default())
                .with_filter(target_filter)
                .boxed()
        };

        (fmt_layer, guard)
    }};
}

/// Inits node logging, returning the global node guard if required.
/// This guard should be held for the life of the program.
///
/// Logging should be instantiated only once.
pub fn init_node_logging(config: &Config) -> Result<Option<WorkerGuard>> {
    let reg = tracing_subscriber::registry();

    let (fmt, guard) = fmt_layer!(config);
    let reg = reg.with(fmt);

    #[cfg(feature = "tokio-console")]
    let reg = reg.with(console_subscriber::spawn());

    #[cfg(feature = "otlp")]
    let reg = reg.with(otlp_layer!()?);

    reg.init();

    Ok(guard)
}

/// Get current root module name (e.g. "sn_node")
fn current_crate_str() -> &'static str {
    // Grab root from module path ("sn_node::log::etc" -> "sn_node")
    let m = module_path!();
    &m[..m.find(':').unwrap_or(m.len())]
}
