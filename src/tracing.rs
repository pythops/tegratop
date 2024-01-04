use anyhow::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, Layer};

pub struct Tracing;

impl Tracing {
    pub fn init() -> Result<()> {
        let log_file = std::fs::File::create("/tmp/tegratop.log")?;
        let file_subscriber = tracing_subscriber::fmt::layer()
            .with_file(true)
            .with_line_number(true)
            .with_writer(log_file)
            .with_target(false)
            .with_ansi(false)
            .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());

        tracing_subscriber::registry()
            .with(file_subscriber)
            .with(ErrorLayer::default())
            .init();
        Ok(())
    }
}
