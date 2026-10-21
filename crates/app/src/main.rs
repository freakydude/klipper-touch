// Do not open a console window in Windows release builds.
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use clap::Parser;
use klipper_touch::app::{Application, Options};
use std::path::PathBuf;
use std::process::ExitCode;

/// Touch screen UI for Klipper printers via Moonraker.
#[derive(Debug, Parser)]
#[command(version = klipper_touch_core::VERSION, about)]
struct Cli {
    /// Moonraker address for this start, e.g. http://printer.local:7125 (default: saved setting)
    #[arg(short, long)]
    moonraker: Option<String>,
    /// Start in fullscreen (kiosk) mode
    #[arg(short, long)]
    fullscreen: bool,
    /// Settings file (default: <config dir>/klipper-touch/settings.json)
    #[arg(long)]
    settings: Option<PathBuf>,
}

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();
    let options = Options { moonraker: cli.moonraker, fullscreen: cli.fullscreen, settings_file: cli.settings };
    match Application::build(options).and_then(Application::run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log::error!("{error}");
            ExitCode::FAILURE
        }
    }
}
