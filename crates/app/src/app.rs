//! Assembling the application: settings, service, window and bridges.

use crate::actions;
use crate::display;
use crate::network_controller::NetworkController;
use crate::presenter::Presenter;
use crate::settings_controller::SettingsController;
use crate::AppWindow;
use freakymate_core::network::{NetworkBackend, NetworkService};
use freakymate_core::settings::{SettingsError, SettingsStore};
use freakymate_core::{Endpoint, PrinterService};
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
    Endpoint(#[from] freakymate_core::moonraker::InvalidEndpoint),
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
        let settings = store.load_or_create().unwrap_or_else(|error| {
            log::warn!("{error}; using defaults");
            freakymate_core::Settings::default()
        });
        let endpoint = Endpoint::parse(options.moonraker.as_deref().unwrap_or(&settings.moonraker))?;
        log::info!("Moonraker at {}", endpoint.display());

        let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build()?;
        let window = AppWindow::new()?;
        // Wayland and X11 desktops find the icon through the app id and `freakymate.desktop`, not the window icon.
        if let Err(error) = slint::set_xdg_app_id("freakymate") {
            log::warn!("cannot set the app id: {error}");
        }
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
        clock.start(TimerMode::Repeated, Duration::from_secs(1), move || clock_presenter.tick());
        presenter.refresh_job_texts();

        // Started as a kiosk, the window never gets a frame: a kiosk compositor such as cage draws none, but winit
        // still keeps the height of its own title bar free when leaving fullscreen (800x445 instead of 800x480).
        window.set_kiosk(options.fullscreen);
        if options.fullscreen || settings.fullscreen() {
            window.window().set_fullscreen(true);
        }
        watch_window_events(&window);
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

/// The one filter for winit's window events; Slint keeps only one per window, so a second registration would
/// silently replace the first.
///
/// - Pointer: fullscreen is the touch-only kiosk mode, where a pointer would only be the cursor of a touch screen.
///   winit may show it again on focus, resize or pointer entry, and the window does not exist yet when the app
///   starts, so the state is re-applied on those events instead of once.
/// - Scale: in fullscreen the layout is fitted to the real display whenever the size changes (see [`display`]); the
///   scale of the system is not used there, a compositor reports 1 for a touch screen. A window keeps the scale of
///   the system, so a larger window shows more room. Whoever sets `SLINT_SCALE_FACTOR` chooses the scale by hand,
///   and nothing is fitted then.
pub fn watch_window_events(window: &AppWindow) {
    let fit = std::env::var_os("SLINT_SCALE_FACTOR").is_none();
    let weak = window.as_weak();
    window.window().on_winit_window_event(move |slint_window, event| {
        if matches!(event, WindowEvent::Focused(_) | WindowEvent::Resized(_) | WindowEvent::CursorEntered { .. }) {
            set_pointer_visible(slint_window, !slint_window.is_fullscreen());
        }
        match event {
            WindowEvent::Resized(_) if fit => {
                // Applied after the event, not inside it: Slint is still handling it.
                let weak = weak.clone();
                Timer::single_shot(Duration::ZERO, move || {
                    if let Some(window) = weak.upgrade() {
                        scale_to_mode(window.window());
                    }
                });
                EventResult::Propagate
            }
            WindowEvent::ScaleFactorChanged { .. } if fit => EventResult::PreventDefault,
            _ => EventResult::Propagate,
        }
    });
}

/// Fullscreen fits the layout to the display; a window shows it at the scale of the system (see
/// [`display::fits_to_display`]).
fn scale_to_mode(window: &slint::Window) {
    let system_scale = window.with_winit_window(|winit_window| winit_window.scale_factor() as f32).unwrap_or(1.0);
    let size = window.size();
    if display::fits_to_display(window.is_fullscreen(), size.width, size.height, system_scale) {
        display::apply_fit(window);
    } else {
        display::apply_system_scale(window, system_scale);
    }
}

/// Shows or hides the pointer; leaving fullscreen brings it back.
pub fn set_pointer_visible(window: &slint::Window, visible: bool) {
    let _ = window.with_winit_window(|winit_window| winit_window.set_cursor_visible(visible));
}
