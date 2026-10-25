//! Happy Hare state from the Klipper objects `mmu` and `mmu_machine`.
//!
//! Field names and value ranges follow Happy Hare 4.1 (`mmu_sync_feedback.py`, `mmu_sensor_manager.py`) and were
//! checked against a real single-gate machine with a proportional sync-feedback buffer and no encoder.

use super::objects::ObjectStore;
use serde_json::Value;

/// Oldest Happy Hare the project supports; older versions report different fields.
const MIN_VERSION: (u32, u32) = (4, 1);

/// Happy Hare 4.1 still lists the Happy Hare 3 names of the gate sensors in `mmu.sensors` as aliases.
const LEGACY_SENSOR_NAMES: [&str; 3] = ["mmu_pre_gate", "mmu_gear", "mmu_gate"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GateAvailability {
    #[default]
    Unknown,
    Empty,
    Available,
    /// Filament is available from the buffer of the gate.
    Buffered,
}

impl GateAvailability {
    fn parse(value: i64) -> Self {
        match value {
            0 => Self::Empty,
            1 => Self::Available,
            2 => Self::Buffered,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadedFilament {
    Loaded,
    Unloaded,
    #[default]
    Unknown,
}

impl LoadedFilament {
    fn parse(text: &str) -> Self {
        match text {
            "Loaded" => Self::Loaded,
            "Unloaded" => Self::Unloaded,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MmuGate {
    pub index: usize,
    pub availability: GateAvailability,
    pub name: String,
    pub material: String,
    /// Only known when the gate has a colour set (`gate_color` is not empty).
    pub color: Option<[u8; 3]>,
    /// First tool that is mapped to this gate (`ttg_map`).
    pub tool: Option<usize>,
}

/// A filament sensor of the MMU as reported in `mmu.sensors`; `triggered` is `None` when the sensor is disabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MmuSensor {
    pub name: String,
    pub triggered: Option<bool>,
}

/// Sync-feedback state as Happy Hare reports it (`get_sync_feedback_string`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncFeedbackState {
    Compressed,
    Neutral,
    Tension,
    /// Enabled, but the controller is not active (for example while not printing).
    Inactive,
    #[default]
    Disabled,
}

impl SyncFeedbackState {
    fn parse(text: &str) -> Self {
        match text {
            "compressed" => Self::Compressed,
            "neutral" => Self::Neutral,
            "tension" => Self::Tension,
            "inactive" => Self::Inactive,
            _ => Self::Disabled,
        }
    }
}

/// Buffer of the gear stepper: `bias` runs from -1 (tension) over 0 (neutral) to +1 (compressed).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SyncFeedback {
    pub state: SyncFeedbackState,
    pub bias: f64,
}

/// Clog and tangle protection; it works with the buffer controller or with an encoder.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FlowGuard {
    pub enabled: bool,
    pub active: bool,
    pub trigger: String,
    pub reason: String,
    pub level: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MmuStatus {
    /// `mmu_machine.happy_hare_version`, for example `4.1.0`; empty when not reported.
    pub version: String,
    pub enabled: bool,
    pub num_gates: usize,
    pub num_units: usize,
    /// Selected gate; -1 unknown, -2 bypass.
    pub gate: i32,
    /// Selected tool; -1 unknown, -2 bypass.
    pub tool: i32,
    pub action: String,
    pub operation: String,
    pub print_state: String,
    pub reason_for_pause: String,
    pub filament: LoadedFilament,
    pub homed: bool,
    pub paused: bool,
    pub locked: bool,
    pub gates: Vec<MmuGate>,
    pub sensors: Vec<MmuSensor>,
    pub sync_feedback: Option<SyncFeedback>,
    pub flowguard: Option<FlowGuard>,
    /// Flow rate in percent from the encoder; `None` without an encoder.
    pub encoder_flow_rate: Option<f64>,
}

impl MmuStatus {
    /// Happy Hare 4.1 or newer; the rest of this model relies on its field names.
    pub fn is_supported(&self) -> bool {
        parse_version(&self.version).is_some_and(|version| version >= MIN_VERSION)
    }

    pub fn selected_gate(&self) -> Option<&MmuGate> {
        usize::try_from(self.gate).ok().and_then(|index| self.gates.get(index))
    }
}

/// `None` without Happy Hare (no `mmu` object).
pub(super) fn read_mmu(store: &ObjectStore) -> Option<MmuStatus> {
    let mmu = store.object("mmu")?;
    let machine = store.object("mmu_machine").unwrap_or(&Value::Null);
    let num_gates = count(mmu, "num_gates").max(count(machine, "num_gates"));
    Some(MmuStatus {
        version: text(machine, "happy_hare_version"),
        enabled: flag(mmu, "enabled"),
        num_gates,
        num_units: count(machine, "num_units").max(1),
        gate: integer(mmu, "gate", -1),
        tool: integer(mmu, "tool", -1),
        action: text(mmu, "action"),
        operation: text(mmu, "operation"),
        print_state: text(mmu, "print_state"),
        reason_for_pause: text(mmu, "reason_for_pause"),
        filament: LoadedFilament::parse(&text(mmu, "filament")),
        homed: flag(mmu, "is_homed"),
        paused: flag(mmu, "is_paused"),
        locked: flag(mmu, "is_locked"),
        gates: read_gates(mmu, num_gates),
        sensors: read_sensors(mmu),
        sync_feedback: read_sync_feedback(mmu),
        flowguard: read_flowguard(mmu),
        encoder_flow_rate: mmu.get("encoder").and_then(|encoder| encoder.get("flow_rate")).and_then(Value::as_f64),
    })
}

/// `major.minor` of a version such as `4.1.0` or `v4.10`.
fn parse_version(text: &str) -> Option<(u32, u32)> {
    let mut parts = text.trim().trim_start_matches('v').split('.');
    Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
}

fn read_gates(mmu: &Value, num_gates: usize) -> Vec<MmuGate> {
    let list = |field: &str| mmu.get(field).and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default();
    let (status, names, materials, colors, rgb, ttg_map) = (
        list("gate_status"),
        list("gate_filament_name"),
        list("gate_material"),
        list("gate_color"),
        list("gate_color_rgb"),
        list("ttg_map"),
    );
    let string_at =
        |values: &[Value], index: usize| values.get(index).and_then(Value::as_str).unwrap_or_default().to_owned();
    (0..num_gates)
        .map(|index| MmuGate {
            index,
            availability: status
                .get(index)
                .and_then(Value::as_i64)
                .map_or(GateAvailability::Unknown, GateAvailability::parse),
            name: string_at(names, index),
            material: string_at(materials, index),
            color: if string_at(colors, index).is_empty() { None } else { rgb.get(index).and_then(color_from_rgb) },
            tool: ttg_map.iter().position(|gate| gate.as_u64() == Some(index as u64)),
        })
        .collect()
}

/// Happy Hare reports colours as three floats from 0 to 1.
fn color_from_rgb(value: &Value) -> Option<[u8; 3]> {
    let channels = value.as_array()?;
    let channel = |index: usize| {
        channels.get(index).and_then(Value::as_f64).map(|part| (part.clamp(0.0, 1.0) * 255.0).round() as u8)
    };
    Some([channel(0)?, channel(1)?, channel(2)?])
}

fn read_sensors(mmu: &Value) -> Vec<MmuSensor> {
    mmu.get("sensors")
        .and_then(Value::as_object)
        .map(|sensors| {
            sensors
                .iter()
                .filter(|(name, _)| !LEGACY_SENSOR_NAMES.contains(&name.as_str()))
                .map(|(name, state)| MmuSensor { name: name.clone(), triggered: state.as_bool() })
                .collect()
        })
        .unwrap_or_default()
}

/// Only present with a buffer; the modelled bias is the better value while printing.
fn read_sync_feedback(mmu: &Value) -> Option<SyncFeedback> {
    let state = mmu.get("sync_feedback_state").and_then(Value::as_str)?;
    let bias = mmu
        .get("sync_feedback_bias_modelled")
        .and_then(Value::as_f64)
        .or_else(|| mmu.get("sync_feedback_bias_raw").and_then(Value::as_f64))
        .unwrap_or_default();
    Some(SyncFeedback { state: SyncFeedbackState::parse(state), bias: bias.clamp(-1.0, 1.0) })
}

fn read_flowguard(mmu: &Value) -> Option<FlowGuard> {
    let flowguard = mmu.get("flowguard").filter(|value| value.is_object())?;
    Some(FlowGuard {
        enabled: flag(flowguard, "enabled"),
        active: flag(flowguard, "active"),
        trigger: text(flowguard, "trigger"),
        reason: text(flowguard, "reason"),
        level: flowguard.get("level").and_then(Value::as_f64).unwrap_or_default(),
    })
}

fn text(object: &Value, field: &str) -> String {
    object.get(field).and_then(Value::as_str).unwrap_or_default().to_owned()
}

fn flag(object: &Value, field: &str) -> bool {
    match object.get(field) {
        Some(Value::Bool(value)) => *value,
        Some(Value::Number(value)) => value.as_i64().is_some_and(|number| number != 0),
        _ => false,
    }
}

fn integer(object: &Value, field: &str, default: i32) -> i32 {
    object.get(field).and_then(Value::as_i64).and_then(|value| i32::try_from(value).ok()).unwrap_or(default)
}

fn count(object: &Value, field: &str) -> usize {
    object.get(field).and_then(Value::as_u64).and_then(|value| usize::try_from(value).ok()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn store(status: Value) -> ObjectStore {
        let mut store = ObjectStore::default();
        store.replace(&status);
        store
    }

    /// `mmu` and `mmu_machine` of a real single-gate machine with a proportional buffer and no encoder (Happy Hare 4.1.0).
    fn single_gate_machine() -> ObjectStore {
        store(json!({
            "mmu": {
                "enabled": true, "num_gates": 1, "gate": 0, "tool": 0, "action": "Idle", "operation": "",
                "print_state": "initialized", "reason_for_pause": "", "filament": "Unloaded", "is_homed": true,
                "is_paused": false, "is_locked": false,
                "gate_status": [0], "gate_color": [""], "gate_color_rgb": [[0.0, 0.0, 0.0]],
                "gate_material": [""], "gate_filament_name": [""], "ttg_map": [0],
                "sensors": {
                    "mmu_entry": false, "mmu_exit": false, "filament_compression": false, "filament_tension": true,
                    "filament_proportional": false, "extruder": false, "toolhead": false,
                    "mmu_pre_gate": false, "mmu_gear": false
                },
                "sync_feedback_state": "inactive", "sync_feedback_enabled": 1,
                "sync_feedback_bias_raw": -1.0, "sync_feedback_bias_modelled": -1.0,
                "flowguard": { "trigger": "", "reason": "", "level": 0.0, "max_clog": 0.0, "max_tangle": 0.0,
                               "active": false, "enabled": true },
                "encoder": null
            },
            "mmu_machine": { "happy_hare_version": "4.1.0", "num_units": 1, "num_gates": 1 }
        }))
    }

    #[test]
    fn no_happy_hare_without_the_mmu_object() {
        assert_eq!(read_mmu(&store(json!({ "extruder": {} }))), None);
    }

    #[test]
    fn reads_a_single_gate_machine_with_a_buffer() {
        let mmu = read_mmu(&single_gate_machine()).unwrap();

        assert!(mmu.enabled && mmu.homed && !mmu.paused);
        assert_eq!((mmu.num_gates, mmu.num_units, mmu.gate, mmu.tool), (1, 1, 0, 0));
        assert_eq!(mmu.filament, LoadedFilament::Unloaded);
        assert_eq!(mmu.encoder_flow_rate, None);
        assert_eq!(
            mmu.gates,
            vec![MmuGate { index: 0, availability: GateAvailability::Empty, tool: Some(0), ..MmuGate::default() }]
        );
        assert_eq!(mmu.sync_feedback, Some(SyncFeedback { state: SyncFeedbackState::Inactive, bias: -1.0 }));
        let flowguard = mmu.flowguard.unwrap();
        assert!(flowguard.enabled && !flowguard.active);
    }

    #[test]
    fn only_happy_hare_4_1_or_newer_is_supported() {
        let with_version = |version: &str| {
            read_mmu(&store(json!({ "mmu": {}, "mmu_machine": { "happy_hare_version": version } }))).unwrap()
        };

        assert!(read_mmu(&single_gate_machine()).unwrap().is_supported());
        assert!(with_version("4.1.0").is_supported() && with_version("v4.10").is_supported());
        assert!(with_version("5.0").is_supported());
        assert!(!with_version("4.0.2").is_supported() && !with_version("3.4").is_supported());
        assert!(!with_version("").is_supported() && !with_version("beta").is_supported());
    }

    #[test]
    fn sensors_skip_the_legacy_names_and_keep_disabled_ones() {
        let sensors = read_mmu(&single_gate_machine()).unwrap().sensors;
        let names: Vec<&str> = sensors.iter().map(|sensor| sensor.name.as_str()).collect();

        assert!(!names.contains(&"mmu_pre_gate") && !names.contains(&"mmu_gear"));
        assert!(names.contains(&"mmu_entry") && names.contains(&"filament_proportional"));
        let tension = sensors.iter().find(|sensor| sensor.name == "filament_tension").unwrap();
        assert_eq!(tension.triggered, Some(true));

        let disabled = store(json!({ "mmu": { "sensors": { "toolhead": null } } }));
        assert_eq!(read_mmu(&disabled).unwrap().sensors, vec![MmuSensor { name: "toolhead".into(), triggered: None }]);
    }

    #[test]
    fn reads_gates_with_colours_and_tool_mapping() {
        let store = store(json!({
            "mmu": {
                "num_gates": 3, "gate": 1,
                "gate_status": [1, 2, -1], "gate_filament_name": ["Red PLA", "", "Blue"],
                "gate_material": ["PLA", "", "PETG"], "gate_color": ["red", "", "blue"],
                "gate_color_rgb": [[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
                "ttg_map": [2, 0, 1]
            }
        }));

        let mmu = read_mmu(&store).unwrap();

        assert_eq!(mmu.selected_gate().map(|gate| gate.index), Some(1));
        assert_eq!(mmu.gates[0].availability, GateAvailability::Available);
        assert_eq!(mmu.gates[0].color, Some([255, 0, 0]));
        assert_eq!(mmu.gates[0].tool, Some(1));
        assert_eq!(mmu.gates[1].availability, GateAvailability::Buffered);
        assert_eq!(mmu.gates[1].color, None);
        assert_eq!(mmu.gates[2].availability, GateAvailability::Unknown);
        assert_eq!((mmu.gates[2].name.as_str(), mmu.gates[2].material.as_str()), ("Blue", "PETG"));
        assert_eq!(mmu.gates[2].tool, Some(0));
    }

    #[test]
    fn bypass_and_unknown_gate_select_nothing() {
        let bypass = read_mmu(&store(json!({ "mmu": { "num_gates": 2, "gate": -2 } }))).unwrap();
        assert_eq!(bypass.gate, -2);
        assert!(bypass.selected_gate().is_none());
    }

    #[test]
    fn buffer_states_follow_the_happy_hare_strings() {
        let state = |text: &str| {
            read_mmu(&store(json!({ "mmu": { "sync_feedback_state": text, "sync_feedback_bias_raw": 0.4 } })))
                .unwrap()
                .sync_feedback
                .unwrap()
        };

        assert_eq!(state("compressed").state, SyncFeedbackState::Compressed);
        assert_eq!(state("tension").state, SyncFeedbackState::Tension);
        assert_eq!(state("neutral").state, SyncFeedbackState::Neutral);
        assert_eq!(state("something else").state, SyncFeedbackState::Disabled);
        assert_eq!(state("neutral").bias, 0.4);
        assert_eq!(read_mmu(&store(json!({ "mmu": { "sync_feedback_state": null } }))).unwrap().sync_feedback, None);
    }

    #[test]
    fn encoder_flow_rate_comes_from_the_encoder_dictionary() {
        let store = store(json!({ "mmu": { "encoder": { "flow_rate": 97.5, "encoder_pos": 12.0 } } }));
        assert_eq!(read_mmu(&store).unwrap().encoder_flow_rate, Some(97.5));
    }
}
