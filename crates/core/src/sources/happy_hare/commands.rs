//! Happy Hare's G-code, run through Klipper. Parameters follow Happy Hare 4.1; `MMU_LOAD` and `MMU_UNLOAD` act on
//! the selected gate and have no gate parameter.

pub const LOAD: &str = "MMU_LOAD";
pub const UNLOAD: &str = "MMU_UNLOAD";
pub const HOME: &str = "MMU_HOME";
pub const RECOVER: &str = "MMU_RECOVER";
pub const UNLOCK: &str = "MMU_UNLOCK";

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

/// Puts a Spoolman spool into a gate; `None` takes the spool out of the record (`SPOOLID=-1`).
pub fn assign_gate_spool(gate: usize, spool_id: Option<u32>) -> String {
    format!("MMU_GATE_MAP GATE={gate} SPOOLID={}", spool_id.map_or(-1, i64::from))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gates_are_named_by_the_documented_parameter() {
        assert_eq!(select_gate(3), "MMU_SELECT GATE=3");
        assert_eq!(eject(None), "MMU_EJECT");
        assert_eq!(eject(Some(1)), "MMU_EJECT GATE=1");
    }

    #[test]
    fn a_gate_spool_is_set_or_taken_out() {
        assert_eq!(assign_gate_spool(2, Some(18)), "MMU_GATE_MAP GATE=2 SPOOLID=18");
        assert_eq!(assign_gate_spool(2, None), "MMU_GATE_MAP GATE=2 SPOOLID=-1");
    }
}
