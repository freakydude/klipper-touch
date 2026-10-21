//! klipper-touch-slint: Touch-Oberfläche für Klipper über Moonraker.
//!
//! Aufbau: `ui/*.slint` beschreibt die Oberfläche, [`presenter`] schreibt den Zustand aus
//! `klipper-touch-core` hinein, [`actions`] leitet Bedienungen als Befehle weiter.

slint::include_modules!();

pub mod actions;
pub mod app;
pub mod format;
pub mod presenter;
pub mod settings_controller;
pub mod thumbnail;
pub mod view;
