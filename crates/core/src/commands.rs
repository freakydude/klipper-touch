//! Operator commands for the printer and their translation into Moonraker calls.

use crate::printer::{Axis, Motion};
use serde_json::{json, Value};

/// A command triggered by a person. The UI only knows this level, not G-code or JSON-RPC.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    // Print job
    StartPrint {
        filename: String,
    },
    PausePrint,
    ResumePrint,
    CancelPrint,
    // Motion
    Home {
        x: bool,
        y: bool,
        z: bool,
    },
    /// Relative move; the target is clamped to the axis range.
    Jog {
        axis: Axis,
        distance: f64,
    },
    DisableMotors,
    QuadGantryLevel,
    ZTiltAdjust,
    // Temperatures and fans
    SetHeaterTarget {
        heater: String,
        target: f64,
    },
    SetPartFanSpeed {
        fraction: f64,
    },
    // Tuning
    AdjustZOffset {
        delta: f64,
    },
    ResetZOffset,
    /// Applies the babystep offset to the probe or endstop position (followed by `SaveConfig`).
    ApplyZOffset {
        to_probe: bool,
    },
    SetSpeedFactor {
        percent: f64,
    },
    SetExtrudeFactor {
        percent: f64,
    },
    SetPressureAdvance {
        value: f64,
    },
    SaveConfig,
    // Extruder
    Extrude {
        length: f64,
        speed: f64,
    },
    // Free-form commands
    RunMacro {
        name: String,
    },
    RunGcode {
        script: String,
    },
    // Klipper and host
    EmergencyStop,
    RestartKlipper,
    FirmwareRestart,
    SetPowerDevice {
        device: String,
        on: bool,
    },
    RestartService {
        service: String,
    },
    RebootHost,
    ShutdownHost,
    RefreshUpdates,
    Upgrade {
        name: String,
    },
}

/// A concrete Moonraker call.
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub method: &'static str,
    pub params: Option<Value>,
}

impl Call {
    fn new(method: &'static str) -> Self {
        Self { method, params: None }
    }

    fn with(method: &'static str, params: Value) -> Self {
        Self { method, params: Some(params) }
    }

    fn gcode(script: impl Into<String>) -> Self {
        Self::with("printer.gcode.script", json!({ "script": script.into() }))
    }
}

/// Fixed speeds for manual moves in mm/s.
const JOG_SPEED_XY: f64 = 100.0;
const JOG_SPEED_Z: f64 = 10.0;

impl Command {
    /// Translates the command; `motion` provides the current position for clamped moves.
    /// `None` if the command would have no effect in the current state.
    pub fn to_call(&self, motion: &Motion) -> Option<Call> {
        let call = match self {
            Self::StartPrint { filename } => Call::with("printer.print.start", json!({ "filename": filename })),
            Self::PausePrint => Call::new("printer.print.pause"),
            Self::ResumePrint => Call::new("printer.print.resume"),
            Self::CancelPrint => Call::new("printer.print.cancel"),
            Self::Home { x, y, z } => Call::gcode(home_gcode(*x, *y, *z)),
            Self::Jog { axis, distance } => Call::gcode(jog_gcode(motion, *axis, *distance)?),
            Self::DisableMotors => Call::gcode("M84"),
            Self::QuadGantryLevel => Call::gcode("QUAD_GANTRY_LEVEL"),
            Self::ZTiltAdjust => Call::gcode("Z_TILT_ADJUST"),
            Self::SetHeaterTarget { heater, target } => Call::gcode(heater_gcode(heater, target.max(0.0))),
            Self::SetPartFanSpeed { fraction } => {
                Call::gcode(format!("M106 S{}", (fraction.clamp(0.0, 1.0) * 255.0).round()))
            }
            Self::AdjustZOffset { delta } => {
                Call::gcode(format!("SET_GCODE_OFFSET Z_ADJUST={} MOVE=1", number(*delta)))
            }
            Self::ResetZOffset => Call::gcode("SET_GCODE_OFFSET Z=0 MOVE=1"),
            Self::ApplyZOffset { to_probe: true } => Call::gcode("Z_OFFSET_APPLY_PROBE"),
            Self::ApplyZOffset { to_probe: false } => Call::gcode("Z_OFFSET_APPLY_ENDSTOP"),
            Self::SetSpeedFactor { percent } => Call::gcode(format!("M220 S{}", percent.max(1.0).round())),
            Self::SetExtrudeFactor { percent } => Call::gcode(format!("M221 S{}", percent.max(1.0).round())),
            Self::SetPressureAdvance { value } => {
                Call::gcode(format!("SET_PRESSURE_ADVANCE ADVANCE={}", number(value.max(0.0))))
            }
            Self::SaveConfig => Call::gcode("SAVE_CONFIG"),
            Self::Extrude { length, speed } => {
                Call::gcode(format!("M83\nG1 E{} F{}", number(*length), number(speed.abs() * 60.0)))
            }
            Self::RunMacro { name } => Call::gcode(name.clone()),
            Self::RunGcode { script } => Call::gcode(script.clone()),
            Self::EmergencyStop => Call::new("printer.emergency_stop"),
            Self::RestartKlipper => Call::new("printer.restart"),
            Self::FirmwareRestart => Call::new("printer.firmware_restart"),
            Self::SetPowerDevice { device, on } => Call::with(
                "machine.device_power.post_device",
                json!({ "device": device, "action": if *on { "on" } else { "off" } }),
            ),
            Self::RestartService { service } => Call::with("machine.services.restart", json!({ "service": service })),
            Self::RebootHost => Call::new("machine.reboot"),
            Self::ShutdownHost => Call::new("machine.shutdown"),
            Self::RefreshUpdates => Call::new("machine.update.refresh"),
            Self::Upgrade { name } => Call::with("machine.update.upgrade", json!({ "name": name })),
        };
        Some(call)
    }

    /// Short name for feedback in the UI.
    pub fn describe(&self) -> String {
        match self {
            Self::RunMacro { name } => name.clone(),
            Self::RunGcode { script } => script.lines().next().unwrap_or_default().to_owned(),
            other => format!("{other:?}").split([' ', '{']).next().unwrap_or_default().to_owned(),
        }
    }
}

fn home_gcode(x: bool, y: bool, z: bool) -> String {
    if x && y && z || !(x || y || z) {
        return "G28".to_owned();
    }
    let axes: Vec<&str> = [(x, "X"), (y, "Y"), (z, "Z")]
        .into_iter()
        .filter_map(|(selected, letter)| selected.then_some(letter))
        .collect();
    format!("G28 {}", axes.join(" "))
}

fn jog_gcode(motion: &Motion, axis: Axis, distance: f64) -> Option<String> {
    let index = axis.index();
    let current = motion.position[index];
    let (minimum, maximum) = (motion.axis_minimum[index], motion.axis_maximum[index]);
    let target = if maximum > minimum { (current + distance).clamp(minimum, maximum) } else { current + distance };
    if (target - current).abs() < 1e-6 {
        return None;
    }
    let speed = if axis == Axis::Z { JOG_SPEED_Z } else { JOG_SPEED_XY };
    Some(format!("G90\nG1 {}{} F{}", axis.letter(), number(target), number(speed * 60.0)))
}

/// G-code for a target temperature, matching the kind of heater.
fn heater_gcode(heater: &str, target: f64) -> String {
    let target = number(target);
    match heater.split_once(' ') {
        None if heater == "extruder" => format!("M104 S{target}"),
        None if heater == "heater_bed" => format!("M140 S{target}"),
        Some(("temperature_fan", name)) => format!("SET_TEMPERATURE_FAN_TARGET TEMPERATURE_FAN={name} TARGET={target}"),
        Some((_, name)) => format!("SET_HEATER_TEMPERATURE HEATER={name} TARGET={target}"),
        None => format!("SET_HEATER_TEMPERATURE HEATER={heater} TARGET={target}"),
    }
}

/// Number without redundant decimal places, the way Klipper likes it.
fn number(value: f64) -> String {
    let rounded = (value * 10_000.0).round() / 10_000.0;
    let text = format!("{rounded:.4}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn motion_at(position: [f64; 4]) -> Motion {
        Motion {
            position,
            axis_minimum: [0.0, 0.0, -2.0, 0.0],
            axis_maximum: [300.0, 300.0, 250.0, 0.0],
            ..Motion::default()
        }
    }

    fn script(command: Command) -> String {
        let call = command.to_call(&motion_at([10.0, 10.0, 5.0, 0.0])).unwrap();
        assert_eq!(call.method, "printer.gcode.script");
        call.params.unwrap()["script"].as_str().unwrap().to_owned()
    }

    #[test]
    fn homing_selects_axes() {
        assert_eq!(script(Command::Home { x: true, y: true, z: false }), "G28 X Y");
        assert_eq!(script(Command::Home { x: true, y: true, z: true }), "G28");
        assert_eq!(script(Command::Home { x: false, y: false, z: true }), "G28 Z");
    }

    #[test]
    fn jog_is_clamped_to_axis_limits() {
        assert_eq!(script(Command::Jog { axis: Axis::X, distance: -50.0 }), "G90\nG1 X0 F6000");
        assert_eq!(script(Command::Jog { axis: Axis::Z, distance: 0.1 }), "G90\nG1 Z5.1 F600");
        let at_minimum = motion_at([0.0, 0.0, 0.0, 0.0]);
        assert_eq!(Command::Jog { axis: Axis::X, distance: -1.0 }.to_call(&at_minimum), None);
    }

    #[test]
    fn heater_targets_use_matching_gcode() {
        assert_eq!(script(Command::SetHeaterTarget { heater: "extruder".into(), target: 210.0 }), "M104 S210");
        assert_eq!(script(Command::SetHeaterTarget { heater: "heater_bed".into(), target: -5.0 }), "M140 S0");
        assert_eq!(
            script(Command::SetHeaterTarget { heater: "heater_generic chamber".into(), target: 45.5 }),
            "SET_HEATER_TEMPERATURE HEATER=chamber TARGET=45.5"
        );
        assert_eq!(
            script(Command::SetHeaterTarget { heater: "temperature_fan nevermore".into(), target: 40.0 }),
            "SET_TEMPERATURE_FAN_TARGET TEMPERATURE_FAN=nevermore TARGET=40"
        );
        assert_eq!(
            script(Command::SetHeaterTarget { heater: "extruder1".into(), target: 200.0 }),
            "SET_HEATER_TEMPERATURE HEATER=extruder1 TARGET=200"
        );
    }

    #[test]
    fn tuning_and_extrusion_commands() {
        assert_eq!(script(Command::AdjustZOffset { delta: -0.005 }), "SET_GCODE_OFFSET Z_ADJUST=-0.005 MOVE=1");
        assert_eq!(script(Command::SetPartFanSpeed { fraction: 0.5 }), "M106 S128");
        assert_eq!(script(Command::SetSpeedFactor { percent: 110.0 }), "M220 S110");
        assert_eq!(script(Command::SetPressureAdvance { value: 0.0425 }), "SET_PRESSURE_ADVANCE ADVANCE=0.0425");
        assert_eq!(script(Command::Extrude { length: -10.0, speed: 5.0 }), "M83\nG1 E-10 F300");
    }

    #[test]
    fn rpc_commands_carry_parameters() {
        let motion = Motion::default();
        let start = Command::StartPrint { filename: "a/b.gcode".into() }.to_call(&motion).unwrap();
        let power = Command::SetPowerDevice { device: "printer".into(), on: false }.to_call(&motion).unwrap();

        assert_eq!(start, Call::with("printer.print.start", json!({ "filename": "a/b.gcode" })));
        assert_eq!(power.params.unwrap(), json!({ "device": "printer", "action": "off" }));
        assert_eq!(Command::EmergencyStop.to_call(&motion).unwrap().method, "printer.emergency_stop");
    }

    #[test]
    fn describes_commands_briefly() {
        assert_eq!(Command::PausePrint.describe(), "PausePrint");
        assert_eq!(Command::Jog { axis: Axis::X, distance: 1.0 }.describe(), "Jog");
        assert_eq!(Command::RunMacro { name: "PARK".into() }.describe(), "PARK");
    }
}
