//! Parser fuer die G-Code-Zeilen, die die UI per `printer.gcode.script` schickt.

use std::fmt;

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
            Axis::X => 'x',
            Axis::Y => 'y',
            Axis::Z => 'z',
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Move {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub e: Option<f64>,
}

impl Move {
    pub fn linear_axes(&self) -> impl Iterator<Item = (Axis, f64)> {
        [(Axis::X, self.x), (Axis::Y, self.y), (Axis::Z, self.z)]
            .into_iter()
            .filter_map(|(axis, value)| value.map(|value| (axis, value)))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Home(Vec<Axis>),
    SetExtruderTarget(f64),
    SetBedTarget(f64),
    SetFanSpeed(f64),
    AbsolutePositioning,
    RelativePositioning,
    Move(Move),
    DisableSteppers,
    SetGcodeOffset { z: Option<f64>, z_adjust: Option<f64> },
    AbsoluteExtrusion,
    RelativeExtrusion,
    SetSpeedFactor(f64),
    SetExtrudeFactor(f64),
    SetPressureAdvance(f64),
    SetHeaterTemperature { heater: String, target: f64 },
    SaveConfig,
    QuadGantryLevel,
    Mmu(MmuCommand),
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmuCommand {
    Home,
    Select { gate: usize },
    ChangeTool { tool: usize },
    Load { gate: Option<usize> },
    Unload,
}

impl MmuCommand {
    pub fn gcode_name(&self) -> &'static str {
        match self {
            MmuCommand::Home => "MMU_HOME",
            MmuCommand::Select { .. } => "MMU_SELECT",
            MmuCommand::ChangeTool { .. } => "MMU_CHANGE_TOOL",
            MmuCommand::Load { .. } => "MMU_LOAD",
            MmuCommand::Unload => "MMU_UNLOAD",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandError(String);

impl CommandError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub fn message(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for CommandError {}

pub fn parse_script(script: &str) -> Result<Vec<Command>, CommandError> {
    script.lines().filter_map(significant_part).map(parse_line).collect()
}

fn significant_part(line: &str) -> Option<&str> {
    let code = line.split(';').next().unwrap_or_default().trim();
    (!code.is_empty()).then_some(code)
}

fn parse_line(line: &str) -> Result<Command, CommandError> {
    let mut words = line.split_whitespace();
    let name = words.next().unwrap_or_default().to_ascii_uppercase();
    let args: Vec<&str> = words.collect();

    if let Some(tool) = tool_number(&name) {
        return Ok(Command::Mmu(MmuCommand::ChangeTool { tool }));
    }

    match name.as_str() {
        "G28" => Ok(Command::Home(parse_axes(&args))),
        "M104" => Ok(Command::SetExtruderTarget(letter_value(&args, 'S')?.unwrap_or(0.0))),
        "M140" => Ok(Command::SetBedTarget(letter_value(&args, 'S')?.unwrap_or(0.0))),
        "M106" => Ok(Command::SetFanSpeed(fan_speed(&args)?)),
        "M107" => Ok(Command::SetFanSpeed(0.0)),
        "M84" | "M18" => Ok(Command::DisableSteppers),
        "G90" => Ok(Command::AbsolutePositioning),
        "G91" => Ok(Command::RelativePositioning),
        "G0" | "G1" => Ok(Command::Move(parse_move(&args)?)),
        "SET_GCODE_OFFSET" => {
            Ok(Command::SetGcodeOffset { z: named_value(&args, "Z")?, z_adjust: named_value(&args, "Z_ADJUST")? })
        }
        "M82" => Ok(Command::AbsoluteExtrusion),
        "M83" => Ok(Command::RelativeExtrusion),
        "M220" => Ok(Command::SetSpeedFactor(letter_value(&args, 'S')?.unwrap_or(100.0) / 100.0)),
        "M221" => Ok(Command::SetExtrudeFactor(letter_value(&args, 'S')?.unwrap_or(100.0) / 100.0)),
        "SET_PRESSURE_ADVANCE" => Ok(Command::SetPressureAdvance(
            named_value(&args, "ADVANCE")?.ok_or_else(|| CommandError::new("Missing parameter ADVANCE"))?,
        )),
        "SET_HEATER_TEMPERATURE" => Ok(Command::SetHeaterTemperature {
            heater: named_text(&args, "HEATER").ok_or_else(|| CommandError::new("Missing parameter HEATER"))?,
            target: named_value(&args, "TARGET")?.unwrap_or(0.0),
        }),
        "SAVE_CONFIG" => Ok(Command::SaveConfig),
        "QUAD_GANTRY_LEVEL" => Ok(Command::QuadGantryLevel),
        "MMU_HOME" => Ok(Command::Mmu(MmuCommand::Home)),
        "MMU_SELECT" => Ok(Command::Mmu(MmuCommand::Select { gate: required_index(&args, "GATE")? })),
        "MMU_CHANGE_TOOL" => Ok(Command::Mmu(MmuCommand::ChangeTool { tool: required_index(&args, "TOOL")? })),
        "MMU_LOAD" => Ok(Command::Mmu(MmuCommand::Load { gate: named_index(&args, "GATE")? })),
        "MMU_UNLOAD" => Ok(Command::Mmu(MmuCommand::Unload)),
        _ => Ok(Command::Ignored),
    }
}

fn tool_number(name: &str) -> Option<usize> {
    name.strip_prefix('T')
        .filter(|digits| !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|digits| digits.parse().ok())
}

fn parse_move(args: &[&str]) -> Result<Move, CommandError> {
    Ok(Move {
        x: letter_value(args, 'X')?,
        y: letter_value(args, 'Y')?,
        z: letter_value(args, 'Z')?,
        e: letter_value(args, 'E')?,
    })
}

fn fan_speed(args: &[&str]) -> Result<f64, CommandError> {
    let raw = letter_value(args, 'S')?.unwrap_or(255.0);
    Ok(raw.clamp(0.0, 255.0) / 255.0)
}

fn parse_axes(args: &[&str]) -> Vec<Axis> {
    args.iter()
        .filter_map(|arg| match arg.to_ascii_uppercase().as_str() {
            "X" => Some(Axis::X),
            "Y" => Some(Axis::Y),
            "Z" => Some(Axis::Z),
            _ => None,
        })
        .collect()
}

fn letter_value(args: &[&str], letter: char) -> Result<Option<f64>, CommandError> {
    args.iter()
        .find(|arg| arg.chars().next().map(|first| first.to_ascii_uppercase()) == Some(letter))
        // Das erste Zeichen ist ein ASCII-Buchstabe und damit genau ein Byte lang.
        .map(|arg| parse_number(&arg[1..]))
        .transpose()
}

fn named_value(args: &[&str], name: &str) -> Result<Option<f64>, CommandError> {
    args.iter()
        .filter_map(|arg| arg.split_once('='))
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| parse_number(value))
        .transpose()
}

fn named_text(args: &[&str], name: &str) -> Option<String> {
    args.iter()
        .filter_map(|arg| arg.split_once('='))
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.to_owned())
}

fn required_index(args: &[&str], name: &str) -> Result<usize, CommandError> {
    named_index(args, name)?.ok_or_else(|| CommandError::new(format!("Missing parameter {name}")))
}

fn named_index(args: &[&str], name: &str) -> Result<Option<usize>, CommandError> {
    named_value(args, name)?
        .map(|value| {
            if value >= 0.0 && value.fract() == 0.0 {
                Ok(value as usize)
            } else {
                Err(CommandError::new(format!("Parameter {name} must be a non-negative integer")))
            }
        })
        .transpose()
}

fn parse_number(text: &str) -> Result<f64, CommandError> {
    text.parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
        .ok_or_else(|| CommandError::new(format!("Unable to parse number '{text}'")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_relative_extrusion_script() {
        let commands = parse_script("G91\nG1 E10 F180").unwrap();

        assert_eq!(
            commands,
            vec![Command::RelativePositioning, Command::Move(Move { e: Some(10.0), ..Move::default() }),]
        );
    }

    #[test]
    fn parses_tuning_commands() {
        let commands =
            parse_script("M220 S110\nM221 S95\nSET_PRESSURE_ADVANCE ADVANCE=0.05\nM83\nSET_HEATER_TEMPERATURE HEATER=chamber TARGET=40")
                .unwrap();

        assert_eq!(
            commands,
            vec![
                Command::SetSpeedFactor(1.1),
                Command::SetExtrudeFactor(0.95),
                Command::SetPressureAdvance(0.05),
                Command::RelativeExtrusion,
                Command::SetHeaterTemperature { heater: "chamber".to_owned(), target: 40.0 },
            ]
        );
    }

    #[test]
    fn parses_homing_with_and_without_axes() {
        assert_eq!(parse_script("G28 X Y").unwrap(), vec![Command::Home(vec![Axis::X, Axis::Y])]);
        assert_eq!(parse_script("G28").unwrap(), vec![Command::Home(vec![])]);
    }

    #[test]
    fn ignores_comments_and_blank_lines() {
        let commands = parse_script("; nur Kommentar\n\nM104 S200 ; Duese\n").unwrap();

        assert_eq!(commands, vec![Command::SetExtruderTarget(200.0)]);
    }

    #[test]
    fn scales_fan_speed_to_fraction() {
        assert_eq!(parse_script("M106 S255").unwrap(), vec![Command::SetFanSpeed(1.0)]);
        assert_eq!(parse_script("M107").unwrap(), vec![Command::SetFanSpeed(0.0)]);
    }

    #[test]
    fn parses_named_gcode_offset_arguments() {
        let commands = parse_script("SET_GCODE_OFFSET Z_ADJUST=-0.05 MOVE=1").unwrap();

        assert_eq!(commands, vec![Command::SetGcodeOffset { z: None, z_adjust: Some(-0.05) }]);
    }

    #[test]
    fn parses_happy_hare_commands() {
        let commands =
            parse_script("MMU_HOME\nMMU_SELECT GATE=2\nMMU_CHANGE_TOOL TOOL=1\nT3\nMMU_LOAD\nMMU_UNLOAD").unwrap();

        assert_eq!(
            commands,
            vec![
                Command::Mmu(MmuCommand::Home),
                Command::Mmu(MmuCommand::Select { gate: 2 }),
                Command::Mmu(MmuCommand::ChangeTool { tool: 1 }),
                Command::Mmu(MmuCommand::ChangeTool { tool: 3 }),
                Command::Mmu(MmuCommand::Load { gate: None }),
                Command::Mmu(MmuCommand::Unload),
            ]
        );
    }

    #[test]
    fn happy_hare_commands_validate_their_parameters() {
        assert_eq!(parse_script("MMU_SELECT").unwrap_err().message(), "Missing parameter GATE");
        assert!(parse_script("MMU_SELECT GATE=1.5").is_err());
        assert!(parse_script("MMU_SELECT GATE=-1").is_err());
    }

    #[test]
    fn only_digits_after_t_make_a_tool_change() {
        assert_eq!(parse_script("TURN_OFF_HEATERS").unwrap(), vec![Command::Ignored]);
    }

    #[test]
    fn unknown_commands_are_ignored() {
        assert_eq!(parse_script("M117 Hallo").unwrap(), vec![Command::Ignored]);
    }

    #[test]
    fn rejects_unparsable_numbers() {
        let error = parse_script("G1 Xabc").unwrap_err();

        assert_eq!(error.message(), "Unable to parse number 'abc'");
    }
}
