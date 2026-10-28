//! Happy Hare's G-code, run through Klipper. Parameters follow Happy Hare 4.1; `MMU_LOAD` and `MMU_UNLOAD` act on
//! the selected gate and have no gate parameter.

pub const LOAD: &str = "MMU_LOAD";
pub const UNLOAD: &str = "MMU_UNLOAD";
pub const HOME: &str = "MMU_HOME";
pub const RECOVER: &str = "MMU_RECOVER";
pub const UNLOCK: &str = "MMU_UNLOCK";
/// Turns off the motors and servos of the MMU so it can be moved by hand; the selector loses its home.
pub const MOTORS_OFF: &str = "MMU_MOTORS_OFF";

/// Gates a command acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gates {
    All,
    One(usize),
}

/// Servo positions `MMU_SERVO POS=` takes that the UI offers: up releases the filament, down grips it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServoCommand {
    Up,
    Down,
}

pub fn select_gate(gate: usize) -> String {
    format!("MMU_SELECT GATE={gate}")
}

/// Ejects the filament of a gate completely; without `gate` the selected one.
pub fn eject(gate: Option<usize>) -> String {
    match gate {
        Some(gate) => format!("MMU_EJECT GATE={gate}"),
        None => "MMU_EJECT".to_owned(),
    }
}

/// Inspects gates and marks them available or empty; Happy Hare unloads loaded filament first, unless only its gate
/// is checked (`mmu_check_gate.py`).
pub fn check_gates(gates: Gates) -> String {
    match gates {
        Gates::All => "MMU_CHECK_GATE ALL=1".to_owned(),
        Gates::One(gate) => format!("MMU_CHECK_GATE GATE={gate}"),
    }
}

/// Grabs filament pushed into a gate and parks it there; Happy Hare refuses while printing.
pub fn preload(gate: usize) -> String {
    format!("MMU_PRELOAD GATE={gate}")
}

pub fn servo(position: ServoCommand) -> String {
    let position = match position {
        ServoCommand::Up => "up",
        ServoCommand::Down => "down",
    };
    format!("MMU_SERVO POS={position}")
}

/// Selects a gate and loads it in one script, so both run in this order.
pub fn load_gate(gate: usize) -> String {
    format!("{}\n{LOAD}", select_gate(gate))
}

/// Puts a Spoolman spool into a gate; `None` takes the spool out of the record (`SPOOLID=-1`).
pub fn assign_gate_spool(gate: usize, spool_id: Option<u32>) -> String {
    format!("MMU_GATE_MAP GATE={gate} SPOOLID={}", spool_id.map_or(-1, i64::from))
}

/// Sets material or colour of a gate in Happy Hare's gate map; `None` keeps what the gate has. Without `TEMP` Happy
/// Hare puts the temperature of the gate back to `default_extruder_temp` (`mmu_gate_map.py`), so the current one goes
/// along. The material is one word of letters, digits and `+-_.`, as G-code parameters cannot hold spaces.
pub fn set_gate_filament(
    gate: usize,
    material: Option<&str>,
    color: Option<[u8; 3]>,
    temperature: Option<f64>,
) -> Result<String, String> {
    let mut script = format!("MMU_GATE_MAP GATE={gate}");
    if let Some(material) = material {
        let valid = !material.is_empty()
            && material.chars().all(|letter| letter.is_ascii_alphanumeric() || "+-_.".contains(letter));
        if !valid {
            return Err(format!("invalid material '{material}'"));
        }
        script += &format!(" MATERIAL={material}");
    }
    if let Some([red, green, blue]) = color {
        script += &format!(" COLOR={red:02x}{green:02x}{blue:02x}");
    }
    if let Some(temperature) = temperature {
        script += &format!(" TEMP={}", temperature.round());
    }
    Ok(script)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gates_are_named_by_the_documented_parameter() {
        assert_eq!(select_gate(3), "MMU_SELECT GATE=3");
        assert_eq!(eject(None), "MMU_EJECT");
        assert_eq!(eject(Some(1)), "MMU_EJECT GATE=1");
        assert_eq!(preload(2), "MMU_PRELOAD GATE=2");
    }

    #[test]
    fn gates_are_checked_one_by_one_or_all_at_once() {
        assert_eq!(check_gates(Gates::One(4)), "MMU_CHECK_GATE GATE=4");
        assert_eq!(check_gates(Gates::All), "MMU_CHECK_GATE ALL=1");
    }

    #[test]
    fn the_servo_takes_the_documented_positions() {
        assert_eq!(servo(ServoCommand::Up), "MMU_SERVO POS=up");
        assert_eq!(servo(ServoCommand::Down), "MMU_SERVO POS=down");
    }

    #[test]
    fn a_gate_spool_is_set_or_taken_out() {
        assert_eq!(assign_gate_spool(2, Some(18)), "MMU_GATE_MAP GATE=2 SPOOLID=18");
        assert_eq!(assign_gate_spool(2, None), "MMU_GATE_MAP GATE=2 SPOOLID=-1");
    }

    #[test]
    fn a_gate_is_selected_and_loaded_in_one_script() {
        assert_eq!(load_gate(3), "MMU_SELECT GATE=3\nMMU_LOAD");
    }

    #[test]
    fn material_and_colour_of_a_gate_keep_its_temperature() {
        assert_eq!(
            set_gate_filament(1, Some("PETG"), None, Some(235.0)).as_deref(),
            Ok("MMU_GATE_MAP GATE=1 MATERIAL=PETG TEMP=235")
        );
        assert_eq!(
            set_gate_filament(0, None, Some([255, 128, 0]), None).as_deref(),
            Ok("MMU_GATE_MAP GATE=0 COLOR=ff8000")
        );
        assert!(set_gate_filament(0, Some("PLA TEMP=300"), None, None).is_err());
        assert!(set_gate_filament(0, Some(""), None, None).is_err());
    }
}
