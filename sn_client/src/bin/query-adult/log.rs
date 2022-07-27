use tracing_subscriber::{fmt, prelude::*, EnvFilter, Layer, Registry};

pub fn init() {
    let fmt = fmt::layer()
        .with_ansi(false)
        .with_filter(EnvFilter::from_default_env());

    Registry::default().with(fmt).init();
}
