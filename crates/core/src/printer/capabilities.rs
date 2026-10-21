//! Fähigkeiten des Druckers, abgeleitet aus `printer.objects.list` und `server.info`.
//!
//! Die UI bietet Funktionen nur an, wenn die passende Fähigkeit erkannt wurde.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
    /// Weitere Heizungen mit Zieltemperatur (`heater_generic`, `temperature_fan`, weitere Extruder).
    pub extra_heaters: Vec<String>,
    pub temperature_sensors: Vec<String>,
    pub filament_sensors: Vec<String>,
    /// Aufrufbare Makros (ohne mit `_` beginnende Hilfsmakros), in Großschreibung.
    pub macros: Vec<String>,
    pub server: ServerComponents,
}

/// Geladene Moonraker-Komponenten, die eigene Bedienfunktionen ermöglichen.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerComponents {
    pub power: bool,
    pub update_manager: bool,
    pub machine: bool,
    pub history: bool,
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

    pub fn has_macro(&self, name: &str) -> bool {
        self.macros.iter().any(|candidate| candidate.eq_ignore_ascii_case(name))
    }

    /// Mindestens ein Weg, das Bett bzw. den Gantry auszurichten.
    pub fn can_level(&self) -> bool {
        self.quad_gantry_level || self.z_tilt
    }

    fn add_object(&mut self, name: &str) {
        let (kind, instance) = name.split_once(' ').map_or((name, None), |(kind, instance)| (kind, Some(instance)));
        match (kind, instance) {
            ("heater_bed", None) => self.heater_bed = true,
            ("fan", None) => self.part_fan = true,
            ("quad_gantry_level", None) => self.quad_gantry_level = true,
            ("z_tilt", None) => self.z_tilt = true,
            ("bed_mesh", None) => self.bed_mesh = true,
            ("probe" | "bltouch" | "smart_effector" | "probe_eddy_current", _) => self.probe = true,
            ("firmware_retraction", None) => self.firmware_retraction = true,
            ("exclude_object", None) => self.exclude_object = true,
            ("mmu", None) => self.mmu = true,
            ("heater_generic" | "temperature_fan", Some(_)) => self.extra_heaters.push(name.to_owned()),
            ("temperature_sensor", Some(_)) => self.temperature_sensors.push(name.to_owned()),
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
    fn reads_moonraker_components() {
        let components = ServerComponents::from_names(["file_manager", "power", "update_manager", "machine"]);

        assert_eq!(components, ServerComponents { power: true, update_manager: true, machine: true, history: false });
    }
}
