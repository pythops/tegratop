use std::fs::{remove_file, OpenOptions};
use std::{env, path::Path};

use anyhow::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, Layer};

use libc::getuid;
pub struct Tracing;
use std::os::unix::fs::chown;

impl Tracing {
    pub fn init() -> Result<()> {
        let log_file_path = Path::new("/tmp/tegratop.log");

        if log_file_path.exists() {
            remove_file(log_file_path)?;
        }

        let log_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(log_file_path)?;

        let uid = match env::var("SUDO_UID") {
            Ok(uid) => uid.parse::<u32>()?,
            Err(_) => unsafe { getuid() },
        };

        chown(log_file_path, Some(uid), Some(uid))?;

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
