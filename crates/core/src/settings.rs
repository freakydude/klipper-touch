//! User settings as a JSON file in the user's configuration directory.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

const APP_FOLDER: &str = "klipper-touch";
const FILE_NAME: &str = "settings.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Address of the Moonraker server, e.g. `http://127.0.0.1:7125`.
    pub moonraker: String,
    pub fullscreen: bool,
    pub steps: Steps,
    pub temperature_presets: Vec<TemperaturePreset>,
    /// Macros the printer had when the touch screen last saw it; a macro missing here counts as new.
    pub known_macros: Vec<String>,
    /// Known macros the user hides from the console.
    pub hidden_macros: Vec<String>,
}

impl Settings {
    /// Brings the macro lists in line with the macros the printer currently has.
    /// Macros that vanished are forgotten (also as hidden), new ones are known and visible.
    /// Returns whether anything changed.
    pub fn reconcile_macros(&mut self, current: &[String]) -> bool {
        let before = (self.known_macros.clone(), self.hidden_macros.clone());
        self.known_macros.retain(|name| current.contains(name));
        self.hidden_macros.retain(|name| current.contains(name));
        for name in current {
            if !self.known_macros.contains(name) {
                self.known_macros.push(name.clone());
            }
        }
        before != (self.known_macros.clone(), self.hidden_macros.clone())
    }
}

/// Last chosen step sizes; the UI offers fixed steps and remembers the selection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Steps {
    pub move_mm: f64,
    pub z_offset_mm: f64,
    pub temperature: f64,
    pub extrude_mm: f64,
    pub extrude_speed: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemperaturePreset {
    pub name: String,
    pub extruder: f64,
    pub bed: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            moonraker: "http://127.0.0.1:7125".to_owned(),
            fullscreen: false,
            steps: Steps::default(),
            temperature_presets: vec![
                TemperaturePreset::new("PLA", 210.0, 60.0),
                TemperaturePreset::new("PETG", 240.0, 80.0),
                TemperaturePreset::new("ABS", 250.0, 105.0),
                TemperaturePreset::new("TPU", 225.0, 50.0),
            ],
            known_macros: Vec::new(),
            hidden_macros: Vec::new(),
        }
    }
}

impl Default for Steps {
    fn default() -> Self {
        Self { move_mm: 10.0, z_offset_mm: 0.025, temperature: 5.0, extrude_mm: 10.0, extrude_speed: 5.0 }
    }
}

impl TemperaturePreset {
    pub fn new(name: &str, extruder: f64, bed: f64) -> Self {
        Self { name: name.to_owned(), extruder, bed }
    }
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("cannot access {path}: {source}")]
    Io { path: PathBuf, source: std::io::Error },
    #[error("invalid settings file {path}: {source}")]
    Format { path: PathBuf, source: serde_json::Error },
    #[error("no configuration directory available on this system")]
    NoConfigDirectory,
}

/// Loads and saves settings at a fixed location.
#[derive(Debug, Clone)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Default location: `<config dir>/klipper-touch/settings.json`.
    pub fn default_location() -> Result<Self, SettingsError> {
        let directory = dirs::config_dir().ok_or(SettingsError::NoConfigDirectory)?;
        Ok(Self::new(directory.join(APP_FOLDER).join(FILE_NAME)))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// If the file is missing, the defaults apply.
    pub fn load(&self) -> Result<Settings, SettingsError> {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => {
                serde_json::from_str(&text).map_err(|source| SettingsError::Format { path: self.path.clone(), source })
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
            Err(source) => Err(SettingsError::Io { path: self.path.clone(), source }),
        }
    }

    /// Writes atomically via a temporary file so a power loss never leaves a half-written file.
    pub fn save(&self, settings: &Settings) -> Result<(), SettingsError> {
        let io_error = |source| SettingsError::Io { path: self.path.clone(), source };
        if let Some(directory) = self.path.parent() {
            std::fs::create_dir_all(directory).map_err(io_error)?;
        }
        let text = serde_json::to_string_pretty(settings)
            .map_err(|source| SettingsError::Format { path: self.path.clone(), source })?;
        let temporary = self.path.with_extension("json.tmp");
        std::fs::write(&temporary, text).map_err(io_error)?;
        std::fs::rename(&temporary, &self.path).map_err(io_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn new_macros_are_known_and_visible() {
        let mut settings = Settings::default();

        assert!(settings.reconcile_macros(&names(&["PARK", "PRINT_END"])));
        assert_eq!(settings.known_macros, names(&["PARK", "PRINT_END"]));
        assert!(settings.hidden_macros.is_empty());
        assert!(!settings.reconcile_macros(&names(&["PARK", "PRINT_END"])));
    }

    #[test]
    fn vanished_macros_are_forgotten_including_hidden() {
        let mut settings = Settings {
            known_macros: names(&["PARK", "OLD"]),
            hidden_macros: names(&["PARK", "OLD"]),
            ..Settings::default()
        };

        assert!(settings.reconcile_macros(&names(&["PARK"])));
        assert_eq!(settings.known_macros, names(&["PARK"]));
        assert_eq!(settings.hidden_macros, names(&["PARK"]));
    }

    #[test]
    fn returning_macro_counts_as_new_and_visible_again() {
        let mut settings =
            Settings { known_macros: names(&["PARK"]), hidden_macros: names(&["PARK"]), ..Settings::default() };

        settings.reconcile_macros(&[]);
        settings.reconcile_macros(&names(&["PARK"]));

        assert!(settings.hidden_macros.is_empty());
        assert_eq!(settings.known_macros, names(&["PARK"]));
    }

    fn temporary_store(name: &str) -> SettingsStore {
        let path = std::env::temp_dir().join(format!("kt-settings-{name}-{}", std::process::id())).join(FILE_NAME);
        let _ = std::fs::remove_file(&path);
        SettingsStore::new(path)
    }

    #[test]
    fn missing_file_gives_defaults() {
        let store = temporary_store("missing");

        assert_eq!(store.load().unwrap(), Settings::default());
    }

    #[test]
    fn saved_settings_load_again() {
        let store = temporary_store("roundtrip");
        let settings = Settings {
            moonraker: "http://voron.local".to_owned(),
            steps: Steps { move_mm: 50.0, ..Steps::default() },
            ..Settings::default()
        };

        store.save(&settings).unwrap();

        assert_eq!(store.load().unwrap(), settings);
    }

    #[test]
    fn partial_file_keeps_defaults_for_missing_fields() {
        let store = temporary_store("partial");
        std::fs::create_dir_all(store.path().parent().unwrap()).unwrap();
        std::fs::write(store.path(), r#"{ "fullscreen": true, "steps": { "move_mm": 1 } }"#).unwrap();

        let settings = store.load().unwrap();

        assert!(settings.fullscreen);
        assert_eq!(settings.steps.move_mm, 1.0);
        assert_eq!(settings.steps.temperature, Steps::default().temperature);
        assert_eq!(settings.temperature_presets.len(), 4);
    }

    #[test]
    fn broken_file_is_reported() {
        let store = temporary_store("broken");
        std::fs::create_dir_all(store.path().parent().unwrap()).unwrap();
        std::fs::write(store.path(), "{ nope").unwrap();

        assert!(matches!(store.load(), Err(SettingsError::Format { .. })));
    }
}
