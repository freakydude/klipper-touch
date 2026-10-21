//! Gemeinsamer Zustand aller Verbindungen: Drucker, Dateikatalog und Laufzeit.

use crate::files::Catalog;
use crate::machine::MachineProfile;
use crate::printer::Printer;
use serde_json::Value;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct Backend {
    printer: Mutex<Printer>,
    catalog: Catalog,
    started: Instant,
}

impl Backend {
    pub fn new(time_scale: f64) -> Self {
        Self::with_profile(time_scale, MachineProfile::default())
    }

    pub fn with_profile(time_scale: f64, profile: MachineProfile) -> Self {
        Self {
            printer: Mutex::new(Printer::with_profile(time_scale, profile)),
            catalog: Catalog::demo(),
            started: Instant::now(),
        }
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
