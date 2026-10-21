//! Renders every UI page against a (fake) Moonraker as PNG, without a window and without a GPU.
//!
//! `cargo run -p klipper-touch --bin ui-snapshot -- --moonraker http://127.0.0.1:7125 --out <folder>`

use klipper_touch::presenter::Presenter;
use klipper_touch::settings_controller::SettingsController;
use klipper_touch::{actions, AppWindow, Confirm, Navigation, Page};
use klipper_touch_core::printer::{KlippyState, PrintState};
use klipper_touch_core::settings::{Settings, SettingsStore};
use klipper_touch_core::{Command, ConnectionState, Endpoint, PrinterService, Query, ServiceChannels, Snapshot};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, WindowAdapter};
use slint::{ComponentHandle, PhysicalSize, Rgb8Pixel};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

const WAIT_LIMIT: Duration = Duration::from_secs(20);

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

struct Options {
    moonraker: String,
    out: PathBuf,
    width: u32,
    height: u32,
}

fn parse_options() -> Result<Options, String> {
    let mut options = Options {
        moonraker: "http://127.0.0.1:7125".to_owned(),
        out: std::env::temp_dir().join("kt-ui-snapshots"),
        width: 800,
        height: 480,
    };
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let value = args.next().ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--moonraker" => options.moonraker = value,
            "--out" => options.out = PathBuf::from(value),
            "--size" => {
                let (width, height) = value.split_once('x').ok_or("size must be WIDTHxHEIGHT")?;
                options.width = width.parse().map_err(|_| "invalid width")?;
                options.height = height.parse().map_err(|_| "invalid height")?;
            }
            _ => return Err(format!("unknown option {flag}")),
        }
    }
    Ok(options)
}

struct Session {
    platform_window: Rc<MinimalSoftwareWindow>,
    window: AppWindow,
    presenter: Rc<Presenter>,
    service: PrinterService,
    channels: ServiceChannels,
    options: Options,
    count: usize,
}

impl Session {
    /// Transfers pending snapshots and events, as the event loop otherwise does.
    fn pump(&mut self, duration: Duration) {
        let deadline = Instant::now() + duration;
        loop {
            if self.channels.snapshots.has_changed().unwrap_or(false) {
                let snapshot = self.channels.snapshots.borrow_and_update().clone();
                self.presenter.apply_snapshot(&snapshot);
            }
            while let Ok(event) = self.channels.events.try_recv() {
                self.presenter.handle_event(event);
            }
            slint::platform::update_timers_and_animations();
            if Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn wait_until(&mut self, what: &str, condition: impl Fn(&Snapshot) -> bool) -> Result<(), String> {
        let deadline = Instant::now() + WAIT_LIMIT;
        while !condition(&self.presenter.snapshot()) {
            if Instant::now() > deadline {
                return Err(format!("timed out waiting for {what}"));
            }
            self.pump(Duration::from_millis(50));
        }
        Ok(())
    }

    fn capture(&mut self, name: &str) -> Result<(), String> {
        self.pump(Duration::from_millis(400));
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

    fn show_page(&mut self, page: Page, name: &str) -> Result<(), String> {
        self.window.global::<Navigation>().set_page(page);
        self.capture(name)
    }
}

fn run() -> Result<(), String> {
    let options = parse_options()?;
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

    let (service, channels) = PrinterService::start(runtime.handle(), endpoint.clone());
    let presenter = Presenter::new(&window, service.clone());
    let store = SettingsStore::new(temporary_settings(&options.out));
    let settings =
        SettingsController::bind(&window, store, Settings::default(), &endpoint, service.clone(), presenter.clone());
    actions::bind(&window, &service, &presenter, &settings);

    let mut session = Session { platform_window, window, presenter, service, channels, options, count: 0 };
    session.capture("connecting")?;
    session.wait_until("printer ready", |snapshot| {
        snapshot.connection == ConnectionState::Connected && snapshot.status.klippy_state == KlippyState::Ready
    })?;

    session.service.send(Command::Home { x: true, y: true, z: true });
    session.service.query(Query::PowerDevices);
    session.service.query(Query::Updates);
    session.service.query(Query::ConsoleHistory);
    session.service.send(Command::RunGcode { script: "STATUS".into() });
    session.wait_until("homing", |snapshot| snapshot.status.motion.homed.all())?;

    session.show_page(Page::Print, "print-standby")?;
    session.presenter.request_files();
    session.window.global::<Navigation>().set_files_open(true);
    session.pump(Duration::from_secs(2));
    session.capture("print-files")?;
    session.window.global::<Navigation>().set_files_open(false);
    for (page, name) in [
        (Page::Move, "move"),
        (Page::Temperature, "temperature"),
        (Page::Tune, "tune"),
        (Page::Extrude, "extrude"),
        (Page::Macros, "macros"),
        (Page::Console, "console"),
        (Page::System, "system"),
        (Page::Settings, "settings"),
    ] {
        session.show_page(page, name)?;
    }

    session.service.send(Command::StartPrint { filename: "benchy.gcode".into() });
    session.wait_until("print running", |snapshot| {
        snapshot.status.job.state == PrintState::Printing && snapshot.status.job.progress > 0.02
    })?;
    session.pump(Duration::from_secs(1));
    session.show_page(Page::Print, "print-printing")?;
    session.show_page(Page::Temperature, "temperature-printing")?;
    session.window.global::<Navigation>().set_page(Page::Print);
    session.window.global::<Confirm>().invoke_ask(
        "Cancel print".into(),
        "The current print will be stopped and cannot be resumed.".into(),
        "cancel-print".into(),
        "".into(),
        true,
    );
    session.capture("confirm-cancel")?;
    session.window.global::<Confirm>().invoke_execute();
    session.wait_until("print cancelled", |snapshot| snapshot.status.job.state == PrintState::Cancelled)?;
    session.capture("print-cancelled")?;

    session.service.send(Command::EmergencyStop);
    session.wait_until("shutdown", |snapshot| snapshot.status.klippy_state == KlippyState::Shutdown)?;
    session.capture("klippy-shutdown")?;
    session.service.send(Command::FirmwareRestart);
    session.wait_until("ready again", |snapshot| snapshot.status.klippy_state == KlippyState::Ready)?;
    drop(runtime);
    Ok(())
}

fn temporary_settings(out: &Path) -> PathBuf {
    out.join("snapshot-settings.json")
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ui-snapshot: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
