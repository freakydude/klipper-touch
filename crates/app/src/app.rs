//! Assembling the application: settings, service, window and bridges.

use crate::actions;
use crate::network_controller::NetworkController;
use crate::presenter::Presenter;
use crate::settings_controller::SettingsController;
use crate::AppWindow;
use klipper_touch_core::network::{NetworkBackend, NetworkService};
use klipper_touch_core::settings::{SettingsError, SettingsStore};
use klipper_touch_core::{Endpoint, PrinterService};
use slint::winit_030::winit::event::WindowEvent;
use slint::winit_030::{EventResult, WinitWindowAccessor};
use slint::{ComponentHandle, Timer, TimerMode};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Start options, typically from the command line.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Overrides the saved Moonraker address for this start.
    pub moonraker: Option<String>,
    pub fullscreen: bool,
    pub settings_file: Option<PathBuf>,
    /// NetworkManager, or simulated networks in development builds.
    pub network: NetworkBackend,
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

/// Running application; keeps runtime and bridges alive as long as the window exists.
pub struct Application {
    pub window: AppWindow,
    pub presenter: std::rc::Rc<Presenter>,
    pub service: PrinterService,
    _settings: std::rc::Rc<SettingsController>,
    _network: std::rc::Rc<NetworkController>,
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
        let (network_service, network_channels) = NetworkService::start(runtime.handle(), options.network);
        let network = NetworkController::bind(
            &window,
            network_service,
            service.clone(),
            presenter.clone(),
            &endpoint,
            options.network.is_simulated(),
        );
        let settings = SettingsController::bind(
            &window,
            store,
            settings,
            &endpoint,
            service.clone(),
            presenter.clone(),
            network.clone(),
        );
        actions::bind(&window, &service, &presenter, &settings);
        presenter.follow(channels)?;
        network.follow(network_channels)?;

        let clock = Timer::default();
        let clock_presenter = presenter.clone();
        clock.start(TimerMode::Repeated, Duration::from_secs(1), move || clock_presenter.refresh_job_texts());
        presenter.refresh_job_texts();

        if options.fullscreen || settings.fullscreen() {
            window.window().set_fullscreen(true);
        }
        follow_fullscreen_pointer(&window);
        Ok(Self {
            window,
            presenter,
            service,
            _settings: settings,
            _network: network,
            _clock: clock,
            _runtime: runtime,
        })
    }

    pub fn run(self) -> Result<(), AppError> {
        self.window.run()?;
        Ok(())
    }
}

/// Fullscreen is the touch-only kiosk mode: a pointer there would only be the cursor of a touch screen.
/// The winit window exists once the event loop runs, so the call waits for the first timer tick.
/// The pointer belongs to windowed use only. winit may show it again on focus, resize or pointer entry, and the
/// window is not yet created when the app starts, so the state is re-applied on those events instead of once.
pub fn follow_fullscreen_pointer(window: &AppWindow) {
    window.window().on_winit_window_event(|window, event| {
        if matches!(event, WindowEvent::Focused(_) | WindowEvent::Resized(_) | WindowEvent::CursorEntered { .. }) {
            set_pointer_visible(window, !window.is_fullscreen());
        }
        EventResult::Propagate
    });
}

/// Shows or hides the pointer; leaving fullscreen brings it back.
pub fn set_pointer_visible(window: &slint::Window, visible: bool) {
    let _ = window.with_winit_window(|winit_window| winit_window.set_cursor_visible(visible));
}
