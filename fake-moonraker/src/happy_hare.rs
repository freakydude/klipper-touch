//! Simplified Happy Hare simulation (objects `mmu` and `mmu_machine`).
//!
//! Field names follow the Happy Hare v4 reference "Printer Variables". Operations run without
//! elapsed time: `action` is always `Idle`, loading and unloading complete immediately.

use crate::gcode::{CommandError, MmuCommand};
use serde_json::{json, Map, Value};

const VERSION: &str = "4.1.0";
const NONE_SELECTED: i32 = -1;
const GATE_EMPTY: i32 = 0;
const GATE_AVAILABLE: i32 = 1;
const FILAMENT_POSITION_UNLOADED: i32 = 0;
const FILAMENT_POSITION_LOADED: i32 = 10;
/// Where Happy Hare's `filament_pos` passes the sensors of the way: gate exit, extruder entry, toolhead.
const FILAMENT_POSITION_GATE_SENSOR: i32 = 1;
const FILAMENT_POSITION_EXTRUDER_SENSOR: i32 = 5;
const FILAMENT_POSITION_TOOLHEAD_SENSOR: i32 = 8;
/// Millimetres from the gate to the nozzle, for `filament_position`.
const PATH_LENGTH: f64 = 1712.4;
const UNIT_NAME: &str = "unit0";

pub(crate) struct Spool {
    pub(crate) name: &'static str,
    pub(crate) material: &'static str,
    pub(crate) vendor: &'static str,
    pub(crate) color: &'static str,
    pub(crate) rgb: [f64; 3],
    pub(crate) temperature: i32,
}

/// The demo filaments the gates cycle through; the two after them belong to the multi-colour spools of Spoolman.
pub(crate) const BASIC_FILAMENTS: usize = 4;

pub(crate) const SPOOLS: [Spool; 6] = [
    Spool { name: "Red PLA", material: "PLA", vendor: "Fake", color: "red", rgb: [1.0, 0.0, 0.0], temperature: 210 },
    Spool {
        name: "Green PETG",
        material: "PETG",
        vendor: "Fake",
        color: "green",
        rgb: [0.0, 0.8, 0.0],
        temperature: 235,
    },
    Spool { name: "Blue PLA", material: "PLA", vendor: "Fake", color: "blue", rgb: [0.0, 0.2, 1.0], temperature: 210 },
    Spool {
        name: "Black ABS",
        material: "ABS",
        vendor: "Fake",
        color: "black",
        rgb: [0.1, 0.1, 0.1],
        temperature: 250,
    },
    // The first colour of the multi-colour filaments, which is what Happy Hare reports for a gate.
    Spool {
        name: "Gradient Red+Green",
        material: "PLA",
        vendor: "Fake",
        color: "red",
        rgb: [0.89, 0.12, 0.17],
        temperature: 215,
    },
    Spool {
        name: "Silk Red+Gold+Black",
        material: "PLA",
        vendor: "Fake",
        color: "red",
        rgb: [1.0, 0.0, 0.02],
        temperature: 220,
    },
];

/// Spools of the gates, in the order the gates take them: the first is one colour, then two, then three colours.
const GATE_SPOOLS: [u32; 7] = [1, 7, 8, 4, 5, 6, 2];

/// Tension from which tangle prevention raises the gear current, and below which it lets go (Happy Hare defaults).
const TANGLE_THRESHOLD: f64 = 0.3;
const TANGLE_RELEASE: f64 = 0.2;

/// What the proportional buffer reports for one situation.
struct Buffer {
    state: &'static str,
    bias: f64,
    flow_rate: f64,
    flowguard_level: f64,
}

#[derive(Debug, Clone)]
pub struct HappyHare {
    gates: usize,
    homed: bool,
    gate: i32,
    tool: i32,
    loaded: bool,
    /// Last movement of the filament: 1 load, -1 unload, 0 none (`filament_direction`).
    direction: i32,
    /// Held by `FAKE_MMU_FILAMENT`; otherwise the position follows `loaded`.
    held_position: Option<i32>,
    toolchanges: u32,
    tool_to_gate: Vec<usize>,
    gate_status: Vec<i32>,
    /// Spoolman spool of each gate.
    gate_spool: Vec<Option<u32>>,
    /// The filament pulls hard on the buffer: tension at the stop and tangle prevention raises the gear current.
    boosting: bool,
}

impl HappyHare {
    pub fn new(gates: usize) -> Self {
        // The last gate stays empty so the UI also has to show an empty slot.
        let mut gate_status = vec![GATE_AVAILABLE; gates];
        if gates > 2 {
            gate_status[gates - 1] = GATE_EMPTY;
        }
        let mut spools = GATE_SPOOLS.into_iter();
        let gate_spool =
            gate_status.iter().map(|status| if *status == GATE_AVAILABLE { spools.next() } else { None }).collect();
        Self {
            gates,
            homed: false,
            gate: NONE_SELECTED,
            tool: NONE_SELECTED,
            loaded: false,
            direction: 0,
            held_position: None,
            toolchanges: 0,
            tool_to_gate: (0..gates).collect(),
            gate_status,
            gate_spool,
            boosting: false,
        }
    }

    pub fn apply(&mut self, command: MmuCommand) -> Result<(), CommandError> {
        match command {
            MmuCommand::Home => {
                self.homed = true;
                Ok(())
            }
            MmuCommand::Select { gate } => self.select_gate(gate),
            MmuCommand::ChangeTool { tool } => self.change_tool(tool),
            MmuCommand::Load => self.load(),
            MmuCommand::Unload => self.unload(),
            MmuCommand::Eject { gate, force } => self.eject(gate, force),
            MmuCommand::GateMap { gate, spool_id } => self.gate_map(gate, spool_id),
            MmuCommand::FakeFilament { position, direction } => {
                self.held_position = Some(position);
                self.direction = direction;
                Ok(())
            }
            MmuCommand::FakeTangle { boosting } => {
                self.boosting = boosting;
                Ok(())
            }
        }
    }

    pub fn status(&self, print_state: &str) -> Map<String, Value> {
        let mut objects = Map::new();
        objects.insert("mmu".to_owned(), self.mmu_json(print_state));
        objects.insert("mmu_machine".to_owned(), self.machine_json());
        objects.insert("mmu_extruder extruder".to_owned(), json!({}));
        objects.insert(format!("mmu_stepper {UNIT_NAME}_gear"), json!({}));
        objects
    }

    pub fn config_sections(&self) -> Vec<(String, Value)> {
        vec![
            ("mmu_machine".to_owned(), json!({ "num_units": 1 })),
            ("mmu".to_owned(), json!({ "enabled": true })),
            ("mmu_extruder extruder".to_owned(), json!({})),
            (format!("mmu_stepper {UNIT_NAME}_gear"), json!({})),
        ]
    }

    fn select_gate(&mut self, gate: usize) -> Result<(), CommandError> {
        self.require_homed()?;
        self.check_gate(gate)?;
        if self.loaded {
            return Err(CommandError::new("Cannot change gate while filament is loaded"));
        }
        self.select(gate);
        Ok(())
    }

    fn change_tool(&mut self, tool: usize) -> Result<(), CommandError> {
        self.require_homed()?;
        let gate =
            *self.tool_to_gate.get(tool).ok_or_else(|| CommandError::new(format!("Tool T{tool} does not exist")))?;
        self.require_filament_at(gate)?;
        self.select(gate);
        self.set_loaded(true);
        self.tool = tool as i32;
        self.toolchanges += 1;
        Ok(())
    }

    fn load(&mut self) -> Result<(), CommandError> {
        self.require_homed()?;
        if self.loaded {
            return Err(CommandError::new("Filament is already loaded"));
        }
        let selected = usize::try_from(self.gate).map_err(|_| CommandError::new("No gate selected"))?;
        self.require_filament_at(selected)?;
        self.set_loaded(true);
        Ok(())
    }

    /// Like `MMU_EJECT` (`mmu_eject.py`): while the filament of the selected gate is loaded, the command only unloads
    /// it (unless `FORCE`); otherwise it pulls the filament out of the gate, which is then empty.
    fn eject(&mut self, gate: Option<usize>, force: bool) -> Result<(), CommandError> {
        self.require_homed()?;
        let current = usize::try_from(self.gate).ok();
        let target = gate.or(current).ok_or_else(|| CommandError::new("No gate selected"))?;
        self.check_gate(target)?;
        if self.loaded && Some(target) == current && !force {
            return self.unload();
        }
        if self.loaded && Some(target) != current {
            return Err(CommandError::new("Cannot eject another gate while filament is loaded"));
        }
        self.set_loaded(false);
        self.gate_status[target] = GATE_EMPTY;
        Ok(())
    }

    fn unload(&mut self) -> Result<(), CommandError> {
        self.require_homed()?;
        if !self.loaded {
            return Err(CommandError::new("Filament is not loaded"));
        }
        self.set_loaded(false);
        Ok(())
    }

    /// A load or an unload ends at its end of the way, moving in its direction.
    fn set_loaded(&mut self, loaded: bool) {
        self.loaded = loaded;
        self.direction = if loaded { 1 } else { -1 };
        self.held_position = None;
    }

    fn filament_position(&self) -> i32 {
        self.held_position.unwrap_or(if self.loaded { FILAMENT_POSITION_LOADED } else { FILAMENT_POSITION_UNLOADED })
    }

    /// Happy Hare's `bowden_progress`: 0 to 100 while the filament moves through the bowden (positions 2 to 4), -1
    /// otherwise. A held position stands for a third of the way per step.
    fn bowden_progress(&self) -> f64 {
        match self.held_position {
            Some(position @ 2..=4) => f64::from(position - 1) * 100.0 / 3.0,
            _ => -1.0,
        }
    }

    fn select(&mut self, gate: usize) {
        self.gate = gate as i32;
        self.tool =
            self.tool_to_gate.iter().position(|mapped| *mapped == gate).map_or(NONE_SELECTED, |tool| tool as i32);
    }

    fn require_homed(&self) -> Result<(), CommandError> {
        if self.homed {
            Ok(())
        } else {
            Err(CommandError::new("MMU is not homed"))
        }
    }

    fn check_gate(&self, gate: usize) -> Result<(), CommandError> {
        if gate < self.gates {
            Ok(())
        } else {
            Err(CommandError::new(format!("Gate {gate} does not exist")))
        }
    }

    fn require_filament_at(&self, gate: usize) -> Result<(), CommandError> {
        if self.gate_status[gate] == GATE_EMPTY {
            Err(CommandError::new(format!("Gate {gate} is empty")))
        } else {
            Ok(())
        }
    }

    /// The filament of a gate: that of its Spoolman spool, which Happy Hare takes over, else a demo filament.
    fn spool(&self, gate: usize) -> &'static Spool {
        match self.gate_spool.get(gate).copied().flatten() {
            Some(id) => &SPOOLS[crate::spoolman::filament_index(id)],
            None => &SPOOLS[gate % BASIC_FILAMENTS],
        }
    }

    /// The Spoolman spool of the selected gate, which Happy Hare makes the active one.
    pub fn selected_spool(&self) -> Option<u32> {
        usize::try_from(self.gate).ok().and_then(|gate| self.gate_spool.get(gate).copied().flatten())
    }

    /// `MMU_GATE_MAP GATE= SPOOLID=`; a negative id takes the spool out.
    fn gate_map(&mut self, gate: usize, spool_id: Option<i64>) -> Result<(), CommandError> {
        self.check_gate(gate)?;
        self.gate_spool[gate] = spool_id.and_then(|id| u32::try_from(id).ok()).filter(|id| *id > 0);
        Ok(())
    }

    fn per_gate<T>(&self, value: impl Fn(&'static Spool) -> T) -> Vec<T> {
        (0..self.gates).map(|gate| value(self.spool(gate))).collect()
    }

    fn active_filament(&self) -> Value {
        match usize::try_from(self.gate) {
            Ok(gate) => {
                let spool = self.spool(gate);
                json!({
                    "filament_name": spool.name,
                    "material": spool.material,
                    "vendor": spool.vendor,
                    "color": spool.color,
                    "spool_id": self.gate_spool.get(gate).copied().flatten().map_or(-1, i64::from),
                    "temperature": spool.temperature,
                    "td": null,
                })
            }
            Err(_) => json!({}),
        }
    }

    fn mmu_json(&self, print_state: &str) -> Value {
        let printing = print_state == "printing";
        let mut mmu = json!({
            "enabled": true,
            "num_gates": self.gates,
            "is_homed": self.homed,
            "print_state": print_state,
            "unit": if self.gate == NONE_SELECTED { NONE_SELECTED } else { 0 },
            "tool": self.tool,
            "gate": self.gate,
            "active_filament": self.active_filament(),
            "num_toolchanges": self.toolchanges,
            "operation": "",
            "filament": if self.loaded { "Loaded" } else { "Unloaded" },
            "filament_pos": self.filament_position(),
            "filament_direction": self.direction,
            "bowden_progress": self.bowden_progress(),
            "filament_position": PATH_LENGTH * f64::from(self.filament_position().max(0)) / f64::from(FILAMENT_POSITION_LOADED)
                * f64::from(if self.direction < 0 { -1 } else { 1 }),
            "action": "Idle",
            "reason_for_pause": "",
            "is_paused": false,
            "is_locked": false,
            "sensors": self.sensors_json(printing),
            "sync_drive": false,
            "spoolman_support": "off",
            "ttg_map": self.tool_to_gate,
            "endless_spool_enabled": 0,
            "gate_status": self.gate_status,
            "gate_filament_name": self.per_gate(|spool| spool.name),
            "gate_material": self.per_gate(|spool| spool.material),
            "gate_vendor": self.per_gate(|spool| spool.vendor),
            "gate_color": self.per_gate(|spool| spool.color),
            "gate_color_rgb": self.per_gate(|spool| spool.rgb),
            "gate_temperature": self.per_gate(|spool| spool.temperature),
            "gate_spool_id": self.gate_spool.iter().map(|id| id.map_or(-1, i64::from)).collect::<Vec<_>>(),
            "spoolman_support": "push",
            "pending_spool_id": -1,
            "encoder": null,
        });
        let (Some(fields), Value::Object(buffer)) = (mmu.as_object_mut(), self.buffer_fields(printing)) else {
            unreachable!("both are JSON objects")
        };
        fields.extend(buffer);
        mmu
    }

    /// The fields of a proportional buffer, as on the real device (no encoder), made up from the print state.
    fn buffer_fields(&self, printing: bool) -> Value {
        let buffer = self.buffer(printing);
        json!({
            "sync_feedback_enabled": 1,
            "sync_feedback_state": buffer.state,
            "sync_feedback_bias_raw": buffer.bias,
            "sync_feedback_bias_modelled": buffer.bias,
            "sync_feedback_flow_rate": buffer.flow_rate,
            "flowguard": {
                "enabled": true,
                "active": printing,
                "trigger": "",
                "reason": "",
                "level": buffer.flowguard_level,
                "max_clog": 0.0,
                "max_tangle": buffer.flowguard_level.min(0.0),
            },
            "tangle_prevention": {
                "enabled": true,
                "active": printing,
                "boosted": printing && self.boosting,
                "threshold": TANGLE_THRESHOLD,
                "release": TANGLE_RELEASE,
            },
        })
    }

    /// The proportional buffer in the three situations the UI tells apart: idle, printing in the middle, and
    /// printing with the filament pulling at the stop. Values are made up and simplified.
    fn buffer(&self, printing: bool) -> Buffer {
        match (printing, self.boosting) {
            (false, _) => Buffer { state: "inactive", bias: 0.0, flow_rate: 100.0, flowguard_level: 0.0 },
            (true, false) => Buffer { state: "neutral", bias: 0.05, flow_rate: 98.47, flowguard_level: 0.0 },
            (true, true) => Buffer { state: "tension", bias: -0.93, flow_rate: 98.47, flowguard_level: -0.4 },
        }
    }

    /// `mmu.sensors` with generic names; the Happy Hare 3 aliases are included as Happy Hare 4.1 does for older UIs.
    fn sensors_json(&self, printing: bool) -> Value {
        let at_entry = usize::try_from(self.gate)
            .ok()
            .and_then(|gate| self.gate_status.get(gate))
            .is_some_and(|status| *status == GATE_AVAILABLE);
        let past = |sensor: i32| self.filament_position() >= sensor;
        json!({
            "mmu_entry": at_entry,
            "mmu_exit": past(FILAMENT_POSITION_GATE_SENSOR),
            "extruder": past(FILAMENT_POSITION_EXTRUDER_SENSOR),
            "toolhead": past(FILAMENT_POSITION_TOOLHEAD_SENSOR),
            "mmu_pre_gate": at_entry,
            "mmu_gear": past(FILAMENT_POSITION_GATE_SENSOR),
            "filament_compression": false,
            "filament_tension": printing && self.boosting,
            "filament_proportional": false,
        })
    }

    fn machine_json(&self) -> Value {
        json!({
            "happy_hare_version": VERSION,
            "num_units": 1,
            "num_gates": self.gates,
            "unit_0": {
                "name": UNIT_NAME,
                "display_name": "Unit 0",
                "vendor": "ERCF",
                "version": "1.1sb",
                "num_gates": self.gates,
                "first_gate": 0,
                "selector_type": "LinearSelector",
                "is_homed": self.homed,
                "has_bypass": false,
                "can_crossload": false,
                "multi_gear": false,
                "filament_buffer": false,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn homed(gates: usize) -> HappyHare {
        let mut hare = HappyHare::new(gates);
        hare.apply(MmuCommand::Home).unwrap();
        hare
    }

    fn mmu(hare: &HappyHare) -> Value {
        hare.status("standby")["mmu"].clone()
    }

    #[test]
    fn starts_unhomed_with_nothing_selected() {
        let status = mmu(&HappyHare::new(4));

        assert_eq!(status["is_homed"], false);
        assert_eq!(status["gate"], -1);
        assert_eq!(status["tool"], -1);
        assert_eq!(status["filament"], "Unloaded");
        assert_eq!(status["active_filament"], json!({}));
    }

    #[test]
    fn last_gate_is_empty() {
        let status = mmu(&HappyHare::new(4));

        assert_eq!(status["gate_status"], json!([1, 1, 1, 0]));
        assert_eq!(status["gate_filament_name"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn every_move_requires_homing() {
        let mut hare = HappyHare::new(4);

        for command in [MmuCommand::Select { gate: 1 }, MmuCommand::ChangeTool { tool: 1 }, MmuCommand::Load] {
            assert_eq!(hare.apply(command).unwrap_err().message(), "MMU is not homed");
        }
    }

    #[test]
    fn select_sets_gate_and_mapped_tool() {
        let mut hare = homed(4);

        hare.apply(MmuCommand::Select { gate: 2 }).unwrap();

        let status = mmu(&hare);
        assert_eq!(status["gate"], 2);
        assert_eq!(status["tool"], 2);
        assert_eq!(status["unit"], 0);
        // Gate 2 holds the spool with three colours (spool 8), whose first colour Happy Hare reports.
        assert_eq!(status["active_filament"]["filament_name"], "Silk Red+Gold+Black");
        assert_eq!(status["active_filament"]["spool_id"], 8);
    }

    #[test]
    fn change_tool_loads_filament_and_counts() {
        let mut hare = homed(4);

        hare.apply(MmuCommand::ChangeTool { tool: 1 }).unwrap();
        hare.apply(MmuCommand::ChangeTool { tool: 2 }).unwrap();

        let status = mmu(&hare);
        assert_eq!(status["tool"], 2);
        assert_eq!(status["gate"], 2);
        assert_eq!(status["filament"], "Loaded");
        assert_eq!(status["filament_pos"], 10);
        assert_eq!(status["num_toolchanges"], 2);
    }

    #[test]
    fn empty_gate_cannot_be_used() {
        let mut hare = homed(4);

        let error = hare.apply(MmuCommand::ChangeTool { tool: 3 }).unwrap_err();

        assert_eq!(error.message(), "Gate 3 is empty");
        assert_eq!(mmu(&hare)["filament"], "Unloaded");
    }

    #[test]
    fn unknown_tool_and_gate_are_rejected() {
        let mut hare = homed(4);

        assert_eq!(hare.apply(MmuCommand::ChangeTool { tool: 9 }).unwrap_err().message(), "Tool T9 does not exist");
        assert_eq!(hare.apply(MmuCommand::Select { gate: 9 }).unwrap_err().message(), "Gate 9 does not exist");
    }

    #[test]
    fn gate_cannot_change_while_loaded() {
        let mut hare = homed(4);
        hare.apply(MmuCommand::ChangeTool { tool: 0 }).unwrap();

        let error = hare.apply(MmuCommand::Select { gate: 1 }).unwrap_err();

        assert_eq!(error.message(), "Cannot change gate while filament is loaded");
    }

    #[test]
    fn unload_keeps_the_gate_selected() {
        let mut hare = homed(4);
        hare.apply(MmuCommand::ChangeTool { tool: 1 }).unwrap();

        hare.apply(MmuCommand::Unload).unwrap();

        let status = mmu(&hare);
        assert_eq!(status["filament"], "Unloaded");
        assert_eq!(status["gate"], 1);
        assert!(hare.apply(MmuCommand::Unload).is_err());
    }

    #[test]
    fn eject_unloads_first_and_then_empties_the_gate() {
        let mut hare = homed(4);
        hare.apply(MmuCommand::Select { gate: 1 }).unwrap();
        hare.apply(MmuCommand::Load).unwrap();

        hare.apply(MmuCommand::Eject { gate: None, force: false }).unwrap();
        assert_eq!(mmu(&hare)["filament"], "Unloaded");
        assert_eq!(mmu(&hare)["gate_status"][1], GATE_AVAILABLE);

        hare.apply(MmuCommand::Eject { gate: None, force: false }).unwrap();
        assert_eq!(mmu(&hare)["gate_status"][1], GATE_EMPTY);
        assert_eq!(
            hare.apply(MmuCommand::Eject { gate: Some(9), force: false }).unwrap_err().message(),
            "Gate 9 does not exist"
        );
    }

    #[test]
    fn a_gate_map_sets_the_spool_and_the_gate_takes_its_filament() {
        let mut hare = homed(4);
        assert_eq!(mmu(&hare)["gate_spool_id"], json!([1, 7, 8, -1]));

        hare.apply(MmuCommand::GateMap { gate: 1, spool_id: Some(4) }).unwrap();
        hare.apply(MmuCommand::GateMap { gate: 0, spool_id: Some(-1) }).unwrap();

        let status = mmu(&hare);
        assert_eq!(status["gate_spool_id"], json!([-1, 4, 8, -1]));
        // Spool 4 is the black ABS: Happy Hare takes material and temperature from Spoolman.
        assert_eq!(status["gate_material"][1], "ABS");
        assert_eq!(status["spoolman_support"], "push");
        assert!(hare.apply(MmuCommand::GateMap { gate: 9, spool_id: Some(1) }).is_err());
    }

    #[test]
    fn the_selected_gate_makes_its_spool_the_active_one() {
        let mut hare = homed(4);
        assert_eq!(hare.selected_spool(), None);

        hare.apply(MmuCommand::Select { gate: 1 }).unwrap();

        assert_eq!(hare.selected_spool(), Some(7));
    }

    #[test]
    fn load_needs_a_selected_gate() {
        let mut hare = homed(4);

        assert_eq!(hare.apply(MmuCommand::Load).unwrap_err().message(), "No gate selected");

        hare.apply(MmuCommand::Select { gate: 1 }).unwrap();
        hare.apply(MmuCommand::Load).unwrap();
        assert_eq!(mmu(&hare)["filament"], "Loaded");
    }

    #[test]
    fn status_registers_all_happy_hare_objects() {
        let objects = HappyHare::new(4).status("standby");

        let names: Vec<&str> = objects.keys().map(String::as_str).collect();
        assert_eq!(names, ["mmu", "mmu_extruder extruder", "mmu_machine", "mmu_stepper unit0_gear"]);
        assert_eq!(objects["mmu_machine"]["unit_0"]["num_gates"], 4);
    }

    #[test]
    fn the_buffer_is_idle_until_a_print_runs() {
        let hare = homed(4);

        assert_eq!(hare.status("standby")["mmu"]["sync_feedback_state"], "inactive");
        assert_eq!(hare.status("standby")["mmu"]["flowguard"]["active"], false);
        assert_eq!(hare.status("printing")["mmu"]["sync_feedback_state"], "neutral");
        assert_eq!(hare.status("printing")["mmu"]["flowguard"]["active"], true);
        assert_eq!(hare.status("printing")["mmu"]["sync_feedback_flow_rate"], 98.47);
        assert!(hare.status("printing")["mmu"]["encoder"].is_null());
    }

    #[test]
    fn the_boost_shows_only_while_printing() {
        let mut hare = homed(4);
        hare.apply(MmuCommand::FakeTangle { boosting: true }).unwrap();

        let printing = hare.status("printing")["mmu"].clone();
        assert_eq!(printing["tangle_prevention"]["boosted"], true);
        assert_eq!(printing["sync_feedback_state"], "tension");
        assert_eq!(printing["sensors"]["filament_tension"], true);
        assert_eq!(hare.status("standby")["mmu"]["tangle_prevention"]["boosted"], false);
    }

    #[test]
    fn the_way_of_the_filament_follows_loads_and_can_be_held_midway() {
        let mut hare = homed(4);
        hare.apply(MmuCommand::Select { gate: 0 }).unwrap();
        hare.apply(MmuCommand::Load).unwrap();
        assert_eq!(
            (mmu(&hare)["filament_pos"].clone(), mmu(&hare)["filament_direction"].clone()),
            (json!(10), json!(1))
        );
        assert_eq!(mmu(&hare)["sensors"]["toolhead"], true);

        hare.apply(MmuCommand::FakeFilament { position: 3, direction: 1 }).unwrap();
        let midway = mmu(&hare);
        assert_eq!(midway["filament_pos"], 3);
        assert!((midway["bowden_progress"].as_f64().unwrap() - 66.67).abs() < 0.1);
        assert_eq!(
            (midway["sensors"]["mmu_exit"].clone(), midway["sensors"]["extruder"].clone()),
            (json!(true), json!(false))
        );

        hare.apply(MmuCommand::Unload).unwrap();
        assert_eq!(
            (mmu(&hare)["filament_pos"].clone(), mmu(&hare)["filament_direction"].clone()),
            (json!(0), json!(-1))
        );
    }
}
