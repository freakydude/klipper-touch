//! freakymate: touch UI for Klipper via Moonraker.
//!
//! Structure: `ui/*.slint` describes the UI, [`presenter`] writes the state from
//! `freakymate-core` into it, [`actions`] forwards user input as commands.

slint::include_modules!();

pub mod actions;
pub mod app;
mod bed_map;
mod calibration_view;
mod chart;
pub mod display;
pub mod format;
mod history_view;
mod items;
pub mod network_controller;
mod objects_view;
pub mod presenter;
pub mod settings_controller;
pub mod status_log;
pub mod thumbnail;
pub mod view;
