//! Zusammensetzen der Anwendung: Einstellungen, Dienst, Fenster und Brücken.

use crate::actions;
use crate::presenter::Presenter;
use crate::settings_controller::SettingsController;
use crate::AppWindow;
use klipper_touch_core::settings::{SettingsError, SettingsStore};
use klipper_touch_core::{Endpoint, PrinterService};
use slint::{ComponentHandle, Timer, TimerMode};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Startoptionen, typischerweise von der Kommandozeile.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Überschreibt die gespeicherte Moonraker-Adresse für diesen Start.
    pub moonraker: Option<String>,
    pub fullscreen: bool,
    pub settings_file: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error(transparent)]
    Endpoint(#[from] klipper_touch_core::moonraker::InvalidEndpoint),
    #[error("cannot start background runtime: {0}")]
    Runtime(#[from] std::io::Error),
    #[error(transparent)]
    Platform(#[from] slint::PlatformError),
    #[error(transparent)]
    EventLoop(#[from] slint::EventLoopError),
}

/// Laufende Anwendung; hält Laufzeit und Brücken am Leben, solange das Fenster existiert.
pub struct Application {
    pub window: AppWindow,
    pub presenter: std::rc::Rc<Presenter>,
    pub service: PrinterService,
    _settings: std::rc::Rc<SettingsController>,
    _clock: Timer,
    _runtime: tokio::runtime::Runtime,
}

impl Application {
    pub fn build(options: Options) -> Result<Self, AppError> {
        let store = match options.settings_file {
            Some(path) => SettingsStore::new(path),
            None => SettingsStore::default_location()?,
        };
        let settings = store.load().unwrap_or_else(|error| {
            log::warn!("{error}; using defaults");
            klipper_touch_core::Settings::default()
        });
        let endpoint = Endpoint::parse(options.moonraker.as_deref().unwrap_or(&settings.moonraker))?;
        log::info!("Moonraker at {}", endpoint.display());

        let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build()?;
        let window = AppWindow::new()?;
        let (service, channels) = PrinterService::start(runtime.handle(), endpoint.clone());
        let presenter = Presenter::new(&window, service.clone());
        let settings =
            SettingsController::bind(&window, store, settings, &endpoint, service.clone(), presenter.clone());
        actions::bind(&window, &service, &presenter, &settings);
        presenter.follow(channels)?;

        let clock = Timer::default();
        let clock_presenter = presenter.clone();
        clock.start(TimerMode::Repeated, Duration::from_secs(1), move || clock_presenter.refresh_job_texts());
        presenter.refresh_job_texts();

        if options.fullscreen || settings.fullscreen() {
            window.window().set_fullscreen(true);
        }
        Ok(Self { window, presenter, service, _settings: settings, _clock: clock, _runtime: runtime })
    }

    pub fn run(self) -> Result<(), AppError> {
        self.window.run()?;
        Ok(())
    }
}
