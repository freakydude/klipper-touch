//! Typed view of what Klipper reports, read from the [`ObjectStore`].

use super::devices::{self, Fan, Led};
use crate::sources::objects::ObjectStore;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KlippyState {
    #[default]
    Disconnected,
    Startup,
    Ready,
    Shutdown,
    Error,
}

impl KlippyState {
    pub fn parse(text: &str) -> Self {
        match text {
            "startup" => Self::Startup,
            "ready" => Self::Ready,
            "shutdown" => Self::Shutdown,
            "error" => Self::Error,
            _ => Self::Disconnected,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disconnected => "disconnected",
            Self::Startup => "startup",
            Self::Ready => "ready",
            Self::Shutdown => "shutdown",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrintState {
    #[default]
    Standby,
    Printing,
    Paused,
    Complete,
    Cancelled,
    Error,
}

impl PrintState {
    pub fn parse(text: &str) -> Self {
        match text {
            "printing" => Self::Printing,
            "paused" => Self::Paused,
            "complete" => Self::Complete,
            "cancelled" => Self::Cancelled,
            "error" => Self::Error,
            _ => Self::Standby,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Standby => "standby",
            Self::Printing => "printing",
            Self::Paused => "paused",
            Self::Complete => "complete",
            Self::Cancelled => "cancelled",
            Self::Error => "error",
        }
    }

    /// A job is running or paused; manual moves and extrusion are locked then.
    pub fn is_active(self) -> bool {
        matches!(self, Self::Printing | Self::Paused)
    }

    /// The job has ended, well or not.
    pub fn is_finished(self) -> bool {
        matches!(self, Self::Complete | Self::Cancelled | Self::Error)
    }
}

/// `idle_timeout.state`: whether the machine is working, independent of a print job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Activity {
    #[default]
    Ready,
    /// Klipper calls this "Printing": any command is being executed, e.g. homing or heating.
    Busy,
    /// Inactive for longer than the idle timeout; Klipper has usually switched off heaters and motors.
    Idle,
}

impl Activity {
    pub fn parse(text: &str) -> Self {
        match text {
            "Printing" => Self::Busy,
            "Idle" => Self::Idle,
            _ => Self::Ready,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Busy => "busy",
            Self::Idle => "idle",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn letter(self) -> char {
        match self {
            Self::X => 'X',
            Self::Y => 'Y',
            Self::Z => 'Z',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HomedAxes {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl HomedAxes {
    pub fn parse(text: &str) -> Self {
        let text = text.to_ascii_lowercase();
        Self { x: text.contains('x'), y: text.contains('y'), z: text.contains('z') }
    }

    pub fn contains(self, axis: Axis) -> bool {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    pub fn all(self) -> bool {
        self.x && self.y && self.z
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrintJob {
    pub state: PrintState,
    pub filename: String,
    pub message: String,
    pub progress: f64,
    pub print_duration: f64,
    pub total_duration: f64,
    pub filament_used: f64,
    pub current_layer: Option<u32>,
    pub total_layer: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Motion {
    pub position: [f64; 4],
    pub live_position: [f64; 4],
    pub homed: HomedAxes,
    pub axis_minimum: [f64; 4],
    pub axis_maximum: [f64; 4],
    pub max_velocity: f64,
    pub max_accel: f64,
    pub square_corner_velocity: f64,
    pub minimum_cruise_ratio: f64,
    pub live_velocity: f64,
    pub live_extruder_velocity: f64,
    pub homing_origin: [f64; 4],
    pub speed_factor: f64,
    pub extrude_factor: f64,
}

/// Lengths in mm the Filament page offers for extruding by hand.
pub const EXTRUDE_LENGTHS_MM: [f64; 5] = [5.0, 10.0, 25.0, 50.0, 100.0];
/// Klipper's default of `max_extrude_only_distance` in mm.
const DEFAULT_MAX_EXTRUDE_ONLY_MM: f64 = 50.0;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExtruderState {
    pub pressure_advance: f64,
    pub smooth_time: f64,
    pub can_extrude: bool,
    /// Configured `filament_diameter` in mm; `None` until the configuration is known.
    pub filament_diameter: Option<f64>,
    /// Configured `max_extrude_only_distance` in mm: Klipper refuses longer moves of the extruder alone.
    pub max_extrude_only_mm: Option<f64>,
}

impl ExtruderState {
    /// The lengths of [`EXTRUDE_LENGTHS_MM`] Klipper accepts (Klipper's default limit while the configuration is
    /// unknown); the limit itself when it is shorter than all of them.
    pub fn extrude_lengths(&self) -> Vec<f64> {
        let limit = self.max_extrude_only_mm.unwrap_or(DEFAULT_MAX_EXTRUDE_ONLY_MM);
        let lengths: Vec<f64> = EXTRUDE_LENGTHS_MM.into_iter().filter(|length| *length <= limit).collect();
        if lengths.is_empty() {
            vec![limit]
        } else {
            lengths
        }
    }
}

/// Heater with a target temperature; `object` is the Klipper object name, e.g. `heater_generic chamber`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Heater {
    pub object: String,
    pub label: String,
    pub temperature: f64,
    pub target: f64,
    pub power: f64,
    /// `min_temp`/`max_temp` from the Klipper configuration; Klipper rejects targets outside (except 0 = off).
    pub min_temp: f64,
    pub max_temp: Option<f64>,
}

impl Heater {
    /// Target after pressing + or − (`step` with sign): jumps over the forbidden range between off and
    /// `min_temp` and stops at `max_temp`.
    pub fn stepped_target(&self, step: f64) -> f64 {
        let maximum = self.max_temp.unwrap_or(f64::INFINITY);
        let next = (self.target + step).min(maximum);
        if next <= 0.0 || (step < 0.0 && next < self.min_temp) {
            0.0
        } else {
            next.max(self.min_temp)
        }
    }

    pub fn can_raise(&self) -> bool {
        self.max_temp.is_none_or(|maximum| self.target < maximum)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TemperatureSensor {
    pub object: String,
    pub label: String,
    /// `None` if the sensor has no value right now, e.g. a stepper driver that is switched off.
    pub temperature: Option<f64>,
}

/// Firmware retraction (`[firmware_retraction]`), as `G10`/`G11` use it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Retraction {
    pub retract_length: f64,
    pub retract_speed: f64,
    pub unretract_extra_length: f64,
    pub unretract_speed: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FilamentSensor {
    pub object: String,
    pub label: String,
    pub enabled: bool,
    pub detected: bool,
}

/// An object of the running print that `EXCLUDE_OBJECT` can skip.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PrintObject {
    pub name: String,
    pub excluded: bool,
}

/// The parts of the state that Klipper itself reports, read from its objects.
pub(in crate::sources) struct KlipperParts {
    pub klippy_state: KlippyState,
    pub klippy_message: String,
    pub activity: Activity,
    pub job: PrintJob,
    pub motion: Motion,
    pub extruder: ExtruderState,
    pub heaters: Vec<Heater>,
    pub sensors: Vec<TemperatureSensor>,
    pub fans: Vec<Fan>,
    pub leds: Vec<Led>,
    pub retraction: Option<Retraction>,
    pub save_config_pending: bool,
    pub leveling_applied: Option<bool>,
    pub filament_sensors: Vec<FilamentSensor>,
    pub print_objects: Vec<PrintObject>,
}

impl KlipperParts {
    pub(in crate::sources) fn read(store: &ObjectStore) -> Self {
        let object = |name: &str| store.object(name).unwrap_or(&Value::Null);
        let webhooks = object("webhooks");
        Self {
            klippy_state: KlippyState::parse(text(webhooks, "state")),
            klippy_message: text(webhooks, "state_message").trim().to_owned(),
            activity: Activity::parse(text(object("idle_timeout"), "state")),
            job: read_job(object("print_stats"), object("display_status"), object("virtual_sdcard")),
            motion: read_motion(object("toolhead"), object("motion_report"), object("gcode_move")),
            extruder: read_extruder(object("extruder"), object("configfile").pointer("/settings/extruder")),
            heaters: read_heaters(store),
            sensors: read_sensors(store),
            fans: devices::read_fans(store),
            leds: devices::read_leds(store),
            retraction: store.object("firmware_retraction").map(read_retraction),
            save_config_pending: flag(object("configfile"), "save_config_pending"),
            leveling_applied: store
                .object("quad_gantry_level")
                .or_else(|| store.object("z_tilt"))
                .map(|leveling| flag(leveling, "applied")),
            filament_sensors: read_filament_sensors(store),
            print_objects: read_print_objects(object("exclude_object")),
        }
    }
}

/// `objects` lists `{ "name": ... }` entries, `excluded_objects` lists plain names.
fn read_print_objects(exclude_object: &Value) -> Vec<PrintObject> {
    let excluded: Vec<&str> = exclude_object
        .get("excluded_objects")
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let objects = exclude_object.get("objects").and_then(Value::as_array);
    objects
        .into_iter()
        .flatten()
        .filter_map(|object| object.get("name").and_then(Value::as_str))
        .map(|name| PrintObject { name: name.to_owned(), excluded: excluded.contains(&name) })
        .collect()
}

fn read_job(print_stats: &Value, display_status: &Value, virtual_sdcard: &Value) -> PrintJob {
    let info = print_stats.get("info").unwrap_or(&Value::Null);
    let progress = display_status
        .get("progress")
        .or_else(|| virtual_sdcard.get("progress"))
        .and_then(Value::as_f64)
        .unwrap_or_default();
    PrintJob {
        state: PrintState::parse(text(print_stats, "state")),
        filename: text(print_stats, "filename").to_owned(),
        message: text(print_stats, "message").to_owned(),
        progress: progress.clamp(0.0, 1.0),
        print_duration: number(print_stats, "print_duration"),
        total_duration: number(print_stats, "total_duration"),
        filament_used: number(print_stats, "filament_used"),
        current_layer: count(info, "current_layer"),
        total_layer: count(info, "total_layer"),
    }
}

fn read_motion(toolhead: &Value, motion_report: &Value, gcode_move: &Value) -> Motion {
    Motion {
        position: vector(toolhead, "position"),
        live_position: vector(motion_report, "live_position"),
        homed: HomedAxes::parse(text(toolhead, "homed_axes")),
        axis_minimum: vector(toolhead, "axis_minimum"),
        axis_maximum: vector(toolhead, "axis_maximum"),
        max_velocity: number(toolhead, "max_velocity"),
        max_accel: number(toolhead, "max_accel"),
        square_corner_velocity: number(toolhead, "square_corner_velocity"),
        minimum_cruise_ratio: number(toolhead, "minimum_cruise_ratio"),
        live_velocity: number(motion_report, "live_velocity"),
        live_extruder_velocity: number(motion_report, "live_extruder_velocity"),
        homing_origin: vector(gcode_move, "homing_origin"),
        speed_factor: number_or(gcode_move, "speed_factor", 1.0),
        extrude_factor: number_or(gcode_move, "extrude_factor", 1.0),
    }
}

fn read_extruder(extruder: &Value, configuration: Option<&Value>) -> ExtruderState {
    ExtruderState {
        pressure_advance: number(extruder, "pressure_advance"),
        smooth_time: number(extruder, "smooth_time"),
        can_extrude: flag(extruder, "can_extrude"),
        filament_diameter: configuration.and_then(|section| section.get("filament_diameter")).and_then(Value::as_f64),
        max_extrude_only_mm: configuration
            .and_then(|section| section.get("max_extrude_only_distance"))
            .and_then(Value::as_f64),
    }
}

fn read_heaters(store: &ObjectStore) -> Vec<Heater> {
    let settings = store.object("configfile").and_then(|configfile| configfile.get("settings"));
    let heater = |name: &str, value: &Value| heater(name, value, settings.and_then(|settings| settings.get(name)));
    let mut heaters: Vec<Heater> = ["extruder", "heater_bed"]
        .into_iter()
        .filter_map(|name| store.object(name).map(|value| heater(name, value)))
        .collect();
    heaters.extend(
        store
            .names()
            .filter(|name| is_extra_heater(name))
            .filter_map(|name| store.object(name).map(|value| heater(name, value))),
    );
    heaters
}

fn is_extra_heater(name: &str) -> bool {
    name.starts_with("heater_generic ")
        || name.starts_with("temperature_fan ")
        || (name.starts_with("extruder") && name != "extruder" && !name.contains(' '))
}

fn heater(object: &str, value: &Value, settings: Option<&Value>) -> Heater {
    let settings = settings.unwrap_or(&Value::Null);
    Heater {
        object: object.to_owned(),
        label: label(object),
        temperature: number(value, "temperature"),
        target: number(value, "target"),
        power: number(value, "power"),
        min_temp: number(settings, "min_temp").max(0.0),
        max_temp: settings.get("max_temp").and_then(Value::as_f64),
    }
}

/// Object kinds that are only a temperature; `tmc2240` reports the temperature of the driver. Not
/// `temperature_host`/`temperature_combined`: Klipper adds them beside the section that uses the sensor type, so they
/// would show the same sensor twice (see `Capabilities::temperature_sensors`).
const SENSOR_KINDS: [&str; 2] = ["temperature_sensor", "tmc2240"];

/// In the order of [`SENSOR_KINDS`], each kind in name order.
fn read_sensors(store: &ObjectStore) -> Vec<TemperatureSensor> {
    SENSOR_KINDS
        .iter()
        .flat_map(|kind| {
            store.names().filter(move |name| name.split_once(' ').is_some_and(|(prefix, _)| prefix == *kind))
        })
        .filter_map(|name| {
            store.object(name).map(|value| TemperatureSensor {
                object: name.to_owned(),
                label: label(name),
                temperature: value.get("temperature").and_then(Value::as_f64),
            })
        })
        .collect()
}

fn read_retraction(value: &Value) -> Retraction {
    Retraction {
        retract_length: number(value, "retract_length"),
        retract_speed: number(value, "retract_speed"),
        unretract_extra_length: number(value, "unretract_extra_length"),
        unretract_speed: number(value, "unretract_speed"),
    }
}

fn read_filament_sensors(store: &ObjectStore) -> Vec<FilamentSensor> {
    store
        .names()
        .filter(|name| name.starts_with("filament_switch_sensor ") || name.starts_with("filament_motion_sensor "))
        .filter_map(|name| {
            store.object(name).map(|value| FilamentSensor {
                object: name.to_owned(),
                label: label(name),
                enabled: flag(value, "enabled"),
                detected: flag(value, "filament_detected"),
            })
        })
        .collect()
}

/// Display name from the object name: `heater_generic chamber` → `Chamber`, `heater_bed` → `Bed`.
pub fn label(object: &str) -> String {
    let base = match object {
        "extruder" => "nozzle",
        "heater_bed" => "bed",
        other => other.split_once(' ').map_or(other, |(_, instance)| instance),
    };
    let words: Vec<String> = base
        .split(['_', ' '])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut characters = word.chars();
            characters.next().map_or_else(String::new, |first| first.to_uppercase().chain(characters).collect())
        })
        .collect();
    words.join(" ")
}

fn text<'a>(object: &'a Value, field: &str) -> &'a str {
    object.get(field).and_then(Value::as_str).unwrap_or_default()
}

fn number(object: &Value, field: &str) -> f64 {
    number_or(object, field, 0.0)
}

fn number_or(object: &Value, field: &str, default: f64) -> f64 {
    object.get(field).and_then(Value::as_f64).unwrap_or(default)
}

fn flag(object: &Value, field: &str) -> bool {
    object.get(field).and_then(Value::as_bool).unwrap_or_default()
}

fn count(object: &Value, field: &str) -> Option<u32> {
    object.get(field).and_then(Value::as_u64).and_then(|value| u32::try_from(value).ok())
}

fn vector(object: &Value, field: &str) -> [f64; 4] {
    let mut values = [0.0; 4];
    if let Some(items) = object.get(field).and_then(Value::as_array) {
        for (slot, item) in values.iter_mut().zip(items) {
            *slot = item.as_f64().unwrap_or_default();
        }
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::PrinterStatus;
    use serde_json::json;

    fn store(status: Value) -> ObjectStore {
        let mut store = ObjectStore::default();
        store.replace(&status);
        store
    }

    #[test]
    fn reads_job_motion_and_heaters() {
        let status = PrinterStatus::from_store(&store(json!({
            "webhooks": { "state": "ready", "state_message": "Printer is ready\n" },
            "idle_timeout": { "state": "Printing" },
            "print_stats": {
                "state": "printing", "filename": "cube.gcode", "print_duration": 60.0,
                "info": { "current_layer": 3, "total_layer": 100 }
            },
            "display_status": { "progress": 0.25 },
            "toolhead": { "position": [1.0, 2.0, 3.0, 4.0], "homed_axes": "xyz", "axis_maximum": [300.0, 300.0, 250.0, 0.0] },
            "gcode_move": { "homing_origin": [0.0, 0.0, 0.05, 0.0], "speed_factor": 1.2 },
            "extruder": { "temperature": 200.5, "target": 210.0, "pressure_advance": 0.04, "can_extrude": true },
            "heater_bed": { "temperature": 60.0, "target": 60.0 },
            "heater_generic chamber": { "temperature": 35.0, "target": 0.0 },
            "temperature_sensor mcu_temp": { "temperature": 41.0 },
            "tmc2240 stepper_x": { "temperature": null },
            "temperature_sensor pi": { "temperature": 48.5 },
            "temperature_host pi": { "temperature": 48.5 },
            "fan": { "speed": 0.5 },
            "quad_gantry_level": { "applied": true }
        })));

        assert_eq!(status.klippy_state, KlippyState::Ready);
        assert_eq!(status.klippy_message, "Printer is ready");
        assert_eq!(status.activity, Activity::Busy);
        assert_eq!(status.job.state, PrintState::Printing);
        assert_eq!(status.job.progress, 0.25);
        assert_eq!((status.job.current_layer, status.job.total_layer), (Some(3), Some(100)));
        assert!(status.motion.homed.all());
        assert_eq!(status.motion.homing_origin[2], 0.05);
        assert_eq!(status.motion.extrude_factor, 1.0);
        assert!(status.extruder.can_extrude);
        let heaters: Vec<&str> = status.heaters.iter().map(|heater| heater.label.as_str()).collect();
        assert_eq!(heaters, ["Nozzle", "Bed", "Chamber"]);
        let sensors: Vec<(&str, Option<f64>)> =
            status.sensors.iter().map(|sensor| (sensor.label.as_str(), sensor.temperature)).collect();
        assert_eq!(sensors, [("Mcu Temp", Some(41.0)), ("Pi", Some(48.5)), ("Stepper X", None)]);
        assert_eq!(status.fans[0].speed, 0.5);
        assert_eq!(status.leveling_applied, Some(true));
    }

    #[test]
    fn missing_objects_give_safe_defaults() {
        let status = PrinterStatus::from_store(&ObjectStore::default());

        assert_eq!(status.klippy_state, KlippyState::Disconnected);
        assert_eq!(status.activity, Activity::Ready);
        assert_eq!(Activity::parse("Idle"), Activity::Idle);
        assert_eq!(status.job.state, PrintState::Standby);
        assert!(status.heaters.is_empty());
        assert!(status.fans.is_empty() && status.leds.is_empty() && status.retraction.is_none());
        assert_eq!(status.leveling_applied, None);
    }

    #[test]
    fn reads_heater_limits_from_the_configuration() {
        let status = PrinterStatus::from_store(&store(json!({
            "extruder": { "temperature": 20.0, "target": 0.0 },
            "heater_bed": { "temperature": 20.0, "target": 0.0 },
            "configfile": { "settings": { "extruder": { "min_temp": 10.0, "max_temp": 280.0 } } }
        })));

        let nozzle = status.heater("extruder").unwrap();
        assert_eq!((nozzle.min_temp, nozzle.max_temp), (10.0, Some(280.0)));
        let bed = status.heater("heater_bed").unwrap();
        assert_eq!((bed.min_temp, bed.max_temp), (0.0, None));
    }

    #[test]
    fn only_lengths_klipper_accepts_are_offered_for_extruding() {
        let configured = |limit: f64| ExtruderState { max_extrude_only_mm: Some(limit), ..ExtruderState::default() };

        assert_eq!(ExtruderState::default().extrude_lengths(), [5.0, 10.0, 25.0, 50.0]);
        assert_eq!(configured(100.0).extrude_lengths(), EXTRUDE_LENGTHS_MM);
        assert_eq!(configured(30.0).extrude_lengths(), [5.0, 10.0, 25.0]);
        assert_eq!(configured(2.0).extrude_lengths(), [2.0]);
        let status = PrinterStatus::from_store(&store(json!({
            "extruder": { "temperature": 20.0 },
            "configfile": { "settings": { "extruder": { "max_extrude_only_distance": 100.0 } } }
        })));
        assert_eq!(status.extruder.max_extrude_only_mm, Some(100.0));
    }

    #[test]
    fn reads_the_filament_diameter_from_the_configuration() {
        let status = PrinterStatus::from_store(&store(json!({
            "extruder": { "temperature": 20.0 },
            "configfile": { "settings": { "extruder": { "filament_diameter": 2.85 } } }
        })));

        assert_eq!(status.extruder.filament_diameter, Some(2.85));
        assert_eq!(PrinterStatus::from_store(&ObjectStore::default()).extruder.filament_diameter, None);
    }

    #[test]
    fn steps_skip_the_range_below_min_temp_and_stop_at_max_temp() {
        let heater = |target| Heater { target, min_temp: 40.0, max_temp: Some(280.0), ..Heater::default() };

        assert_eq!(heater(0.0).stepped_target(5.0), 40.0);
        assert_eq!(heater(40.0).stepped_target(5.0), 45.0);
        assert_eq!(heater(45.0).stepped_target(-10.0), 0.0);
        assert_eq!(heater(60.0).stepped_target(-10.0), 50.0);
        assert_eq!(heater(0.0).stepped_target(-5.0), 0.0);
        assert_eq!(heater(275.0).stepped_target(10.0), 280.0);
        assert_eq!(heater(300.0).stepped_target(-5.0), 280.0);
        assert!(!heater(280.0).can_raise());

        let unlimited = Heater { target: 0.0, ..Heater::default() };
        assert_eq!(unlimited.stepped_target(5.0), 5.0);
        assert!(unlimited.can_raise());
    }

    #[test]
    fn labels_are_human_readable() {
        assert_eq!(label("extruder"), "Nozzle");
        assert_eq!(label("heater_bed"), "Bed");
        assert_eq!(label("temperature_fan electronics_fan"), "Electronics Fan");
        assert_eq!(label("extruder1"), "Extruder1");
    }

    #[test]
    fn marks_excluded_print_objects() {
        let exclude_object = json!({
            "objects": [{ "name": "Benchy_0" }, { "name": "Benchy_1" }],
            "excluded_objects": ["Benchy_1"],
        });
        assert_eq!(
            read_print_objects(&exclude_object),
            vec![
                PrintObject { name: "Benchy_0".into(), excluded: false },
                PrintObject { name: "Benchy_1".into(), excluded: true },
            ]
        );
        assert!(read_print_objects(&Value::Null).is_empty());
    }
}
