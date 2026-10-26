//! Headless application session: the real UI bindings on a software-rendered window, driven step by step.

use crate::Options;
use klipper_touch::network_controller::NetworkController;
use klipper_touch::presenter::Presenter;
use klipper_touch::settings_controller::SettingsController;
use klipper_touch::{actions, AppWindow, Navigation, Page};
use klipper_touch_core::network::{NetworkBackend, NetworkChannels, NetworkService};
use klipper_touch_core::settings::{Settings, SettingsStore};
use klipper_touch_core::{Endpoint, PrinterService, ServiceChannels, Snapshot};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, WindowAdapter};
use slint::{ComponentHandle, PhysicalSize, Rgb8Pixel};
use std::rc::Rc;
use std::time::{Duration, Instant};

const WAIT_LIMIT: Duration = Duration::from_secs(20);
/// Time for Slint and the services to settle before a picture is taken.
const SETTLE_TIME: Duration = Duration::from_millis(400);

struct SnapshotPlatform {
    window: Rc<MinimalSoftwareWindow>,
    start: Instant,
}

impl Platform for SnapshotPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        self.start.elapsed()
    }
}

pub struct Session {
    pub window: AppWindow,
    pub presenter: Rc<Presenter>,
    pub service: PrinterService,
    pub network: Rc<NetworkController>,
    pub network_service: NetworkService,
    platform_window: Rc<MinimalSoftwareWindow>,
    channels: ServiceChannels,
    network_channels: NetworkChannels,
    options: Options,
    count: usize,
    _runtime: tokio::runtime::Runtime,
}

impl Session {
    /// Builds the window with the same bridges as the app, connected to Moonraker and simulated networks.
    pub fn start(options: Options) -> Result<Self, String> {
        std::fs::create_dir_all(&options.out).map_err(|error| error.to_string())?;
        let platform_window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        slint::platform::set_platform(Box::new(SnapshotPlatform {
            window: platform_window.clone(),
            start: Instant::now(),
        }))
        .map_err(|error| format!("{error:?}"))?;

        let endpoint = Endpoint::parse(&options.moonraker).map_err(|error| error.to_string())?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        let window = AppWindow::new().map_err(|error| error.to_string())?;
        window.window().set_size(PhysicalSize::new(options.width, options.height));
        window.show().map_err(|error| error.to_string())?;
        if options.fit {
            klipper_touch::display::apply_fit(window.window());
        }

        let (service, channels) = PrinterService::start(runtime.handle(), endpoint.clone());
        let presenter = Presenter::new(&window, service.clone());
        let (network_service, network_channels) = NetworkService::start(runtime.handle(), NetworkBackend::Simulated);
        let network = NetworkController::bind(
            &window,
            network_service.clone(),
            service.clone(),
            presenter.clone(),
            &endpoint,
            true,
        );
        let store = SettingsStore::new(options.out.join("snapshot-settings.json"));
        let settings = SettingsController::bind(
            &window,
            store,
            Settings::default(),
            &endpoint,
            service.clone(),
            presenter.clone(),
            network.clone(),
        );
        actions::bind(&window, &service, &presenter, &settings);

        Ok(Self {
            window,
            presenter,
            service,
            network,
            network_service,
            platform_window,
            channels,
            network_channels,
            options,
            count: 0,
            _runtime: runtime,
        })
    }

    /// Transfers pending snapshots and events, as the event loop otherwise does.
    pub fn pump(&mut self, duration: Duration) {
        let deadline = Instant::now() + duration;
        loop {
            if self.channels.snapshots.has_changed().unwrap_or(false) {
                let snapshot = self.channels.snapshots.borrow_and_update().clone();
                self.presenter.apply_snapshot(&snapshot);
            }
            while let Ok(event) = self.channels.events.try_recv() {
                self.presenter.handle_event(event);
            }
            if self.network_channels.states.has_changed().unwrap_or(false) {
                let state = self.network_channels.states.borrow_and_update().clone();
                self.network.apply_state(state);
            }
            while let Ok(notice) = self.network_channels.notices.try_recv() {
                self.network.show_notice(&notice);
            }
            slint::platform::update_timers_and_animations();
            if Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    pub fn wait_until(&mut self, what: &str, condition: impl Fn(&Snapshot) -> bool) -> Result<(), String> {
        self.wait_until_for(what, WAIT_LIMIT, condition)
    }

    pub fn wait_until_for(
        &mut self,
        what: &str,
        limit: Duration,
        condition: impl Fn(&Snapshot) -> bool,
    ) -> Result<(), String> {
        let deadline = Instant::now() + limit;
        while !condition(&self.presenter.snapshot()) {
            if Instant::now() > deadline {
                return Err(format!("timed out waiting for {what}"));
            }
            self.pump(Duration::from_millis(50));
        }
        Ok(())
    }

    /// Renders the window into `<out>/<number>-<name>.png` and prints the path.
    pub fn capture(&mut self, name: &str) -> Result<(), String> {
        self.pump(SETTLE_TIME);
        let (width, height) = (self.options.width as usize, self.options.height as usize);
        let mut buffer = vec![Rgb8Pixel::default(); width * height];
        self.platform_window.request_redraw();
        self.platform_window.draw_if_needed(|renderer| {
            renderer.render(&mut buffer, width);
        });
        let bytes: Vec<u8> = buffer.iter().flat_map(|pixel| [pixel.r, pixel.g, pixel.b]).collect();
        self.count += 1;
        let path = self.options.out.join(format!("{:02}-{name}.png", self.count));
        image::RgbImage::from_raw(self.options.width, self.options.height, bytes)
            .ok_or("buffer size mismatch")?
            .save(&path)
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
        println!("{}", path.display());
        Ok(())
    }

    pub fn show_page(&mut self, page: Page, name: &str) -> Result<(), String> {
        self.window.global::<Navigation>().set_page(page);
        self.capture(name)
    }
}
