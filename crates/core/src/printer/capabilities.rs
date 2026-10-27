//! Printer capabilities, derived from `printer.objects.list` and `server.info`.
//!
//! The UI only offers functions when the matching capability was detected.

use super::devices::{self, FanKind, WhiteChannel};
use super::macros::{self, MacroDetails};
use crate::tuning::TuningConfig;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Capabilities {
    pub heater_bed: bool,
    pub part_fan: bool,
    pub quad_gantry_level: bool,
    pub z_tilt: bool,
    pub bed_mesh: bool,
    pub probe: bool,
    pub firmware_retraction: bool,
    pub exclude_object: bool,
    pub mmu: bool,
    /// Additional heaters with a target temperature (`heater_generic`, `temperature_fan`, additional extruders).
    pub extra_heaters: Vec<String>,
    /// Temperatures without a target: `temperature_sensor` and the drivers that measure their own (`tmc2240`).
    /// Klipper adds a `temperature_host` or `temperature_combined` object next to the section that uses that sensor
    /// type (`temperature_sensor cb1` brings `temperature_host cb1`); those are the same sensor and left out.
    pub temperature_sensors: Vec<String>,
    pub filament_sensors: Vec<String>,
    /// `fan`, `fan_generic`, `heater_fan` and `controller_fan` objects.
    pub fans: Vec<String>,
    /// LED objects (`led`, `neopixel`, `dotstar`, `pca9533`, `pca9632`), each its own config section; one object
    /// can be a chain of several chips.
    pub leds: Vec<String>,
    /// Which chips of each LED have a white channel, from the configuration.
    pub led_white: Vec<(String, WhiteChannel)>,
    /// Effects of the optional led_effect plugin, by object name (`led_effect <name>`).
    pub led_effects: Vec<String>,
    /// The configured motion and extrusion values, for the expert tab.
    pub tuning: TuningConfig,
    /// Callable macros (excluding helper macros starting with `_`), in upper case.
    pub macros: Vec<String>,
    /// Parameters of each macro by upper-case name, read once from the configuration when subscribing (it only
    /// changes with a restart). Shared, because every snapshot carries the capabilities.
    pub macro_details: Arc<BTreeMap<String, MacroDetails>>,
    pub server: ServerComponents,
}

/// Where Load and Unload come from, see [`Capabilities::filament_handling`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilamentHandling {
    Mmu,
    Macros { load: bool, unload: bool },
    None,
}

/// Loaded Moonraker components that enable their own operator functions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerComponents {
    pub power: bool,
    pub update_manager: bool,
    pub machine: bool,
    pub history: bool,
    /// Moonraker's Spoolman component is loaded, which means that Spoolman is configured.
    pub spoolman: bool,
}

impl ServerComponents {
    pub fn from_names<'a>(components: impl IntoIterator<Item = &'a str>) -> Self {
        let mut found = Self::default();
        for component in components {
            match component {
                "power" => found.power = true,
                "update_manager" => found.update_manager = true,
                "machine" => found.machine = true,
                "history" => found.history = true,
                "spoolman" => found.spoolman = true,
                _ => {}
            }
        }
        found
    }
}

impl Capabilities {
    pub fn from_objects<'a>(names: impl IntoIterator<Item = &'a str>) -> Self {
        let mut capabilities = Self::default();
        for name in names {
            capabilities.add_object(name);
        }
        capabilities.macros.sort();
        capabilities
    }

    /// What only the configuration tells: macro parameters, LEDs with white, the configured tuning values. Read once
    /// when subscribing; the configuration changes only with a restart.
    pub fn apply_settings(&mut self, settings: Option<&Value>) {
        self.macro_details = Arc::new(macros::details_from_settings(settings));
        let section = |object: &str| settings.and_then(|settings| settings.get(object));
        self.led_white = self.leds.iter().map(|led| (led.clone(), devices::white_channel(section(led)))).collect();
        self.tuning = TuningConfig::from_settings(settings);
    }

    pub fn white_channel(&self, led: &str) -> WhiteChannel {
        self.led_white.iter().find(|(candidate, _)| candidate == led).map(|(_, white)| *white).unwrap_or_default()
    }

    /// The LED the light switch of the status bar turns on and off.
    pub fn main_light(&self) -> Option<&str> {
        devices::main_light(&self.leds)
    }

    pub fn has_macro(&self, name: &str) -> bool {
        self.macros.iter().any(|candidate| candidate.eq_ignore_ascii_case(name))
    }

    /// What the Load and Unload buttons run: Happy Hare first, otherwise the macros `LOAD_FILAMENT` and
    /// `UNLOAD_FILAMENT` (as KlipperScreen does), otherwise nothing. There is no generic fallback: the sequence
    /// depends on the length of the bowden tube and the temperatures of the machine.
    pub fn filament_handling(&self) -> FilamentHandling {
        if self.mmu {
            return FilamentHandling::Mmu;
        }
        let (load, unload) = (self.has_macro("LOAD_FILAMENT"), self.has_macro("UNLOAD_FILAMENT"));
        if load || unload {
            FilamentHandling::Macros { load, unload }
        } else {
            FilamentHandling::None
        }
    }

    fn add_object(&mut self, name: &str) {
        let (kind, instance) = name.split_once(' ').map_or((name, None), |(kind, instance)| (kind, Some(instance)));
        match (kind, instance) {
            ("heater_bed", None) => self.heater_bed = true,
            ("fan", None) => {
                self.part_fan = true;
                self.fans.push(name.to_owned());
            }
            ("quad_gantry_level", None) => self.quad_gantry_level = true,
            ("z_tilt", None) => self.z_tilt = true,
            ("bed_mesh", None) => self.bed_mesh = true,
            ("probe" | "bltouch" | "smart_effector" | "probe_eddy_current", _) => self.probe = true,
            ("firmware_retraction", None) => self.firmware_retraction = true,
            ("exclude_object", None) => self.exclude_object = true,
            ("mmu", None) => self.mmu = true,
            ("heater_generic" | "temperature_fan", Some(_)) => self.extra_heaters.push(name.to_owned()),
            ("temperature_sensor" | "tmc2240", Some(_)) => self.temperature_sensors.push(name.to_owned()),
            ("led_effect", Some(_)) => self.led_effects.push(name.to_owned()),
            _ if FanKind::of(name).is_some() => self.fans.push(name.to_owned()),
            _ if devices::is_led(name) => self.leds.push(name.to_owned()),
            ("filament_switch_sensor" | "filament_motion_sensor", Some(_)) => {
                self.filament_sensors.push(name.to_owned())
            }
            ("gcode_macro", Some(macro_name)) if !macro_name.starts_with('_') => {
                self.macros.push(macro_name.to_ascii_uppercase());
            }
            (extruder, None) if is_additional_extruder(extruder) => self.extra_heaters.push(name.to_owned()),
            _ => {}
        }
    }
}

fn is_additional_extruder(name: &str) -> bool {
    name.strip_prefix("extruder")
        .is_some_and(|number| !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_optional_hardware_and_macros() {
        let capabilities = Capabilities::from_objects([
            "webhooks",
            "extruder",
            "extruder1",
            "heater_bed",
            "fan",
            "quad_gantry_level",
            "bed_mesh",
            "bltouch",
            "heater_generic chamber",
            "temperature_sensor raspberry_pi",
            "temperature_host raspberry_pi",
            "filament_switch_sensor runout",
            "gcode_macro PRINT_START",
            "gcode_macro load_filament",
            "gcode_macro _HELPER",
            "mmu",
        ]);

        assert!(capabilities.heater_bed && capabilities.part_fan && capabilities.quad_gantry_level);
        assert!(capabilities.bed_mesh && capabilities.probe && capabilities.mmu);
        assert!(!capabilities.z_tilt);
        assert_eq!(capabilities.extra_heaters, ["extruder1", "heater_generic chamber"]);
        assert_eq!(capabilities.temperature_sensors, ["temperature_sensor raspberry_pi"]);
        assert_eq!(capabilities.filament_sensors, ["filament_switch_sensor runout"]);
        assert_eq!(capabilities.macros, ["LOAD_FILAMENT", "PRINT_START"]);
        assert!(capabilities.has_macro("load_filament"));
    }

    #[test]
    fn load_and_unload_prefer_happy_hare_then_macros() {
        let handling = |objects: &[&str]| Capabilities::from_objects(objects.iter().copied()).filament_handling();

        assert_eq!(handling(&["mmu", "gcode_macro LOAD_FILAMENT"]), FilamentHandling::Mmu);
        assert_eq!(
            handling(&["gcode_macro load_filament", "gcode_macro UNLOAD_FILAMENT"]),
            FilamentHandling::Macros { load: true, unload: true }
        );
        assert_eq!(handling(&["gcode_macro LOAD_FILAMENT"]), FilamentHandling::Macros { load: true, unload: false });
        assert_eq!(handling(&["gcode_macro PRINT_START", "extruder"]), FilamentHandling::None);
    }

    #[test]
    fn reads_moonraker_components() {
        let components = ServerComponents::from_names(["file_manager", "power", "update_manager", "machine"]);

        assert_eq!(
            components,
            ServerComponents { power: true, update_manager: true, machine: true, history: false, spoolman: false }
        );
        assert!(ServerComponents::from_names(["spoolman"]).spoolman);
    }
}
