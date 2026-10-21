//! Vereinfachte Happy-Hare-Simulation (Objekte `mmu` und `mmu_machine`).
//!
//! Feldnamen folgen der Happy-Hare-v4-Referenz "Printer Variables". Vorgaenge laufen ohne
//! Zeitverlauf: `action` ist immer `Idle`, Laden und Entladen sind sofort abgeschlossen.

use crate::gcode::{CommandError, MmuCommand};
use serde_json::{json, Map, Value};

const VERSION: &str = "4.1.0";
const NONE_SELECTED: i32 = -1;
const GATE_EMPTY: i32 = 0;
const GATE_AVAILABLE: i32 = 1;
const FILAMENT_POSITION_UNLOADED: i32 = 0;
const FILAMENT_POSITION_LOADED: i32 = 10;
const UNIT_NAME: &str = "unit0";

struct Spool {
    name: &'static str,
    material: &'static str,
    vendor: &'static str,
    color: &'static str,
    rgb: [f64; 3],
    temperature: i32,
}

const SPOOLS: [Spool; 4] = [
    Spool { name: "Red PLA", material: "PLA", vendor: "Fake", color: "red", rgb: [1.0, 0.0, 0.0], temperature: 210 },
    Spool { name: "Green PETG", material: "PETG", vendor: "Fake", color: "green", rgb: [0.0, 0.8, 0.0], temperature: 235 },
    Spool { name: "Blue PLA", material: "PLA", vendor: "Fake", color: "blue", rgb: [0.0, 0.2, 1.0], temperature: 210 },
    Spool { name: "Black ABS", material: "ABS", vendor: "Fake", color: "black", rgb: [0.1, 0.1, 0.1], temperature: 250 },
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
}

impl HappyHare {
    pub fn new(gates: usize) -> Self {
        // Das letzte Gate bleibt leer, damit die UI auch einen leeren Slot zeigen muss.
        let mut gate_status = vec![GATE_AVAILABLE; gates];
        if gates > 2 {
            gate_status[gates - 1] = GATE_EMPTY;
        }
        Self {
            gates,
            homed: false,
            gate: NONE_SELECTED,
            tool: NONE_SELECTED,
            loaded: false,
            toolchanges: 0,
            tool_to_gate: (0..gates).collect(),
            gate_status,
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
            MmuCommand::Load { gate } => self.load(gate),
            MmuCommand::Unload => self.unload(),
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
        let gate = *self
            .tool_to_gate
            .get(tool)
            .ok_or_else(|| CommandError::new(format!("Tool T{tool} does not exist")))?;
        self.require_filament_at(gate)?;
        self.select(gate);
        self.loaded = true;
        self.tool = tool as i32;
        self.toolchanges += 1;
        Ok(())
    }

    fn load(&mut self, gate: Option<usize>) -> Result<(), CommandError> {
        self.require_homed()?;
        if self.loaded {
            return Err(CommandError::new("Filament is already loaded"));
        }
        if let Some(gate) = gate {
            self.check_gate(gate)?;
            self.select(gate);
        }
        let selected = usize::try_from(self.gate).map_err(|_| CommandError::new("No gate selected"))?;
        self.require_filament_at(selected)?;
        self.loaded = true;
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
        self.tool = self
            .tool_to_gate
            .iter()
            .position(|mapped| *mapped == gate)
            .map_or(NONE_SELECTED, |tool| tool as i32);
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

    fn spool(&self, gate: usize) -> &'static Spool {
        &SPOOLS[gate % SPOOLS.len()]
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
                    "spool_id": -1,
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
            "encoder": self.encoder_json(),
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

        for command in [MmuCommand::Select { gate: 1 }, MmuCommand::ChangeTool { tool: 1 }, MmuCommand::Load { gate: None }] {
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
    fn load_needs_a_selected_gate() {
        let mut hare = homed(4);

        assert_eq!(hare.apply(MmuCommand::Load { gate: None }).unwrap_err().message(), "No gate selected");

        hare.apply(MmuCommand::Load { gate: Some(1) }).unwrap();
        assert_eq!(mmu(&hare)["filament"], "Loaded");
    }

    #[test]
    fn status_registers_all_happy_hare_objects() {
        let objects = HappyHare::new(4).status("standby");

        let names: Vec<&str> = objects.keys().map(String::as_str).collect();
        assert_eq!(names, ["mmu", "mmu_encoder unit0", "mmu_extruder extruder", "mmu_machine", "mmu_stepper unit0_gear"]);
        assert_eq!(objects["mmu_machine"]["unit_0"]["num_gates"], 4);
    }
}
