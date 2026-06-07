mod analyze;
mod app;
mod cleaner;
mod cleaners;
mod cli;
mod doctor;
mod findings;
mod report;
mod rules;
mod safety;
mod scanner;
mod status;
mod theme;
mod tui;
mod uninstall;

use std::fs::{self, File};
use tracing_subscriber::fmt;
use tracing_subscriber::{Registry, prelude::*};

fn init_logging() {
    let Some(base_dirs) = directories::BaseDirs::new() else {
        return;
    };

    let log_dir = base_dirs.cache_dir().join("nibble");
    if fs::create_dir_all(&log_dir).is_err() {
        return;
    }

    let Ok(log_file) = File::create(log_dir.join("nibble.log")) else {
        return;
    };

    let file_layer = fmt::layer().with_writer(log_file).with_ansi(false);
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(tracing::Level::INFO.into());

    let _ = Registry::default().with(filter).with(file_layer).try_init();
}

fn main() {
    // Initialize file-based logging
    init_logging();

    tracing::info!("Starting Nibble...");

    // Execute app runner
    if let Err(e) = app::run_app() {
        eprintln!("Error: {:?}", e);
        tracing::error!("App execution failed: {:?}", e);
        std::process::exit(1);
    }

    tracing::info!("Nibble completed successfully.");
}
