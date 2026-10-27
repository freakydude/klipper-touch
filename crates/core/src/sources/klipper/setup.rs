//! What this Klipper has: read from `printer.objects.list` and, once per subscription, from its configuration (it
//! only changes with a restart, which subscribes again).

use super::devices::{self, FanKind, WhiteChannel};
use super::macros::{self, MacroDetails};
use super::tuning::TuningConfig;
use crate::sources::subscription::Subscription;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

const HEATER_FIELDS: &[&str] = &["temperature", "target", "power"];

/// How the gantry or bed is levelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leveling {
    /// `[quad_gantry_level]`, `QUAD_GANTRY_LEVEL`; wins if both are configured.
    QuadGantry,
    /// `[z_tilt]`, `Z_TILT_ADJUST`.
    ZTilt,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Setup {
    pub leveling: Option<Leveling>,
    /// A probe (`probe`, `bltouch`, `smart_effector`, `probe_eddy_current`): the Z offset goes to the probe, else
    /// to the endstop.
    pub probe: bool,
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
    /// The configured motion and extrusion values, for the expert tab.
    pub tuning: TuningConfig,
    /// Callable macros (excluding helper macros starting with `_`), in upper case.
    pub macros: Vec<String>,
    /// Parameters of each macro by upper-case name, read once from the configuration. Shared, because every
    /// snapshot carries the setup.
    pub macro_details: Arc<BTreeMap<String, MacroDetails>>,
}

impl Setup {
    pub fn from_objects<'a>(names: impl IntoIterator<Item = &'a str>) -> Self {
        let mut setup = Self::default();
        let (mut quad_gantry, mut z_tilt) = (false, false);
        for name in names {
            let (kind, instance) = name.split_once(' ').map_or((name, None), |(kind, instance)| (kind, Some(instance)));
            match (kind, instance) {
                ("quad_gantry_level", None) => quad_gantry = true,
                ("z_tilt", None) => z_tilt = true,
                ("probe" | "bltouch" | "smart_effector" | "probe_eddy_current", _) => setup.probe = true,
                ("heater_generic" | "temperature_fan", Some(_)) => setup.extra_heaters.push(name.to_owned()),
                ("temperature_sensor" | "tmc2240", Some(_)) => setup.temperature_sensors.push(name.to_owned()),
                _ if FanKind::of(name).is_some() => setup.fans.push(name.to_owned()),
                _ if devices::is_led(name) => setup.leds.push(name.to_owned()),
                ("filament_switch_sensor" | "filament_motion_sensor", Some(_)) => {
                    setup.filament_sensors.push(name.to_owned())
                }
                ("gcode_macro", Some(macro_name)) if !macro_name.starts_with('_') => {
                    setup.macros.push(macro_name.to_ascii_uppercase());
                }
                (extruder, None) if is_additional_extruder(extruder) => setup.extra_heaters.push(name.to_owned()),
                _ => {}
            }
        }
        setup.leveling = match (quad_gantry, z_tilt) {
            (true, _) => Some(Leveling::QuadGantry),
            (false, true) => Some(Leveling::ZTilt),
            (false, false) => None,
        };
        setup.macros.sort();
        setup
    }

    /// What only the configuration tells: macro parameters, LEDs with white, the configured tuning values.
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

    /// A Klipper with nothing but these macros, for tests of the macro conventions.
    #[cfg(test)]
    pub(crate) fn with_macros(macros: &[&str]) -> Self {
        Self { macros: macros.iter().map(|name| name.to_ascii_uppercase()).collect(), ..Self::default() }
    }

    /// Klipper's own objects and fields; `configfile.settings` holds the heater limits and is sent once, it does not
    /// change at runtime.
    pub(in crate::sources) fn subscribe(&self, subscription: &mut Subscription) {
        subscription.add("webhooks", None);
        subscription.add("print_stats", None);
        subscription.add("display_status", Some(&["progress", "message"]));
        subscription.add("virtual_sdcard", Some(&["progress", "is_active"]));
        subscription.add("idle_timeout", Some(&["state"]));
        subscription.add(
            "toolhead",
            Some(&[
                "position",
                "homed_axes",
                "axis_minimum",
                "axis_maximum",
                "max_velocity",
                "max_accel",
                "square_corner_velocity",
                "minimum_cruise_ratio",
            ]),
        );
        subscription.add("motion_report", Some(&["live_position", "live_velocity", "live_extruder_velocity"]));
        subscription.add("gcode_move", Some(&["homing_origin", "speed_factor", "extrude_factor"]));
        subscription.add(
            "extruder",
            Some(&["temperature", "target", "power", "pressure_advance", "smooth_time", "can_extrude"]),
        );
        subscription.add("configfile", Some(&["save_config_pending", "settings"]));
        subscription.add("heater_bed", Some(HEATER_FIELDS));
        subscription.add("quad_gantry_level", Some(&["applied"]));
        subscription.add("z_tilt", Some(&["applied"]));
        subscription.add(
            "firmware_retraction",
            Some(&["retract_length", "retract_speed", "unretract_extra_length", "unretract_speed"]),
        );
        subscription.add("exclude_object", Some(&["objects", "excluded_objects"]));
        for heater in &self.extra_heaters {
            subscription.add(heater, None);
        }
        for sensor in &self.temperature_sensors {
            subscription.add(sensor, Some(&["temperature"]));
        }
        for fan in &self.fans {
            subscription.add(fan, Some(&["speed", "rpm"]));
        }
        for led in &self.leds {
            subscription.add(led, Some(&["color_data"]));
        }
        for sensor in &self.filament_sensors {
            subscription.add(sensor, Some(&["enabled", "filament_detected"]));
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
        let setup = Setup::from_objects([
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
        ]);

        assert_eq!(setup.leveling, Some(Leveling::QuadGantry));
        assert!(setup.probe);
        assert_eq!(setup.fans, ["fan"]);
        assert_eq!(setup.extra_heaters, ["extruder1", "heater_generic chamber"]);
        assert_eq!(setup.temperature_sensors, ["temperature_sensor raspberry_pi"]);
        assert_eq!(setup.filament_sensors, ["filament_switch_sensor runout"]);
        assert_eq!(setup.macros, ["LOAD_FILAMENT", "PRINT_START"]);
        assert!(setup.has_macro("load_filament"));
    }

    #[test]
    fn quad_gantry_level_wins_over_z_tilt() {
        let leveling = |objects: &[&str]| Setup::from_objects(objects.iter().copied()).leveling;

        assert_eq!(leveling(&["z_tilt"]), Some(Leveling::ZTilt));
        assert_eq!(leveling(&["z_tilt", "quad_gantry_level"]), Some(Leveling::QuadGantry));
        assert_eq!(leveling(&["extruder"]), None);
    }
}
