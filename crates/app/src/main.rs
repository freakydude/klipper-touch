// Do not open a console window in Windows release builds.
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use clap::Parser;
use freakymate::app::{Application, Options};
use freakymate_core::network::NetworkBackend;
use std::path::PathBuf;
use std::process::ExitCode;

/// Touch screen UI for Klipper printers via Moonraker.
#[derive(Debug, Parser)]
#[command(version = freakymate_core::VERSION, about)]
struct Cli {
    /// Moonraker address for this start, e.g. http://printer.local:7125 (default: saved setting)
    #[arg(short, long)]
    moonraker: Option<String>,
    /// Start in fullscreen (kiosk) mode
    #[arg(short, long)]
    fullscreen: bool,
    /// Settings file (default: <config dir>/freakymate/settings.json; the installer uses printer_data/config)
    #[arg(long)]
    settings: Option<PathBuf>,
    /// Simulated Wi-Fi and Ethernet instead of NetworkManager (development build only)
    #[cfg(feature = "dev-mode")]
    #[arg(long)]
    simulate_network: bool,
}

impl Cli {
    fn network_backend(&self) -> NetworkBackend {
        #[cfg(feature = "dev-mode")]
        if self.simulate_network {
            return NetworkBackend::Simulated;
        }
        NetworkBackend::System
    }
}

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();
    let options = Options {
        network: cli.network_backend(),
        moonraker: cli.moonraker,
        fullscreen: cli.fullscreen,
        settings_file: cli.settings,
    };
    match Application::build(options).and_then(Application::run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log::error!("{error}");
            ExitCode::FAILURE
        }
    }
}
