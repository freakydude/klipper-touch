//! Shared state of all connections: printer, host, file catalog and runtime.

use crate::files::Catalog;
use crate::host::Host;
use crate::machine::MachineProfile;
use crate::printer::Printer;
use crate::spoolman::Spoolman;
use serde_json::{json, Value};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;

const NOTIFICATION_BUFFER: usize = 64;

pub struct Backend {
    printer: Mutex<Printer>,
    host: Mutex<Host>,
    spoolman: Mutex<Spoolman>,
    spoolman_enabled: bool,
    catalog: Catalog,
    started: Instant,
    notifications: broadcast::Sender<String>,
}

impl Backend {
    pub fn new(time_scale: f64) -> Self {
        Self::with_profile(time_scale, MachineProfile::default())
    }

    pub fn with_profile(time_scale: f64, profile: MachineProfile) -> Self {
        Self {
            printer: Mutex::new(Printer::with_profile(time_scale, profile)),
            host: Mutex::new(Host::default()),
            spoolman: Mutex::new(Spoolman::default()),
            spoolman_enabled: profile.spoolman,
            catalog: Catalog::demo(),
            started: Instant::now(),
            notifications: broadcast::channel(NOTIFICATION_BUFFER).0,
        }
    }

    /// Receiver for notifications to all connections (G-code responses, Klippy state ...).
    pub fn notifications(&self) -> broadcast::Receiver<String> {
        self.notifications.subscribe()
    }

    pub fn notify(&self, method: &str, params: Value) {
        let message = json!({ "jsonrpc": "2.0", "method": method, "params": params });
        let _ = self.notifications.send(message.to_string());
    }

    /// Reports a Klipper response line like Moonraker: in the history and as `notify_gcode_response`.
    pub fn respond(&self, message: &str) {
        self.with_host(|host| host.record("response", message, unix_time()));
        self.notify("notify_gcode_response", json!([message]));
    }

    pub fn record_command(&self, script: &str) {
        self.with_host(|host| host.record("command", script, unix_time()));
    }

    pub fn with_host<R>(&self, action: impl FnOnce(&mut Host) -> R) -> R {
        let mut host = self.host.lock().expect("host lock poisoned");
        action(&mut host)
    }

    pub fn spoolman_enabled(&self) -> bool {
        self.spoolman_enabled
    }

    pub fn with_spoolman<R>(&self, action: impl FnOnce(&mut Spoolman) -> R) -> R {
        let mut spoolman = self.spoolman.lock().expect("spoolman lock poisoned");
        action(&mut spoolman)
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub fn status(&self) -> Value {
        self.with_printer(|printer| printer.status())
    }

    pub fn object_names(&self) -> Vec<String> {
        self.with_printer(|printer| printer.object_names())
    }

    pub fn advance_simulation(&self, elapsed: Duration) {
        self.with_printer(|printer| printer.advance(elapsed));
    }

    pub fn eventtime(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }

    pub fn with_printer<R>(&self, action: impl FnOnce(&mut Printer) -> R) -> R {
        let mut printer = self.printer.lock().expect("printer lock poisoned");
        action(&mut printer)
    }
}

fn unix_time() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0.0, |elapsed| elapsed.as_secs_f64())
}
