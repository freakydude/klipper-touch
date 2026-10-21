//! UI-independent core of klipper-touch: Moonraker connection, printer state, commands and settings.
//!
//! The entry point is [`PrinterService`]; the UI reads [`Snapshot`]s and [`Event`]s and sends [`Command`]s.

pub mod commands;
pub mod files;
pub mod moonraker;
pub mod printer;
pub mod service;
pub mod settings;
pub mod system;

pub use commands::Command;
pub use moonraker::Endpoint;
pub use service::{ConnectionState, Event, PrinterService, Query, ServiceChannels, Snapshot};
pub use settings::{Settings, SettingsStore};
