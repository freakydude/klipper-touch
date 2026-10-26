//! Happy Hare state from the Klipper objects `mmu` and `mmu_machine`.
//!
//! Field names and value ranges follow Happy Hare 4.1 (`mmu_sync_feedback.py`, `mmu_sensor_manager.py`) and were
//! checked against a real single-gate machine with a proportional sync-feedback buffer and no encoder.

use super::objects::ObjectStore;
use crate::sourced::{prefer_measured, Sourced};
use serde_json::Value;

/// Oldest Happy Hare the project supports; older versions report different fields.
const MIN_VERSION: (u32, u32) = (4, 1);

const PROPORTIONAL_SENSOR: &str = "filament_proportional";
pub const ORIGIN_ENCODER: &str = "encoder";
pub const ORIGIN_SYNC_FEEDBACK: &str = "sync feedback with proportional sensor";

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
    /// Spoolman spool of the gate (`gate_spool_id`); `None` for none (Happy Hare reports -1).
    pub spool_id: Option<u32>,
    /// Print temperature of the gate in °C (`gate_temperature`), from Spoolman if the gate has a spool.
    pub temperature: Option<f64>,
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
    /// Tangle prevention raises the gear current right now: it is on, a proportional sensor reports the tension and
    /// the tension is above its threshold.
    pub boosting: bool,
    pub trigger: String,
    pub reason: String,
    pub level: f64,
}

/// One word for the operator: a trigger wins over the boost of tangle prevention, which wins over monitoring, which
/// wins over ready.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowGuardState {
    Off,
    Triggered,
    Boosting,
    Monitoring,
    Ready,
}

impl FlowGuard {
    pub fn state(&self) -> FlowGuardState {
        if !self.enabled {
            FlowGuardState::Off
        } else if !self.trigger.is_empty() {
            FlowGuardState::Triggered
        } else if self.boosting {
            FlowGuardState::Boosting
        } else if self.active {
            FlowGuardState::Monitoring
        } else {
            FlowGuardState::Ready
        }
    }
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
    /// Gate of each tool (`ttg_map`).
    pub tool_to_gate: Vec<usize>,
    /// `off`, `readonly`, `push` or `pull`; empty if not reported.
    pub spoolman_support: String,
    pub sensors: Vec<MmuSensor>,
    pub sync_feedback: Option<SyncFeedback>,
    pub flowguard: Option<FlowGuard>,
    /// Flow rate in percent: measured by the encoder, else estimated by the sync-feedback controller with a
    /// proportional sensor. `None` if neither exists.
    pub flow_rate: Option<Sourced<f64>>,
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
    let sensors = read_sensors(mmu);
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
        tool_to_gate: mmu
            .get("ttg_map")
            .and_then(Value::as_array)
            .map(|map| map.iter().filter_map(|gate| gate.as_u64().map(|gate| gate as usize)).collect())
            .unwrap_or_default(),
        spoolman_support: text(mmu, "spoolman_support"),
        flow_rate: read_flow_rate(mmu, &sensors),
        flowguard: read_flowguard(mmu, &sensors),
        sensors,
        sync_feedback: read_sync_feedback(mmu),
    })
}

/// Happy Hare keeps `sync_feedback_flow_rate` at 100 for every buffer and only estimates it when an enabled
/// proportional sensor exists (`mmu_sync_feedback.py`), so the field alone says nothing without that sensor.
fn read_flow_rate(mmu: &Value, sensors: &[MmuSensor]) -> Option<Sourced<f64>> {
    let measured = mmu
        .get("encoder")
        .and_then(|encoder| encoder.get("flow_rate"))
        .and_then(Value::as_f64)
        .map(|rate| Sourced::measured(rate, ORIGIN_ENCODER));
    let estimated = mmu
        .get("sync_feedback_flow_rate")
        .and_then(Value::as_f64)
        .filter(|_| has_proportional_sensor(sensors))
        .map(|rate| Sourced::estimated(rate, ORIGIN_SYNC_FEEDBACK));
    prefer_measured(measured, estimated)
}

fn has_proportional_sensor(sensors: &[MmuSensor]) -> bool {
    sensors.iter().any(|sensor| sensor.name == PROPORTIONAL_SENSOR && sensor.triggered.is_some())
}

/// A Klipper filament sensor that belongs to the MMU itself (gates and buffer). `toolhead` and `extruder` stay common:
/// they sit behind the MMU and matter without one. `object` is the Klipper object name, for example
/// `filament_switch_sensor mmu_entry_0`.
pub fn is_mmu_klipper_sensor(object: &str) -> bool {
    let name = object.split_once(' ').map_or(object, |(_, name)| name);
    let name = name.rsplit_once(':').map_or(name, |(_, name)| name);
    name.starts_with("mmu_") || matches!(name, "filament_compression" | "filament_tension" | "filament_proportional")
}

/// `major.minor` of a version such as `4.1.0` or `v4.10`.
fn parse_version(text: &str) -> Option<(u32, u32)> {
    let mut parts = text.trim().trim_start_matches('v').split('.');
    Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
}

fn read_gates(mmu: &Value, num_gates: usize) -> Vec<MmuGate> {
    let list = |field: &str| mmu.get(field).and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default();
    let (status, names, materials, colors, rgb, ttg_map, spool_ids, temperatures) = (
        list("gate_status"),
        list("gate_filament_name"),
        list("gate_material"),
        list("gate_color"),
        list("gate_color_rgb"),
        list("ttg_map"),
        list("gate_spool_id"),
        list("gate_temperature"),
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
            spool_id: spool_ids.get(index).and_then(Value::as_i64).and_then(|id| u32::try_from(id).ok()),
            temperature: temperatures.get(index).and_then(Value::as_f64).filter(|degrees| *degrees > 0.0),
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

/// Tangle prevention counts only with a proportional sensor, as Happy Hare arms it only then.
fn read_flowguard(mmu: &Value, sensors: &[MmuSensor]) -> Option<FlowGuard> {
    let flowguard = mmu.get("flowguard").filter(|value| value.is_object())?;
    let tangle_prevention = mmu.get("tangle_prevention").filter(|value| value.is_object());
    let boosting = tangle_prevention
        .is_some_and(|prevention| flag(prevention, "enabled") && flag(prevention, "boosted"))
        && has_proportional_sensor(sensors);
    Some(FlowGuard {
        enabled: flag(flowguard, "enabled"),
        active: flag(flowguard, "active"),
        boosting,
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
        assert_eq!(mmu.flow_rate, None);
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
    fn flow_rate_comes_from_the_encoder_dictionary() {
        let store = store(json!({ "mmu": { "encoder": { "flow_rate": 97.5, "encoder_pos": 12.0 } } }));
        assert_eq!(read_mmu(&store).unwrap().flow_rate, Some(Sourced::measured(97.5, ORIGIN_ENCODER)));
    }

    #[test]
    fn flow_rate_without_encoder_is_estimated_by_the_proportional_sensor() {
        let store = store(json!({ "mmu": {
            "encoder": null, "sensors": { "filament_proportional": false }, "sync_feedback_flow_rate": 93.4
        } }));
        assert_eq!(read_mmu(&store).unwrap().flow_rate, Some(Sourced::estimated(93.4, ORIGIN_SYNC_FEEDBACK)));
    }

    #[test]
    fn encoder_flow_rate_wins_over_the_estimate() {
        let store = store(json!({ "mmu": {
            "encoder": { "flow_rate": 97.5 }, "sensors": { "filament_proportional": true },
            "sync_feedback_flow_rate": 93.4
        } }));
        assert_eq!(read_mmu(&store).unwrap().flow_rate, Some(Sourced::measured(97.5, ORIGIN_ENCODER)));
    }

    #[test]
    fn buffer_without_proportional_sensor_has_no_flow_rate() {
        // Happy Hare reports the initial 100 for tension/compression buffers, which is no estimate.
        let store = store(json!({ "mmu": {
            "sensors": { "filament_tension": true }, "sync_feedback_flow_rate": 100.0
        } }));
        assert_eq!(read_mmu(&store).unwrap().flow_rate, None);
    }

    #[test]
    fn disabled_proportional_sensor_has_no_flow_rate() {
        let store = store(json!({ "mmu": {
            "sensors": { "filament_proportional": null }, "sync_feedback_flow_rate": 100.0
        } }));
        assert_eq!(read_mmu(&store).unwrap().flow_rate, None);
    }

    #[test]
    fn flow_rate_is_missing_without_buffer() {
        let store = store(json!({ "mmu": {
            "sensors": { "filament_proportional": false }, "sync_feedback_flow_rate": null
        } }));
        assert_eq!(read_mmu(&store).unwrap().flow_rate, None);
    }

    #[test]
    fn mmu_sensors_are_told_apart_from_common_klipper_sensors() {
        assert!(is_mmu_klipper_sensor("filament_switch_sensor mmu_entry_0"));
        assert!(is_mmu_klipper_sensor("filament_switch_sensor filament_tension"));
        assert!(is_mmu_klipper_sensor("filament_switch_sensor unit0:mmu_exit_1"));
        assert!(!is_mmu_klipper_sensor("filament_switch_sensor toolhead"));
        assert!(!is_mmu_klipper_sensor("filament_switch_sensor extruder"));
        assert!(!is_mmu_klipper_sensor("filament_motion_sensor extruder_encoder"));
    }

    #[test]
    fn flowguard_state_is_one_word_where_a_trigger_wins() {
        let flowguard = |enabled, active, trigger: &str| FlowGuard {
            enabled,
            active,
            trigger: trigger.into(),
            ..FlowGuard::default()
        };

        assert_eq!(flowguard(false, true, "clog").state(), FlowGuardState::Off);
        assert_eq!(flowguard(true, true, "clog").state(), FlowGuardState::Triggered);
        assert_eq!(flowguard(true, true, "").state(), FlowGuardState::Monitoring);
        assert_eq!(flowguard(true, false, "").state(), FlowGuardState::Ready);
    }

    #[test]
    fn boosting_tangle_prevention_shows_between_trigger_and_monitoring() {
        let boosting = |trigger: &str| FlowGuard {
            enabled: true,
            active: true,
            boosting: true,
            trigger: trigger.into(),
            ..FlowGuard::default()
        };

        assert_eq!(boosting("").state(), FlowGuardState::Boosting);
        assert_eq!(boosting("tangle").state(), FlowGuardState::Triggered);
        assert_eq!(FlowGuard { boosting: true, ..FlowGuard::default() }.state(), FlowGuardState::Off);
    }

    #[test]
    fn the_boost_needs_the_switch_and_a_proportional_sensor() {
        let boosting = |sensors: Value, enabled: bool| {
            read_mmu(&store(json!({ "mmu": {
                "sensors": sensors,
                "flowguard": { "enabled": true, "active": true },
                "tangle_prevention": { "enabled": enabled, "active": true, "boosted": true, "threshold": 0.3 }
            } })))
            .unwrap()
            .flowguard
            .unwrap()
            .boosting
        };

        assert!(boosting(json!({ "filament_proportional": false }), true));
        assert!(!boosting(json!({ "filament_proportional": false }), false));
        assert!(!boosting(json!({ "filament_tension": true }), true));
    }
}
