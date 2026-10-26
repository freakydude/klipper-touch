//! UI-independent core of klipper-touch: Moonraker connection, printer state, commands and settings.
//!
//! The entry point is [`PrinterService`]; the UI reads [`Snapshot`]s and [`Event`]s and sends [`Command`]s.

pub mod commands;
pub mod estimates;
pub mod files;
pub mod moonraker;
pub mod network;
pub mod printer;
pub mod service;
pub mod settings;
pub mod sourced;
pub mod spoolman;
pub mod system;

pub use commands::Command;
pub use moonraker::Endpoint;
pub use service::{ConnectionState, Event, PrinterService, Query, ServiceChannels, Snapshot};
pub use settings::{Settings, SettingsStore};
pub use sourced::{Provenance, Sourced};

/// Version of this build: set by CI from git tags, otherwise the crate version marked as a local build.
pub const VERSION: &str = match option_env!("KLIPPER_TOUCH_VERSION") {
    Some(version) => version,
    None => concat!(env!("CARGO_PKG_VERSION"), "-dev"),
};
