//! klipper-touch: touch UI for Klipper via Moonraker.
//!
//! Structure: `ui/*.slint` describes the UI, [`presenter`] writes the state from
//! `klipper-touch-core` into it, [`actions`] forwards user input as commands.

slint::include_modules!();

pub mod actions;
pub mod app;
pub mod format;
pub mod network_controller;
pub mod presenter;
pub mod settings_controller;
pub mod status_log;
pub mod thumbnail;
pub mod view;
