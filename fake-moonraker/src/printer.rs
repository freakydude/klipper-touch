//! Simulated printer: heaters, toolhead and print job.

use crate::files::GcodeFile;
use crate::gcode::{Axis, Command, CommandError, MmuCommand, Move};
use crate::happy_hare::HappyHare;
use crate::machine::{MachineProfile, FILAMENT_SENSORS};
use serde_json::{json, Map, Value};
use std::time::Duration;

const AMBIENT_TEMPERATURE: f64 = 24.0;
const HEAT_RATE_PER_SECOND: f64 = 3.0;
const COOL_RATE_PER_SECOND: f64 = 0.5;
const MIN_EXTRUDE_TEMPERATURE: f64 = 170.0;
const MAX_EXTRUDER_TARGET: f64 = 300.0;
const MAX_BED_TARGET: f64 = 120.0;
const MIN_HEATER_TEMPERATURE: f64 = 10.0;
const MAX_CHAMBER_TARGET: f64 = 70.0;
const HOST_TEMPERATURE: f64 = 45.0;
/// Macros every simulated printer knows; `_` macros are helper macros and invisible to the UI.
const MACROS: [&str; 6] = ["PRINT_START", "PRINT_END", "LOAD_FILAMENT", "UNLOAD_FILAMENT", "PARK", "_PURGE_HELPER"];
const AXIS_MINIMUM: [f64; 4] = [0.0; 4];
const AXIS_MAXIMUM: [f64; 4] = [300.0, 300.0, 300.0, 0.0];
const ALL_AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
const EXTRUDER_INDEX: usize = 3;
const SHUTDOWN_MESSAGE: &str = "Shutdown due to M112 command";
/// Klipper's default `[idle_timeout] timeout`; afterwards heaters and motors are switched off.
const IDLE_TIMEOUT_SECONDS: f64 = 600.0;
/// How long `idle_timeout` reports "Printing" (busy) after a command.
const BUSY_SECONDS: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KlippyState {
    Ready,
    Shutdown,
}

impl KlippyState {
    fn as_str(self) -> &'static str {
        match self {
            KlippyState::Ready => "ready",
            KlippyState::Shutdown => "shutdown",
        }
    }

    fn message(self) -> &'static str {
        match self {
            KlippyState::Ready => "Printer is ready",
            KlippyState::Shutdown => SHUTDOWN_MESSAGE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintState {
    Standby,
    Printing,
    Paused,
    Complete,
    Cancelled,
    Error,
}

impl PrintState {
    pub fn as_str(self) -> &'static str {
        match self {
            PrintState::Standby => "standby",
            PrintState::Printing => "printing",
            PrintState::Paused => "paused",
            PrintState::Complete => "complete",
            PrintState::Cancelled => "cancelled",
            PrintState::Error => "error",
        }
    }

    fn is_active(self) -> bool {
        matches!(self, PrintState::Printing | PrintState::Paused)
    }
}

#[derive(Debug, Clone, Copy)]
struct Heater {
    temperature: f64,
    target: f64,
}

impl Heater {
    fn cold() -> Self {
        Self { temperature: AMBIENT_TEMPERATURE, target: 0.0 }
    }

    fn advance(&mut self, seconds: f64) {
        let goal = self.target.max(AMBIENT_TEMPERATURE);
        let rate = if self.temperature < goal { HEAT_RATE_PER_SECOND } else { COOL_RATE_PER_SECOND };
        self.temperature = step_towards(self.temperature, goal, rate * seconds);
    }

    fn set_target(&mut self, target: f64, maximum: f64) -> Result<(), CommandError> {
        // Like Klipper: 0 switches off, everything else must lie within min_temp..=max_temp.
        if target != 0.0 && !(MIN_HEATER_TEMPERATURE..=maximum).contains(&target) {
            return Err(CommandError::new(format!(
                "Requested temperature ({target:.1}) out of range ({MIN_HEATER_TEMPERATURE:.1}:{maximum:.1})"
            )));
        }
        self.target = target;
        Ok(())
    }

    fn to_json(self) -> Value {
        json!({ "temperature": round(self.temperature, 2), "target": self.target })
    }
}

#[derive(Debug, Clone)]
struct PrintJob {
    estimated_time: f64,
    layer_count: u32,
    filament_total: f64,
}

#[derive(Debug, Clone)]
pub struct Printer {
    time_scale: f64,
    profile: MachineProfile,
    mmu: Option<HappyHare>,
    klippy: KlippyState,
    extruder: Heater,
    bed: Heater,
    chamber: Option<Heater>,
    position: [f64; 4],
    homed: [bool; 3],
    relative_positioning: bool,
    relative_extrusion: bool,
    fan_speed: f64,
    speed_factor: f64,
    extrude_factor: f64,
    pressure_advance: f64,
    z_offset: f64,
    config_pending: bool,
    gantry_leveled: bool,
    print_state: PrintState,
    filename: String,
    message: String,
    print_duration: f64,
    total_duration: f64,
    job: Option<PrintJob>,
    /// Simulated seconds since the last command; drives `idle_timeout.state`.
    inactive_seconds: f64,
}

impl Printer {
    pub fn new(time_scale: f64) -> Self {
        Self::with_profile(time_scale, MachineProfile::default())
    }

    pub fn with_profile(time_scale: f64, profile: MachineProfile) -> Self {
        Self {
            time_scale,
            profile,
            mmu: profile.mmu_gates.map(HappyHare::new),
            klippy: KlippyState::Ready,
            extruder: Heater::cold(),
            bed: Heater::cold(),
            chamber: profile.chamber_heater.then(Heater::cold),
            position: [0.0; 4],
            homed: [false; 3],
            relative_positioning: false,
            relative_extrusion: false,
            fan_speed: 0.0,
            speed_factor: 1.0,
            extrude_factor: 1.0,
            pressure_advance: 0.04,
            z_offset: 0.0,
            config_pending: false,
            gantry_leveled: false,
            print_state: PrintState::Standby,
            filename: String::new(),
            message: String::new(),
            print_duration: 0.0,
            total_duration: 0.0,
            job: None,
            inactive_seconds: BUSY_SECONDS,
        }
    }

    pub fn print_state(&self) -> PrintState {
        self.print_state
    }

    pub fn advance(&mut self, elapsed: Duration) {
        let seconds = elapsed.as_secs_f64() * self.time_scale;
        self.extruder.advance(seconds);
        self.bed.advance(seconds);
        if let Some(chamber) = self.chamber.as_mut() {
            chamber.advance(seconds);
        }
        self.advance_print(seconds);
        self.advance_idle(seconds);
    }

    fn advance_idle(&mut self, seconds: f64) {
        if self.print_state == PrintState::Printing {
            self.inactive_seconds = 0.0;
            return;
        }
        let was_idle = self.inactive_seconds >= IDLE_TIMEOUT_SECONDS;
        self.inactive_seconds += seconds;
        if !was_idle && self.inactive_seconds >= IDLE_TIMEOUT_SECONDS {
            self.switch_heaters_off();
            self.homed = [false; 3];
        }
    }

    fn idle_state(&self) -> &'static str {
        if self.print_state == PrintState::Printing || self.inactive_seconds < BUSY_SECONDS {
            "Printing"
        } else if self.inactive_seconds >= IDLE_TIMEOUT_SECONDS {
            "Idle"
        } else {
            "Ready"
        }
    }

    pub fn apply(&mut self, command: Command) -> Result<(), CommandError> {
        self.ensure_running()?;
        self.inactive_seconds = 0.0;
        match command {
            Command::Home(axes) => self.home(&axes),
            Command::SetExtruderTarget(target) => self.extruder.set_target(target, MAX_EXTRUDER_TARGET)?,
            Command::SetBedTarget(target) => self.bed.set_target(target, MAX_BED_TARGET)?,
            Command::SetFanSpeed(speed) => self.fan_speed = speed,
            Command::AbsolutePositioning => self.relative_positioning = false,
            Command::RelativePositioning => self.relative_positioning = true,
            Command::Move(movement) => self.move_toolhead(movement)?,
            Command::DisableSteppers => self.homed = [false; 3],
            Command::SetGcodeOffset { z, z_adjust } => self.set_z_offset(z, z_adjust),
            Command::AbsoluteExtrusion => self.relative_extrusion = false,
            Command::RelativeExtrusion => self.relative_extrusion = true,
            Command::SetSpeedFactor(factor) => self.speed_factor = factor.max(0.01),
            Command::SetExtrudeFactor(factor) => self.extrude_factor = factor.max(0.01),
            Command::SetPressureAdvance(value) => self.pressure_advance = value.max(0.0),
            Command::SetHeaterTemperature { heater, target } => self.set_named_heater(&heater, target)?,
            Command::SaveConfig => self.config_pending = false,
            Command::QuadGantryLevel => self.level_gantry()?,
            Command::Mmu(command) => self.apply_mmu(command)?,
            Command::Help | Command::Ignored => {}
        }
        Ok(())
    }

    pub fn start_print(&mut self, file: &GcodeFile) -> Result<(), CommandError> {
        self.ensure_running()?;
        if self.print_state.is_active() {
            return Err(CommandError::new("Print job already in progress"));
        }
        self.extruder.target = file.first_layer_extruder_temperature;
        self.bed.target = file.first_layer_bed_temperature;
        self.filename = file.path.clone();
        self.message.clear();
        self.print_duration = 0.0;
        self.total_duration = 0.0;
        self.job = Some(PrintJob {
            estimated_time: file.estimated_time,
            layer_count: file.layer_count,
            filament_total: file.filament_total(),
        });
        self.print_state = PrintState::Printing;
        Ok(())
    }

    pub fn pause_print(&mut self) -> Result<(), CommandError> {
        self.transition(PrintState::Printing, PrintState::Paused)
    }

    pub fn resume_print(&mut self) -> Result<(), CommandError> {
        self.transition(PrintState::Paused, PrintState::Printing)
    }

    pub fn cancel_print(&mut self) -> Result<(), CommandError> {
        self.ensure_running()?;
        if !self.print_state.is_active() {
            return Err(CommandError::new("No active print job"));
        }
        self.print_state = PrintState::Cancelled;
        self.switch_heaters_off();
        Ok(())
    }

    pub fn emergency_stop(&mut self) {
        self.klippy = KlippyState::Shutdown;
        self.switch_heaters_off();
        if self.print_state.is_active() {
            self.print_state = PrintState::Error;
            self.message = SHUTDOWN_MESSAGE.to_owned();
        }
    }

    pub fn restart(&mut self) {
        *self = Self::with_profile(self.time_scale, self.profile);
    }

    pub fn object_names(&self) -> Vec<String> {
        match self.status() {
            Value::Object(objects) => objects.into_iter().map(|(name, _)| name).collect(),
            _ => Vec::new(),
        }
    }

    pub fn status(&self) -> Value {
        let mut objects = self.core_status();
        if self.profile.quad_gantry_level {
            objects.insert("quad_gantry_level".to_owned(), json!({ "applied": self.gantry_leveled }));
        }
        if let Some(mmu) = &self.mmu {
            objects.extend(mmu.status(self.print_state.as_str()));
        }
        if let Some(chamber) = self.chamber {
            objects.insert("heater_generic chamber".to_owned(), chamber.to_json());
        }
        objects.insert("temperature_sensor raspberry_pi".to_owned(), json!({ "temperature": HOST_TEMPERATURE }));
        for (index, name) in FILAMENT_SENSORS.iter().take(self.profile.filament_sensors).enumerate() {
            objects.insert((*name).to_owned(), json!({ "enabled": true, "filament_detected": index != 1 }));
        }
        for name in MACROS {
            objects.insert(format!("gcode_macro {name}"), json!({}));
        }
        Value::Object(objects)
    }

    /// Klipper text for the current state, as returned by `printer.info`.
    pub fn state_message(&self) -> &'static str {
        self.klippy.message()
    }

    pub fn klippy_state(&self) -> &'static str {
        self.klippy.as_str()
    }

    fn set_named_heater(&mut self, heater: &str, target: f64) -> Result<(), CommandError> {
        match (heater, self.chamber.as_mut()) {
            ("extruder", _) => self.extruder.set_target(target, MAX_EXTRUDER_TARGET),
            ("heater_bed", _) => self.bed.set_target(target, MAX_BED_TARGET),
            ("chamber", Some(chamber)) => chamber.set_target(target, MAX_CHAMBER_TARGET),
            _ => Err(CommandError::new(format!("Unknown heater '{heater}'"))),
        }
    }

    fn core_status(&self) -> Map<String, Value> {
        let Value::Object(objects) = json!({
            "webhooks": { "state": self.klippy.as_str(), "state_message": self.klippy.message() },
            "idle_timeout": { "state": self.idle_state(), "printing_time": 0.0 },
            "heater_bed": self.bed.to_json(),
            "extruder": self.extruder_json(),
            "toolhead": self.toolhead_json(),
            "fan": { "speed": self.fan_speed, "rpm": null },
            "gcode_move": {
                "homing_origin": [0.0, 0.0, self.z_offset, 0.0],
                "speed": 1500.0,
                "speed_factor": self.speed_factor,
                "extrude_factor": self.extrude_factor,
            },
            "print_stats": self.print_stats_json(),
            "display_status": { "progress": round(self.progress(), 4), "message": null },
            "motion_report": {
                "live_position": self.rounded_position(),
                "live_velocity": 0.0,
                "live_extruder_velocity": 0.0,
                "steppers": ["stepper_x", "stepper_y", "stepper_z", "extruder"],
                "trapq": ["toolhead", "extruder"],
            },
            "configfile": self.configfile_json(),
        }) else {
            unreachable!("json! object literal");
        };
        objects
    }

    fn configfile_json(&self) -> Value {
        let settings = self.config_sections();
        json!({
            "config": raw_config(&settings),
            "settings": settings,
            "save_config_pending": self.config_pending,
        })
    }

    fn config_sections(&self) -> Map<String, Value> {
        let mut sections = Map::new();
        sections
            .insert("printer".to_owned(), json!({ "kinematics": "corexy", "max_velocity": 300, "max_accel": 3000 }));
        sections.insert(
            "extruder".to_owned(),
            json!({
                "min_extrude_temp": MIN_EXTRUDE_TEMPERATURE,
                "min_temp": MIN_HEATER_TEMPERATURE,
                "max_temp": MAX_EXTRUDER_TARGET
            }),
        );
        sections
            .insert("heater_bed".to_owned(), json!({ "min_temp": MIN_HEATER_TEMPERATURE, "max_temp": MAX_BED_TARGET }));
        sections.insert("fan".to_owned(), json!({}));
        if self.chamber.is_some() {
            sections.insert(
                "heater_generic chamber".to_owned(),
                json!({ "min_temp": MIN_HEATER_TEMPERATURE, "max_temp": MAX_CHAMBER_TARGET }),
            );
        }
        if self.profile.quad_gantry_level {
            sections.insert("quad_gantry_level".to_owned(), json!({ "max_adjust": 10 }));
        }
        if let Some(mmu) = &self.mmu {
            sections.extend(mmu.config_sections());
        }
        sections
    }

    fn apply_mmu(&mut self, command: MmuCommand) -> Result<(), CommandError> {
        match self.mmu.as_mut() {
            Some(mmu) => mmu.apply(command),
            None => Err(CommandError::new(format!("Unknown command:\"{}\"", command.gcode_name()))),
        }
    }

    fn ensure_running(&self) -> Result<(), CommandError> {
        match self.klippy {
            KlippyState::Ready => Ok(()),
            KlippyState::Shutdown => Err(CommandError::new("Klipper reports shutdown, restart required")),
        }
    }

    fn transition(&mut self, from: PrintState, to: PrintState) -> Result<(), CommandError> {
        self.ensure_running()?;
        if self.print_state != from {
            return Err(CommandError::new(format!("Print job is not {}", from.as_str())));
        }
        self.print_state = to;
        Ok(())
    }

    fn switch_heaters_off(&mut self) {
        self.extruder.target = 0.0;
        self.bed.target = 0.0;
    }

    fn advance_print(&mut self, seconds: f64) {
        match self.print_state {
            PrintState::Printing => {
                self.total_duration += seconds;
                self.print_duration += seconds;
                if self.progress() >= 1.0 {
                    self.print_state = PrintState::Complete;
                    self.switch_heaters_off();
                }
            }
            PrintState::Paused => self.total_duration += seconds,
            _ => {}
        }
    }

    fn home(&mut self, axes: &[Axis]) {
        let axes = if axes.is_empty() { &ALL_AXES[..] } else { axes };
        for axis in axes {
            self.homed[axis.index()] = true;
            self.position[axis.index()] = AXIS_MINIMUM[axis.index()];
        }
    }

    fn move_toolhead(&mut self, movement: Move) -> Result<(), CommandError> {
        let mut target = self.position;
        for (axis, value) in movement.linear_axes() {
            let index = axis.index();
            if !self.homed[index] {
                return Err(CommandError::new("Must home axis first"));
            }
            let position = self.resolve(self.position[index], value);
            if !(AXIS_MINIMUM[index]..=AXIS_MAXIMUM[index]).contains(&position) {
                return Err(CommandError::new("Move out of range"));
            }
            target[index] = position;
        }
        if let Some(distance) = movement.e {
            if self.extruder.temperature < MIN_EXTRUDE_TEMPERATURE {
                return Err(CommandError::new("Extrude below minimum temp"));
            }
            target[EXTRUDER_INDEX] = if self.relative_positioning || self.relative_extrusion {
                self.position[EXTRUDER_INDEX] + distance
            } else {
                distance
            };
        }
        self.position = target;
        Ok(())
    }

    fn resolve(&self, current: f64, value: f64) -> f64 {
        if self.relative_positioning {
            current + value
        } else {
            value
        }
    }

    fn set_z_offset(&mut self, z: Option<f64>, z_adjust: Option<f64>) {
        if let Some(z) = z {
            self.z_offset = z;
        }
        if let Some(adjustment) = z_adjust {
            self.z_offset += adjustment;
        }
    }

    fn level_gantry(&mut self) -> Result<(), CommandError> {
        if !self.homed.iter().all(|homed| *homed) {
            return Err(CommandError::new("Must home axes first"));
        }
        self.gantry_leveled = true;
        Ok(())
    }

    fn progress(&self) -> f64 {
        self.job.as_ref().map_or(0.0, |job| (self.print_duration / job.estimated_time).min(1.0))
    }

    fn filament_used(&self) -> f64 {
        self.job.as_ref().map_or(0.0, |job| job.filament_total * self.progress())
    }

    fn layers(&self) -> (Option<u32>, Option<u32>) {
        match &self.job {
            Some(job) => {
                let layer = (self.progress() * f64::from(job.layer_count)).floor() as u32 + 1;
                (Some(layer.min(job.layer_count)), Some(job.layer_count))
            }
            None => (None, None),
        }
    }

    fn homed_axes(&self) -> String {
        ALL_AXES.iter().filter(|axis| self.homed[axis.index()]).map(|axis| axis.letter()).collect()
    }

    fn rounded_position(&self) -> [f64; 4] {
        self.position.map(|value| round(value, 3))
    }

    fn extruder_json(&self) -> Value {
        json!({
            "temperature": round(self.extruder.temperature, 2),
            "target": self.extruder.target,
            "power": if self.extruder.target > self.extruder.temperature { 1.0 } else { 0.0 },
            "pressure_advance": self.pressure_advance,
            "smooth_time": 0.04,
            "can_extrude": self.extruder.temperature >= MIN_EXTRUDE_TEMPERATURE,
        })
    }

    fn toolhead_json(&self) -> Value {
        json!({
            "position": self.rounded_position(),
            "homed_axes": self.homed_axes(),
            "max_accel": 3000.0,
            "axis_minimum": AXIS_MINIMUM,
            "axis_maximum": AXIS_MAXIMUM,
            "max_velocity": 300.0,
            "square_corner_velocity": 5.0,
            "minimum_cruise_ratio": 0.5,
        })
    }

    fn print_stats_json(&self) -> Value {
        let (current_layer, total_layer) = self.layers();
        json!({
            "filename": self.filename,
            "total_duration": round(self.total_duration, 2),
            "print_duration": round(self.print_duration, 2),
            "filament_used": round(self.filament_used(), 2),
            "state": self.print_state.as_str(),
            "message": self.message,
            "info": { "current_layer": current_layer, "total_layer": total_layer },
        })
    }
}

fn raw_config(settings: &Map<String, Value>) -> Map<String, Value> {
    settings
        .iter()
        .map(|(section, options)| {
            let raw_options: Map<String, Value> = options.as_object().map_or_else(Map::new, |options| {
                options.iter().map(|(name, value)| (name.clone(), Value::String(raw_value(value)))).collect()
            });
            (section.clone(), Value::Object(raw_options))
        })
        .collect()
}

fn raw_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn step_towards(current: f64, goal: f64, max_step: f64) -> f64 {
    if (goal - current).abs() <= max_step {
        goal
    } else {
        current + max_step * (goal - current).signum()
    }
}

fn round(value: f64, decimals: i32) -> f64 {
    let factor = 10f64.powi(decimals);
    (value * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::Catalog;
    use crate::gcode::parse_script;

    fn run(printer: &mut Printer, script: &str) -> Result<(), CommandError> {
        parse_script(script)?.into_iter().try_for_each(|command| printer.apply(command))
    }

    fn printer() -> Printer {
        Printer::new(1.0)
    }

    #[test]
    fn extruder_heats_up_to_target_and_stops() {
        let mut printer = printer();
        run(&mut printer, "M104 S60").unwrap();

        printer.advance(Duration::from_secs(5));
        assert_eq!(printer.status()["extruder"]["temperature"], 39.0);

        printer.advance(Duration::from_secs(60));
        assert_eq!(printer.status()["extruder"]["temperature"], 60.0);
    }

    #[test]
    fn cools_down_to_ambient_temperature() {
        let mut printer = printer();
        run(&mut printer, "M140 S40").unwrap();
        printer.advance(Duration::from_secs(60));
        run(&mut printer, "M140 S0").unwrap();

        printer.advance(Duration::from_secs(3600));

        assert_eq!(printer.status()["heater_bed"]["temperature"], 24.0);
    }

    #[test]
    fn rejects_temperature_above_limit() {
        let error = run(&mut printer(), "M104 S350").unwrap_err();

        assert_eq!(error.message(), "Requested temperature (350.0) out of range (10.0:300.0)");
    }

    #[test]
    fn idle_timeout_goes_from_busy_to_ready_to_idle_and_switches_off() {
        let mut printer = printer();
        assert_eq!(printer.status()["idle_timeout"]["state"], "Ready");

        run(&mut printer, "M104 S200\nG28").unwrap();
        assert_eq!(printer.status()["idle_timeout"]["state"], "Printing");

        printer.advance(Duration::from_secs(10));
        assert_eq!(printer.status()["idle_timeout"]["state"], "Ready");

        printer.advance(Duration::from_secs(600));
        let status = printer.status();
        assert_eq!(status["idle_timeout"]["state"], "Idle");
        assert_eq!(status["extruder"]["target"], 0.0);
        assert_eq!(status["toolhead"]["homed_axes"], "");
    }

    #[test]
    fn moving_requires_homing() {
        let error = run(&mut printer(), "G90\nG0 X10").unwrap_err();

        assert_eq!(error.message(), "Must home axis first");
    }

    #[test]
    fn moves_after_homing_and_reports_homed_axes() {
        let mut printer = printer();

        run(&mut printer, "G28 X Y\nG90\nG0 X10 Y20").unwrap();

        let status = printer.status();
        assert_eq!(status["toolhead"]["homed_axes"], "xy");
        assert_eq!(status["toolhead"]["position"], json!([10.0, 20.0, 0.0, 0.0]));
    }

    #[test]
    fn rejects_moves_out_of_range() {
        let mut printer = printer();
        run(&mut printer, "G28").unwrap();

        let error = run(&mut printer, "G0 X500").unwrap_err();

        assert_eq!(error.message(), "Move out of range");
    }

    #[test]
    fn extrusion_needs_a_hot_nozzle() {
        let mut printer = printer();
        assert!(run(&mut printer, "G91\nG1 E10").is_err());

        run(&mut printer, "M104 S200").unwrap();
        printer.advance(Duration::from_secs(120));

        run(&mut printer, "G1 E10").unwrap();
        assert_eq!(printer.status()["extruder"]["can_extrude"], true);
    }

    #[test]
    fn gcode_offset_adjusts_homing_origin() {
        let mut printer = printer();

        run(&mut printer, "SET_GCODE_OFFSET Z_ADJUST=0.05\nSET_GCODE_OFFSET Z_ADJUST=0.05").unwrap();

        assert_eq!(printer.status()["gcode_move"]["homing_origin"], json!([0.0, 0.0, 0.1, 0.0]));
    }

    #[test]
    fn print_runs_to_completion_and_switches_heaters_off() {
        let mut printer = printer();
        let file = Catalog::demo().find("calibration_cube.gcode").unwrap().clone();

        printer.start_print(&file).unwrap();
        assert_eq!(printer.print_state(), PrintState::Printing);
        assert_eq!(printer.status()["extruder"]["target"], 210.0);

        printer.advance(Duration::from_secs(750));
        assert_eq!(printer.status()["display_status"]["progress"], 0.5);
        assert_eq!(printer.status()["print_stats"]["info"]["current_layer"], 51);

        printer.advance(Duration::from_secs(750));
        assert_eq!(printer.print_state(), PrintState::Complete);
        assert_eq!(printer.status()["extruder"]["target"], 0.0);
    }

    #[test]
    fn paused_print_does_not_progress() {
        let mut printer = printer();
        let file = Catalog::demo().find("benchy.gcode").unwrap().clone();
        printer.start_print(&file).unwrap();

        printer.pause_print().unwrap();
        printer.advance(Duration::from_secs(600));

        let status = printer.status();
        assert_eq!(status["print_stats"]["state"], "paused");
        assert_eq!(status["print_stats"]["print_duration"], 0.0);
        assert_eq!(status["print_stats"]["total_duration"], 600.0);
    }

    #[test]
    fn cancel_requires_an_active_print() {
        let mut printer = printer();

        assert!(printer.cancel_print().is_err());
    }

    #[test]
    fn rejects_second_print_while_one_is_running() {
        let mut printer = printer();
        let file = Catalog::demo().find("benchy.gcode").unwrap().clone();
        printer.start_print(&file).unwrap();

        let error = printer.start_print(&file).unwrap_err();

        assert_eq!(error.message(), "Print job already in progress");
    }

    #[test]
    fn emergency_stop_shuts_down_until_restart() {
        let mut printer = printer();
        let file = Catalog::demo().find("benchy.gcode").unwrap().clone();
        printer.start_print(&file).unwrap();

        printer.emergency_stop();

        assert_eq!(printer.status()["webhooks"]["state"], "shutdown");
        assert_eq!(printer.print_state(), PrintState::Error);
        assert!(run(&mut printer, "G28").is_err());

        printer.restart();
        assert_eq!(printer.status()["webhooks"]["state"], "ready");
        assert!(run(&mut printer, "G28").is_ok());
    }

    fn names(printer: &Printer) -> Vec<String> {
        printer.object_names()
    }

    #[test]
    fn basic_profile_has_no_optional_modules() {
        let printer = Printer::with_profile(1.0, MachineProfile::basic());

        let names = names(&printer);

        assert!(names.contains(&"extruder".to_owned()));
        assert!(!names.contains(&"quad_gantry_level".to_owned()));
        assert!(!names.iter().any(|name| name.starts_with("mmu")));
        assert!(printer.status()["configfile"]["settings"].get("quad_gantry_level").is_none());
    }

    #[test]
    fn voron_profile_adds_quad_gantry_level() {
        let printer = Printer::with_profile(1.0, MachineProfile::voron());

        assert!(names(&printer).contains(&"quad_gantry_level".to_owned()));
        assert_eq!(printer.status()["configfile"]["settings"]["quad_gantry_level"]["max_adjust"], 10);
    }

    #[test]
    fn happy_hare_profile_registers_mmu_objects_and_config() {
        let printer = Printer::with_profile(1.0, MachineProfile::happy_hare(6));

        let status = printer.status();

        assert!(names(&printer).contains(&"mmu".to_owned()));
        assert_eq!(status["mmu"]["num_gates"], 6);
        assert_eq!(status["mmu_machine"]["num_gates"], 6);
        assert!(status["configfile"]["settings"].get("mmu_machine").is_some());
    }

    #[test]
    fn raw_config_holds_the_same_sections_as_strings() {
        let status = printer().status();

        assert_eq!(status["configfile"]["settings"]["extruder"]["max_temp"], 300.0);
        assert_eq!(status["configfile"]["config"]["extruder"]["max_temp"], "300.0");
    }

    #[test]
    fn mmu_commands_are_unknown_without_happy_hare() {
        let error = run(&mut printer(), "MMU_HOME").unwrap_err();

        assert_eq!(error.message(), "Unknown command:\"MMU_HOME\"");
    }

    #[test]
    fn tool_change_runs_through_the_printer_with_happy_hare() {
        let mut printer = Printer::with_profile(1.0, MachineProfile::happy_hare(4));

        run(&mut printer, "MMU_HOME\nT1").unwrap();

        let status = printer.status();
        assert_eq!(status["mmu"]["tool"], 1);
        assert_eq!(status["mmu"]["filament"], "Loaded");
    }

    #[test]
    fn restart_keeps_the_machine_profile() {
        let mut printer = Printer::with_profile(1.0, MachineProfile::happy_hare(4));
        run(&mut printer, "MMU_HOME").unwrap();

        printer.restart();

        assert_eq!(printer.status()["mmu"]["is_homed"], false);
    }
}
