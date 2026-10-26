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
const UNIT_NAME: &str = "unit0";

pub(crate) struct Spool {
    pub(crate) name: &'static str,
    pub(crate) material: &'static str,
    pub(crate) vendor: &'static str,
    pub(crate) color: &'static str,
    pub(crate) rgb: [f64; 3],
    pub(crate) temperature: i32,
}

pub(crate) const SPOOLS: [Spool; 4] = [
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
];

#[derive(Debug, Clone)]
pub struct HappyHare {
    gates: usize,
    homed: bool,
    gate: i32,
    tool: i32,
    loaded: bool,
    toolchanges: u32,
    tool_to_gate: Vec<usize>,
    gate_status: Vec<i32>,
    /// Spoolman spool of each gate.
    gate_spool: Vec<Option<u32>>,
}

impl HappyHare {
    pub fn new(gates: usize) -> Self {
        // The last gate stays empty so the UI also has to show an empty slot.
        let mut gate_status = vec![GATE_AVAILABLE; gates];
        if gates > 2 {
            gate_status[gates - 1] = GATE_EMPTY;
        }
        let gate_spool = gate_status
            .iter()
            .enumerate()
            .map(|(gate, status)| (*status == GATE_AVAILABLE).then(|| gate as u32 + 1))
            .collect();
        Self {
            gates,
            homed: false,
            gate: NONE_SELECTED,
            tool: NONE_SELECTED,
            loaded: false,
            toolchanges: 0,
            tool_to_gate: (0..gates).collect(),
            gate_status,
            gate_spool,
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
        }
    }

    pub fn status(&self, print_state: &str) -> Map<String, Value> {
        let mut objects = Map::new();
        objects.insert("mmu".to_owned(), self.mmu_json(print_state));
        objects.insert("mmu_machine".to_owned(), self.machine_json());
        objects.insert(format!("mmu_encoder {UNIT_NAME}"), self.encoder_json());
        objects.insert("mmu_extruder extruder".to_owned(), json!({}));
        objects.insert(format!("mmu_stepper {UNIT_NAME}_gear"), json!({}));
        objects
    }

    pub fn config_sections(&self) -> Vec<(String, Value)> {
        vec![
            ("mmu_machine".to_owned(), json!({ "num_units": 1 })),
            ("mmu".to_owned(), json!({ "enabled": true })),
            (format!("mmu_encoder {UNIT_NAME}"), json!({})),
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
        self.loaded = true;
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
        self.loaded = true;
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
        self.loaded = false;
        self.gate_status[target] = GATE_EMPTY;
        Ok(())
    }

    fn unload(&mut self) -> Result<(), CommandError> {
        self.require_homed()?;
        if !self.loaded {
            return Err(CommandError::new("Filament is not loaded"));
        }
        self.loaded = false;
        Ok(())
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
            Some(id) => &SPOOLS[(id as usize - 1) % SPOOLS.len()],
            None => &SPOOLS[gate % SPOOLS.len()],
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
        json!({
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
            "filament_pos": if self.loaded { FILAMENT_POSITION_LOADED } else { FILAMENT_POSITION_UNLOADED },
            "action": "Idle",
            "reason_for_pause": "",
            "is_paused": false,
            "is_locked": false,
            "sensors": self.sensors_json(),
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
            "encoder": self.encoder_json(),
        })
    }

    /// `mmu.sensors` with generic names; the Happy Hare 3 aliases are included as Happy Hare 4.1 does for older UIs.
    fn sensors_json(&self) -> Value {
        let at_entry = usize::try_from(self.gate)
            .ok()
            .and_then(|gate| self.gate_status.get(gate))
            .is_some_and(|status| *status == GATE_AVAILABLE);
        json!({
            "mmu_entry": at_entry,
            "mmu_exit": self.loaded,
            "extruder": self.loaded,
            "toolhead": self.loaded,
            "mmu_pre_gate": at_entry,
            "mmu_gear": self.loaded,
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

    fn encoder_json(&self) -> Value {
        json!({ "encoder_pos": 0.0, "enabled": true, "flow_rate": 100 })
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
        assert_eq!(status["active_filament"]["filament_name"], "Blue PLA");
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
        assert_eq!(mmu(&hare)["gate_spool_id"], json!([1, 2, 3, -1]));

        hare.apply(MmuCommand::GateMap { gate: 1, spool_id: Some(4) }).unwrap();
        hare.apply(MmuCommand::GateMap { gate: 0, spool_id: Some(-1) }).unwrap();

        let status = mmu(&hare);
        assert_eq!(status["gate_spool_id"], json!([-1, 4, 3, -1]));
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

        assert_eq!(hare.selected_spool(), Some(2));
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
        assert_eq!(
            names,
            ["mmu", "mmu_encoder unit0", "mmu_extruder extruder", "mmu_machine", "mmu_stepper unit0_gear"]
        );
        assert_eq!(objects["mmu_machine"]["unit_0"]["num_gates"], 4);
    }
}
