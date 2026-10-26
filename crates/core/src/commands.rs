//! Operator commands for the printer and their translation into Moonraker calls.

use crate::printer::{Axis, Capabilities, FilamentHandling, Motion, PrinterStatus};
use serde_json::{json, Value};

/// A command triggered by a person. The UI only knows this level, not G-code or JSON-RPC.
///
/// Some commands state an intent (`Level`, `ApplyPreset`, `CoolDown`, `StepHeater`, `ApplyZOffset`, `LoadFilament`,
/// `UnloadFilament`); [`Command::resolve`] turns them into the commands this printer needs. Only those reach
/// [`Command::to_call`].
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
    /// Levels with what the printer has: quad gantry level, else Z tilt.
    Level,
    QuadGantryLevel,
    ZTiltAdjust,
    // Temperatures and fans
    SetHeaterTarget {
        heater: String,
        target: f64,
    },
    /// Raises or lowers the target by `step` (with sign) from the target shown now, see [`crate::printer::Heater::stepped_target`].
    StepHeater {
        heater: String,
        step: f64,
    },
    /// Heats the nozzle and, if there is one, the bed.
    ApplyPreset {
        extruder: f64,
        bed: f64,
    },
    /// Switches every heater off.
    CoolDown,
    SetPartFanSpeed {
        fraction: f64,
    },
    // Tuning
    AdjustZOffset {
        delta: f64,
    },
    ResetZOffset,
    /// Applies the babystep offset to the probe if there is one, else to the endstop (followed by `SaveConfig`).
    ApplyZOffset,
    ApplyZOffsetToProbe,
    ApplyZOffsetToEndstop,
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
    /// Loads or unloads with what the printer has: Happy Hare, else the macros `LOAD_FILAMENT`/`UNLOAD_FILAMENT`.
    LoadFilament,
    UnloadFilament,
    // Free-form commands
    RunMacro {
        name: String,
    },
    /// A blank script is ignored.
    RunGcode {
        script: String,
    },
    /// Skips an object of the running print (Klipper `exclude_object`).
    ExcludeObject {
        name: String,
    },
    // Happy Hare. `MMU_LOAD` and `MMU_UNLOAD` act on the selected gate; they have no gate parameter.
    MmuSelectGate {
        gate: usize,
    },
    MmuLoad,
    MmuUnload,
    /// Ejects the filament of a gate completely; without `gate` the selected one.
    MmuEject {
        gate: Option<usize>,
    },
    MmuHome,
    MmuRecover,
    MmuUnlock,
    /// Puts a Spoolman spool into a gate; `None` takes the spool out of the record.
    AssignGateSpool {
        gate: usize,
        spool_id: Option<u32>,
    },
    // Spoolman
    /// The spool Moonraker tracks the filament use of; `None` for none.
    SetActiveSpool {
        spool_id: Option<u32>,
    },
    /// Sets the weight of a spool from a measurement; `weight` is the gross weight of the spool in grams.
    MeasureSpool {
        spool_id: u32,
        weight: f64,
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

/// Why an intent cannot be carried out on this printer.
const NO_LEVELING: &str = "this printer has neither quad gantry level nor Z tilt";
const NO_LOAD_WAY: &str = "no way to load filament: no Happy Hare and no LOAD_FILAMENT macro";
const NO_UNLOAD_WAY: &str = "no way to unload filament: no Happy Hare and no UNLOAD_FILAMENT macro";

impl Command {
    /// Turns an intent into the commands this printer needs; every other command stays as it is. `status` shows the
    /// heater targets including those sent but not yet reported, so steps build on each other.
    pub fn resolve(self, capabilities: &Capabilities, status: &PrinterStatus) -> Result<Vec<Command>, String> {
        let heater_target = |heater: &str, target: f64| Command::SetHeaterTarget { heater: heater.to_owned(), target };
        let commands = match self {
            Self::Level if capabilities.quad_gantry_level => vec![Self::QuadGantryLevel],
            Self::Level if capabilities.z_tilt => vec![Self::ZTiltAdjust],
            Self::Level => return Err(NO_LEVELING.to_owned()),
            Self::ApplyZOffset if capabilities.probe => vec![Self::ApplyZOffsetToProbe],
            Self::ApplyZOffset => vec![Self::ApplyZOffsetToEndstop],
            Self::ApplyPreset { extruder, bed } => {
                let mut commands = vec![heater_target("extruder", extruder)];
                if capabilities.heater_bed {
                    commands.push(heater_target("heater_bed", bed));
                }
                commands
            }
            Self::CoolDown => status.heaters.iter().map(|heater| heater_target(&heater.object, 0.0)).collect(),
            Self::StepHeater { heater, step } => {
                let current = status.heater(&heater).ok_or_else(|| format!("unknown heater {heater}"))?;
                vec![heater_target(&heater, current.stepped_target(step))]
            }
            Self::LoadFilament => match capabilities.filament_handling() {
                FilamentHandling::Mmu => vec![Self::MmuLoad],
                FilamentHandling::Macros { load: true, .. } => vec![Self::RunMacro { name: "LOAD_FILAMENT".into() }],
                _ => return Err(NO_LOAD_WAY.to_owned()),
            },
            Self::UnloadFilament => match capabilities.filament_handling() {
                FilamentHandling::Mmu => vec![Self::MmuUnload],
                FilamentHandling::Macros { unload: true, .. } => {
                    vec![Self::RunMacro { name: "UNLOAD_FILAMENT".into() }]
                }
                _ => return Err(NO_UNLOAD_WAY.to_owned()),
            },
            Self::RunGcode { script } => match script.trim() {
                "" => Vec::new(),
                trimmed => vec![Self::RunGcode { script: trimmed.to_owned() }],
            },
            other => vec![other],
        };
        Ok(commands)
    }

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
            Self::ApplyZOffsetToProbe => Call::gcode("Z_OFFSET_APPLY_PROBE"),
            Self::ApplyZOffsetToEndstop => Call::gcode("Z_OFFSET_APPLY_ENDSTOP"),
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
            Self::ExcludeObject { name } => Call::gcode(exclude_object_gcode(name)?),
            Self::MmuSelectGate { gate } => Call::gcode(format!("MMU_SELECT GATE={gate}")),
            Self::MmuLoad => Call::gcode("MMU_LOAD"),
            Self::MmuUnload => Call::gcode("MMU_UNLOAD"),
            Self::MmuEject { gate: Some(gate) } => Call::gcode(format!("MMU_EJECT GATE={gate}")),
            Self::MmuEject { gate: None } => Call::gcode("MMU_EJECT"),
            Self::MmuHome => Call::gcode("MMU_HOME"),
            Self::MmuRecover => Call::gcode("MMU_RECOVER"),
            Self::MmuUnlock => Call::gcode("MMU_UNLOCK"),
            Self::AssignGateSpool { gate, spool_id } => {
                Call::gcode(format!("MMU_GATE_MAP GATE={gate} SPOOLID={}", spool_id.map_or(-1, i64::from)))
            }
            Self::SetActiveSpool { spool_id } => {
                Call::with("server.spoolman.post_spool_id", json!({ "spool_id": spool_id }))
            }
            Self::MeasureSpool { spool_id, weight } => Call::with(
                "server.spoolman.proxy",
                json!({
                    "request_method": "PUT",
                    "path": format!("/v1/spool/{spool_id}/measure"),
                    "body": { "weight": weight },
                    "use_v2_response": true
                }),
            ),
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
            // Intents are resolved before; none of them is a call of its own.
            Self::Level
            | Self::StepHeater { .. }
            | Self::ApplyPreset { .. }
            | Self::CoolDown
            | Self::ApplyZOffset
            | Self::LoadFilament
            | Self::UnloadFilament => return None,
        };
        Some(call)
    }

    /// The command changes what Spoolman knows, so the spools have to be read again when it succeeded.
    pub fn changes_spools(&self) -> bool {
        matches!(self, Self::SetActiveSpool { .. } | Self::MeasureSpool { .. } | Self::AssignGateSpool { .. })
    }

    /// Short name for feedback in the UI.
    pub fn describe(&self) -> String {
        match self {
            Self::RunMacro { name } => name.clone(),
            Self::RunGcode { script } => script.lines().next().unwrap_or_default().to_owned(),
            other => format!("{other:?}").split([' ', '{']).next().unwrap_or_default().to_owned(),
        }
    }

    /// Restarts, reboots and updates end a running print; they are refused while a job is active.
    pub fn interrupts_print(&self) -> bool {
        matches!(
            self,
            Self::RestartKlipper
                | Self::FirmwareRestart
                | Self::RestartService { .. }
                | Self::RebootHost
                | Self::ShutdownHost
                | Self::Upgrade { .. }
        )
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

/// `EXCLUDE_OBJECT` reads the name up to the next space, so names with spaces cannot be excluded.
fn exclude_object_gcode(name: &str) -> Option<String> {
    (!name.is_empty() && !name.contains(char::is_whitespace)).then(|| format!("EXCLUDE_OBJECT NAME={name}"))
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
    fn happy_hare_commands_use_the_documented_parameters() {
        assert_eq!(script(Command::MmuSelectGate { gate: 3 }), "MMU_SELECT GATE=3");
        assert_eq!(script(Command::MmuLoad), "MMU_LOAD");
        assert_eq!(script(Command::MmuUnload), "MMU_UNLOAD");
        assert_eq!(script(Command::MmuEject { gate: None }), "MMU_EJECT");
        assert_eq!(script(Command::MmuEject { gate: Some(1) }), "MMU_EJECT GATE=1");
        assert_eq!(script(Command::MmuHome), "MMU_HOME");
        assert_eq!(script(Command::MmuRecover), "MMU_RECOVER");
        assert_eq!(script(Command::MmuUnlock), "MMU_UNLOCK");
    }

    #[test]
    fn exclude_object_names_the_object() {
        let exclude = |name: &str| Command::ExcludeObject { name: name.into() };
        assert_eq!(script(exclude("Benchy_0")), "EXCLUDE_OBJECT NAME=Benchy_0");
        assert_eq!(exclude("Part 2").to_call(&motion_at([0.0; 4])), None);
        assert_eq!(exclude("").to_call(&motion_at([0.0; 4])), None);
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

    fn printer(objects: &[&str]) -> Capabilities {
        Capabilities::from_objects(objects.iter().copied())
    }

    fn status_with_targets(targets: &[(&str, f64)]) -> PrinterStatus {
        PrinterStatus {
            heaters: targets
                .iter()
                .map(|(object, target)| crate::printer::Heater {
                    object: (*object).to_owned(),
                    target: *target,
                    min_temp: 10.0,
                    ..crate::printer::Heater::default()
                })
                .collect(),
            ..PrinterStatus::default()
        }
    }

    fn resolved(command: Command, objects: &[&str]) -> Result<Vec<Command>, String> {
        command.resolve(&printer(objects), &PrinterStatus::default())
    }

    fn set(heater: &str, target: f64) -> Command {
        Command::SetHeaterTarget { heater: heater.into(), target }
    }

    #[test]
    fn leveling_uses_quad_gantry_level_before_z_tilt() {
        assert_eq!(resolved(Command::Level, &["quad_gantry_level", "z_tilt"]), Ok(vec![Command::QuadGantryLevel]));
        assert_eq!(resolved(Command::Level, &["z_tilt"]), Ok(vec![Command::ZTiltAdjust]));
        assert!(resolved(Command::Level, &["extruder"]).is_err());
    }

    #[test]
    fn the_z_offset_goes_to_the_probe_if_there_is_one() {
        assert_eq!(resolved(Command::ApplyZOffset, &["probe"]), Ok(vec![Command::ApplyZOffsetToProbe]));
        assert_eq!(resolved(Command::ApplyZOffset, &["extruder"]), Ok(vec![Command::ApplyZOffsetToEndstop]));
        assert_eq!(script(Command::ApplyZOffsetToProbe), "Z_OFFSET_APPLY_PROBE");
        assert_eq!(script(Command::ApplyZOffsetToEndstop), "Z_OFFSET_APPLY_ENDSTOP");
    }

    #[test]
    fn a_preset_heats_the_bed_only_if_there_is_one() {
        let preset = Command::ApplyPreset { extruder: 210.0, bed: 60.0 };

        assert_eq!(
            resolved(preset.clone(), &["extruder", "heater_bed"]),
            Ok(vec![set("extruder", 210.0), set("heater_bed", 60.0)])
        );
        assert_eq!(resolved(preset, &["extruder"]), Ok(vec![set("extruder", 210.0)]));
    }

    #[test]
    fn cooling_down_switches_every_heater_off() {
        let status =
            status_with_targets(&[("extruder", 210.0), ("heater_bed", 60.0), ("heater_generic chamber", 40.0)]);

        let commands = Command::CoolDown.resolve(&printer(&[]), &status);

        assert_eq!(
            commands,
            Ok(vec![set("extruder", 0.0), set("heater_bed", 0.0), set("heater_generic chamber", 0.0)])
        );
    }

    #[test]
    fn a_step_starts_from_the_target_shown_now() {
        let status = status_with_targets(&[("extruder", 200.0)]);
        let step = |step| Command::StepHeater { heater: "extruder".into(), step };

        assert_eq!(step(5.0).resolve(&printer(&[]), &status), Ok(vec![set("extruder", 205.0)]));
        assert_eq!(step(-5.0).resolve(&printer(&[]), &status), Ok(vec![set("extruder", 195.0)]));
        assert!(Command::StepHeater { heater: "nozzle".into(), step: 5.0 }.resolve(&printer(&[]), &status).is_err());
    }

    #[test]
    fn loading_prefers_happy_hare_then_the_macros() {
        let macro_named = |name: &str| Command::RunMacro { name: name.into() };

        assert_eq!(resolved(Command::LoadFilament, &["mmu", "gcode_macro LOAD_FILAMENT"]), Ok(vec![Command::MmuLoad]));
        assert_eq!(resolved(Command::UnloadFilament, &["mmu"]), Ok(vec![Command::MmuUnload]));
        assert_eq!(
            resolved(Command::LoadFilament, &["gcode_macro LOAD_FILAMENT"]),
            Ok(vec![macro_named("LOAD_FILAMENT")])
        );
        assert_eq!(
            resolved(Command::UnloadFilament, &["gcode_macro UNLOAD_FILAMENT"]),
            Ok(vec![macro_named("UNLOAD_FILAMENT")])
        );
        assert!(resolved(Command::LoadFilament, &["gcode_macro UNLOAD_FILAMENT"]).is_err());
        assert!(resolved(Command::UnloadFilament, &["extruder"]).is_err());
    }

    #[test]
    fn typed_gcode_is_trimmed_and_blank_input_is_dropped() {
        let gcode = |script: &str| Command::RunGcode { script: script.into() };

        assert_eq!(resolved(gcode("  G28 \n"), &[]), Ok(vec![gcode("G28")]));
        assert_eq!(resolved(gcode("   "), &[]), Ok(Vec::new()));
    }

    #[test]
    fn other_commands_stay_as_they_are() {
        assert_eq!(resolved(Command::PausePrint, &[]), Ok(vec![Command::PausePrint]));
    }

    #[test]
    fn intents_are_no_calls_of_their_own() {
        assert_eq!(Command::Level.to_call(&Motion::default()), None);
        assert_eq!(Command::CoolDown.to_call(&Motion::default()), None);
    }

    #[test]
    fn spool_commands_reach_moonraker_and_happy_hare() {
        assert_eq!(script(Command::AssignGateSpool { gate: 2, spool_id: Some(18) }), "MMU_GATE_MAP GATE=2 SPOOLID=18");
        assert_eq!(script(Command::AssignGateSpool { gate: 2, spool_id: None }), "MMU_GATE_MAP GATE=2 SPOOLID=-1");
        let motion = Motion::default();
        let active = Command::SetActiveSpool { spool_id: Some(18) }.to_call(&motion).unwrap();
        assert_eq!(active, Call::with("server.spoolman.post_spool_id", json!({ "spool_id": 18 })));
        let none = Command::SetActiveSpool { spool_id: None }.to_call(&motion).unwrap();
        assert_eq!(none.params.unwrap(), json!({ "spool_id": null }));
        let measure = Command::MeasureSpool { spool_id: 18, weight: 793.5 }.to_call(&motion).unwrap();
        assert_eq!(measure.method, "server.spoolman.proxy");
        assert_eq!(
            measure.params.unwrap(),
            json!({ "request_method": "PUT", "path": "/v1/spool/18/measure", "body": { "weight": 793.5 }, "use_v2_response": true })
        );
        assert!(Command::MeasureSpool { spool_id: 1, weight: 1.0 }.changes_spools());
        assert!(!Command::PausePrint.changes_spools());
    }

    #[test]
    fn describes_commands_briefly() {
        assert_eq!(Command::PausePrint.describe(), "PausePrint");
        assert_eq!(Command::Jog { axis: Axis::X, distance: 1.0 }.describe(), "Jog");
        assert_eq!(Command::RunMacro { name: "PARK".into() }.describe(), "PARK");
    }
}
