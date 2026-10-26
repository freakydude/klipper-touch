//! Spoolman: the spools, and the lanes they sit in. A lane is one place for a spool: a gate of Happy Hare, or the one
//! spool of a printer without MMU (the active spool of Moonraker).

use crate::files::FileMetadata;
use crate::printer::MmuStatus;
use serde_json::Value;
use std::collections::HashMap;

/// A spool with less than this share of its start weight left is flagged.
const LOW_SHARE: f64 = 0.15;
/// A target that is further off than this from the temperature of the spool gets a hint, in °C.
const TEMPERATURE_TOLERANCE: f64 = 5.0;
/// Colours further apart than this (distance in RGB, at most 441) count as different.
const COLOR_DISTANCE: f64 = 120.0;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Spool {
    pub id: u32,
    pub vendor: String,
    pub filament_name: String,
    pub material: String,
    pub color: Option<[u8; 3]>,
    /// Grams of filament left.
    pub remaining_weight: Option<f64>,
    /// Grams of filament at the start.
    pub initial_weight: Option<f64>,
    /// Weight of the empty spool in grams.
    pub spool_weight: Option<f64>,
    /// g/cm³.
    pub density: Option<f64>,
    /// Filament diameter in mm.
    pub diameter: Option<f64>,
    /// Recommended temperatures in °C (`settings_extruder_temp`, `settings_bed_temp`), if the filament has them.
    pub extruder_temp: Option<f64>,
    pub bed_temp: Option<f64>,
    /// Where Spoolman thinks the spool is, e.g. `voron @ MMU Gate:0`.
    pub location: String,
    /// ISO time, sorts as text; empty if the spool was never used.
    pub last_used: String,
}

impl Spool {
    /// `None` without an id, so a broken entry is skipped.
    pub fn from_json(value: &Value) -> Option<Self> {
        let id = u32::try_from(value.get("id")?.as_u64()?).ok()?;
        let filament = value.get("filament").unwrap_or(&Value::Null);
        let number = |object: &Value, field: &str| object.get(field).and_then(Value::as_f64);
        let text =
            |object: &Value, field: &str| object.get(field).and_then(Value::as_str).unwrap_or_default().to_owned();
        Some(Self {
            id,
            vendor: filament.pointer("/vendor/name").and_then(Value::as_str).unwrap_or_default().to_owned(),
            filament_name: text(filament, "name"),
            material: text(filament, "material"),
            color: filament.get("color_hex").and_then(Value::as_str).and_then(parse_color),
            remaining_weight: number(value, "remaining_weight"),
            initial_weight: number(value, "initial_weight").or_else(|| number(filament, "weight")),
            spool_weight: number(value, "spool_weight").or_else(|| number(filament, "spool_weight")),
            density: number(filament, "density"),
            diameter: number(filament, "diameter"),
            extruder_temp: number(filament, "settings_extruder_temp"),
            bed_temp: number(filament, "settings_bed_temp"),
            location: text(value, "location"),
            last_used: text(value, "last_used"),
        })
    }

    /// Vendor and filament name, e.g. `Geeetech Red`.
    pub fn title(&self) -> String {
        [self.vendor.as_str(), self.filament_name.as_str()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Share of the start weight that is left, 0 to 1.
    pub fn remaining_share(&self) -> Option<f64> {
        let (left, start) = (self.remaining_weight?, self.initial_weight.filter(|weight| *weight > 0.0)?);
        Some((left / start).clamp(0.0, 1.0))
    }

    pub fn is_low(&self) -> bool {
        self.remaining_share().is_some_and(|share| share < LOW_SHARE)
    }

    /// Weight in grams of a length of this filament; `None` without diameter and density.
    pub fn weight_of_length(&self, millimeters: f64) -> Option<f64> {
        let (diameter, density) = (self.diameter.filter(|d| *d > 0.0)?, self.density.filter(|d| *d > 0.0)?);
        let cubic_millimeters = std::f64::consts::PI * (diameter / 2.0).powi(2) * millimeters.max(0.0);
        Some(cubic_millimeters / 1000.0 * density)
    }
}

/// `E31F2B` or `#E31F2B`.
pub fn parse_color(text: &str) -> Option<[u8; 3]> {
    let hex = text.trim().trim_start_matches('#');
    if hex.len() < 6 || !hex.is_char_boundary(6) {
        return None;
    }
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    Some([channel(0..2)?, channel(2..4)?, channel(4..6)?])
}

/// What Moonraker knows about Spoolman, and the spools the printer needs now.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpoolmanState {
    /// Moonraker is connected to Spoolman.
    pub connected: bool,
    /// Usage Moonraker could not report yet.
    pub pending_reports: usize,
    /// The spool Moonraker tracks the filament use of.
    pub active_spool_id: Option<u32>,
    pub spools: HashMap<u32, Spool>,
}

/// `server.spoolman.status`.
pub fn parse_status(result: &Value) -> (bool, usize, Option<u32>) {
    (
        result.get("spoolman_connected").and_then(Value::as_bool).unwrap_or_default(),
        result.get("pending_reports").and_then(Value::as_array).map_or(0, Vec::len),
        result.get("spool_id").and_then(Value::as_u64).and_then(|id| u32::try_from(id).ok()),
    )
}

/// The answer of `server.spoolman.proxy`: the version 2 response has `response` and `error`, the old one is the
/// answer of Spoolman itself.
pub fn proxy_response(result: &Value) -> Result<Value, String> {
    if result.get("response").is_some() || result.get("error").is_some() {
        return match result.get("error").filter(|error| !error.is_null()) {
            Some(error) => Err(error.get("message").and_then(Value::as_str).unwrap_or("Spoolman error").to_owned()),
            None => Ok(result.get("response").cloned().unwrap_or(Value::Null)),
        };
    }
    Ok(result.clone())
}

pub fn parse_spool_list(value: &Value) -> Vec<Spool> {
    value.as_array().map(|spools| spools.iter().filter_map(Spool::from_json).collect()).unwrap_or_default()
}

/// One place for a spool.
#[derive(Debug, Clone, PartialEq)]
pub struct Lane {
    /// `None` without MMU.
    pub gate: Option<usize>,
    /// First tool that is mapped to the gate.
    pub tool: Option<usize>,
    pub spool_id: Option<u32>,
    /// The spool, once Spoolman has answered.
    pub spool: Option<Spool>,
    /// The filament of this lane is the one in use: the selected gate, or the only spool.
    pub active: bool,
}

/// The lanes of the printer: a gate each with MMU, otherwise the one active spool.
pub fn lanes(mmu: Option<&MmuStatus>, state: &SpoolmanState) -> Vec<Lane> {
    let spool = |id: Option<u32>| id.and_then(|id| state.spools.get(&id)).cloned();
    match mmu.filter(|mmu| mmu.enabled && !mmu.gates.is_empty()) {
        Some(mmu) => mmu
            .gates
            .iter()
            .map(|gate| Lane {
                gate: Some(gate.index),
                tool: gate.tool,
                spool_id: gate.spool_id,
                spool: spool(gate.spool_id),
                active: usize::try_from(mmu.gate) == Ok(gate.index),
            })
            .collect(),
        None => vec![Lane {
            gate: None,
            tool: Some(0),
            spool_id: state.active_spool_id,
            spool: spool(state.active_spool_id),
            active: true,
        }],
    }
}

/// The lane in use, if it has a spool.
pub fn active_spool(lanes: &[Lane]) -> Option<&Spool> {
    lanes.iter().find(|lane| lane.active).and_then(|lane| lane.spool.as_ref())
}

/// The spool for a hint about the temperature: the target that does not match it, if it is off by more than the
/// tolerance. Without target (off) or without a recommendation there is nothing to say.
pub fn temperature_hint(target: f64, recommended: Option<f64>) -> Option<f64> {
    let recommended = recommended?;
    (target > 0.0 && (target - recommended).abs() > TEMPERATURE_TOLERANCE).then_some(recommended)
}

/// What differs between a print and the spools that would print it. Only hints: nothing here forbids anything.
#[derive(Debug, Clone, PartialEq)]
pub enum PrintHint {
    Material {
        tool: usize,
        expected: String,
        loaded: String,
    },
    /// Planned grams against what the spool has left.
    Weight {
        tool: usize,
        needed: f64,
        left: f64,
    },
    Color {
        tool: usize,
    },
}

/// Compares what the file asks for per tool with the spool in the lane of that tool. A fact that is missing on one
/// side (no weights in the file, no weight in Spoolman, no spool in the gate) is not compared.
pub fn print_hints(metadata: &FileMetadata, lanes: &[Lane], tool_to_gate: &[usize]) -> Vec<PrintHint> {
    let tools: Vec<usize> = if metadata.referenced_tools.is_empty() {
        (0..metadata.filament_weights.len().max(1)).collect()
    } else {
        metadata.referenced_tools.clone()
    };
    let materials: Vec<&str> =
        metadata.filament_type.as_deref().map(|types| types.split(';').collect()).unwrap_or_default();
    let mut hints = Vec::new();
    for &tool in &tools {
        let Some(spool) = lane_of_tool(lanes, tool_to_gate, tool).and_then(|lane| lane.spool.as_ref()) else {
            continue;
        };
        let expected = materials.get(tool).or(materials.first().filter(|_| materials.len() == 1)).copied();
        if let Some(expected) = expected.filter(|expected| !same_material(expected, &spool.material)) {
            hints.push(PrintHint::Material {
                tool,
                expected: expected.trim().to_owned(),
                loaded: spool.material.clone(),
            });
        }
        let needed = metadata
            .filament_weights
            .get(tool)
            .copied()
            .or_else(|| (tools.len() == 1).then_some(metadata.filament_weight_total).flatten());
        if let (Some(needed), Some(left)) = (needed, spool.remaining_weight) {
            if needed > left {
                hints.push(PrintHint::Weight { tool, needed, left });
            }
        }
        let wanted = metadata.filament_colors.get(tool).and_then(|color| parse_color(color));
        if let (Some(wanted), Some(loaded)) = (wanted, spool.color) {
            if color_distance(wanted, loaded) > COLOR_DISTANCE {
                hints.push(PrintHint::Color { tool });
            }
        }
    }
    hints
}

fn lane_of_tool<'a>(lanes: &'a [Lane], tool_to_gate: &[usize], tool: usize) -> Option<&'a Lane> {
    if lanes.iter().any(|lane| lane.gate.is_some()) {
        let gate = tool_to_gate.get(tool).copied().unwrap_or(tool);
        lanes.iter().find(|lane| lane.gate == Some(gate))
    } else {
        lanes.first().filter(|_| tool == 0)
    }
}

/// `ABS+` is `ABS`, case does not matter.
fn same_material(left: &str, right: &str) -> bool {
    let normalized = |text: &str| text.trim().trim_end_matches('+').to_ascii_uppercase();
    normalized(left) == normalized(right)
}

fn color_distance(left: [u8; 3], right: [u8; 3]) -> f64 {
    left.iter().zip(right).map(|(a, b)| (f64::from(*a) - f64::from(b)).powi(2)).sum::<f64>().sqrt()
}

/// The spools for the picker: those that sit in a lane first, then the most recently used.
pub fn picker_order(mut spools: Vec<Spool>, inserted: &[u32]) -> Vec<Spool> {
    spools.sort_by(|left, right| {
        inserted
            .contains(&right.id)
            .cmp(&inserted.contains(&left.id))
            .then_with(|| right.last_used.cmp(&left.last_used))
            .then_with(|| right.id.cmp(&left.id))
    });
    spools
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::printer::{MmuGate, MmuStatus};
    use serde_json::json;

    /// Shape of `GET /v1/spool/18` of Spoolman 0.2x as the printer of the author returned it (values rounded).
    fn spool_18() -> Value {
        json!({
            "id": 18, "registered": "2026-02-02T22:10:00Z", "first_used": "2026-02-02T22:17:26Z",
            "last_used": "2026-10-26T12:12:18Z",
            "filament": {
                "id": 7, "name": "Red", "vendor": { "id": 3, "name": "Geeetech" }, "material": "ABS+",
                "density": 1.04, "diameter": 1.75, "weight": 1000.0, "spool_weight": 192.0,
                "settings_extruder_temp": 240, "settings_bed_temp": 120, "color_hex": "E31F2B"
            },
            "remaining_weight": 613.11, "used_weight": 386.89, "remaining_length": 245_099.2,
            "initial_weight": 1000.0, "spool_weight": 180.0, "location": "voron @ MMU Gate:0", "archived": false,
            "extra": { "printer_name": "\"voron\"", "mmu_gate_map": "\"{}\"" }
        })
    }

    fn spool(id: u32, material: &str, left: f64, color: [u8; 3]) -> Spool {
        Spool {
            id,
            material: material.into(),
            remaining_weight: Some(left),
            initial_weight: Some(1000.0),
            color: Some(color),
            ..Spool::default()
        }
    }

    fn state_with(spools: &[Spool], active: Option<u32>) -> SpoolmanState {
        SpoolmanState {
            connected: true,
            active_spool_id: active,
            spools: spools.iter().map(|spool| (spool.id, spool.clone())).collect(),
            ..SpoolmanState::default()
        }
    }

    fn mmu_with(spool_ids: &[Option<u32>], selected: i32, tool_to_gate: &[usize]) -> MmuStatus {
        MmuStatus {
            enabled: true,
            num_gates: spool_ids.len(),
            gate: selected,
            tool_to_gate: tool_to_gate.to_vec(),
            gates: spool_ids
                .iter()
                .enumerate()
                .map(|(index, spool_id)| MmuGate {
                    index,
                    spool_id: *spool_id,
                    tool: Some(index),
                    ..MmuGate::default()
                })
                .collect(),
            ..MmuStatus::default()
        }
    }

    #[test]
    fn reads_a_spool_as_spoolman_returns_it() {
        let spool = Spool::from_json(&spool_18()).unwrap();

        assert_eq!((spool.id, spool.title().as_str(), spool.material.as_str()), (18, "Geeetech Red", "ABS+"));
        assert_eq!(spool.color, Some([0xE3, 0x1F, 0x2B]));
        assert_eq!((spool.extruder_temp, spool.bed_temp), (Some(240.0), Some(120.0)));
        assert_eq!((spool.density, spool.diameter, spool.spool_weight), (Some(1.04), Some(1.75), Some(180.0)));
        assert_eq!(spool.location, "voron @ MMU Gate:0");
        assert!((spool.remaining_share().unwrap() - 0.6131).abs() < 1e-3);
        assert!(!spool.is_low());
    }

    #[test]
    fn a_spool_without_start_weight_takes_that_of_its_filament() {
        let mut value = spool_18();
        value["initial_weight"] = Value::Null;

        assert_eq!(Spool::from_json(&value).unwrap().initial_weight, Some(1000.0));
    }

    #[test]
    fn a_spool_under_15_percent_is_low_and_without_start_weight_it_is_not() {
        assert!(spool(1, "PLA", 120.0, [0; 3]).is_low());
        assert!(!spool(1, "PLA", 150.0, [0; 3]).is_low());
        assert!(!Spool { remaining_weight: Some(1.0), ..Spool::default() }.is_low());
    }

    #[test]
    fn entries_without_an_id_are_skipped() {
        assert_eq!(parse_spool_list(&json!([{ "filament": {} }, spool_18()])).len(), 1);
        assert!(parse_spool_list(&json!({ "not": "a list" })).is_empty());
    }

    #[test]
    fn colours_come_with_or_without_hash() {
        assert_eq!(parse_color("#26A69A"), Some([0x26, 0xA6, 0x9A]));
        assert_eq!(parse_color("26a69a"), Some([0x26, 0xA6, 0x9A]));
        assert_eq!(parse_color("12"), None);
        assert_eq!(parse_color("zzzzzz"), None);
    }

    #[test]
    fn the_status_names_connection_pending_reports_and_active_spool() {
        let status = parse_status(&json!({ "spoolman_connected": true, "pending_reports": [{}, {}], "spool_id": 18 }));
        assert_eq!(status, (true, 2, Some(18)));
        assert_eq!(
            parse_status(&json!({ "spoolman_connected": false, "pending_reports": [], "spool_id": null })),
            (false, 0, None)
        );
    }

    #[test]
    fn the_proxy_answers_in_version_2_or_raw() {
        assert_eq!(proxy_response(&json!({ "response": { "id": 1 }, "error": null })), Ok(json!({ "id": 1 })));
        assert_eq!(
            proxy_response(
                &json!({ "response": null, "error": { "status_code": 404, "message": "No spool with ID 5 found" } })
            ),
            Err("No spool with ID 5 found".to_owned())
        );
        assert_eq!(proxy_response(&json!({ "id": 1 })), Ok(json!({ "id": 1 })));
    }

    #[test]
    fn a_printer_without_mmu_has_one_lane_with_the_active_spool() {
        let state = state_with(&[spool(5, "PLA", 800.0, [0; 3])], Some(5));

        let lanes = lanes(None, &state);

        assert_eq!(lanes.len(), 1);
        assert_eq!((lanes[0].gate, lanes[0].spool_id, lanes[0].active), (None, Some(5), true));
        assert_eq!(active_spool(&lanes).map(|spool| spool.id), Some(5));
    }

    #[test]
    fn with_an_mmu_every_gate_is_a_lane_and_the_selected_one_is_active() {
        let state = state_with(&[spool(5, "PLA", 800.0, [0; 3]), spool(6, "PETG", 300.0, [0; 3])], Some(6));
        let mmu = mmu_with(&[Some(5), Some(6), None], 1, &[0, 1, 2]);

        let lanes = lanes(Some(&mmu), &state);

        assert_eq!(lanes.iter().map(|lane| lane.spool_id).collect::<Vec<_>>(), [Some(5), Some(6), None]);
        assert_eq!(lanes.iter().map(|lane| lane.active).collect::<Vec<_>>(), [false, true, false]);
        assert_eq!(active_spool(&lanes).map(|spool| spool.id), Some(6));
    }

    #[test]
    fn a_spool_that_spoolman_has_not_answered_yet_is_a_lane_without_spool() {
        let lanes = lanes(Some(&mmu_with(&[Some(9)], 0, &[0])), &SpoolmanState::default());

        assert_eq!((lanes[0].spool_id, lanes[0].spool.is_none()), (Some(9), true));
    }

    fn file(weights: &[f64], types: &str, colors: &[&str]) -> FileMetadata {
        FileMetadata {
            filament_weights: weights.to_vec(),
            filament_weight_total: Some(weights.iter().sum()),
            filament_type: Some(types.to_owned()),
            filament_colors: colors.iter().map(|color| (*color).to_owned()).collect(),
            ..FileMetadata::default()
        }
    }

    #[test]
    fn a_matching_spool_gives_no_hint() {
        let lanes = lanes(None, &state_with(&[spool(5, "PLA", 800.0, [0x26, 0xA6, 0x9A])], Some(5)));

        assert!(print_hints(&file(&[64.0], "PLA", &["#26A69A"]), &lanes, &[]).is_empty());
    }

    #[test]
    fn material_weight_and_colour_that_do_not_fit_are_named() {
        let lanes = lanes(None, &state_with(&[spool(5, "PETG", 40.0, [0xE3, 0x1F, 0x2B])], Some(5)));

        let hints = print_hints(&file(&[62.0], "PLA", &["#26A69A"]), &lanes, &[]);

        assert_eq!(
            hints,
            [
                PrintHint::Material { tool: 0, expected: "PLA".into(), loaded: "PETG".into() },
                PrintHint::Weight { tool: 0, needed: 62.0, left: 40.0 },
                PrintHint::Color { tool: 0 },
            ]
        );
    }

    #[test]
    fn abs_plus_counts_as_abs_and_close_colours_as_equal() {
        let lanes = lanes(None, &state_with(&[spool(5, "ABS+", 500.0, [0xE3, 0x1F, 0x2B])], Some(5)));

        assert!(print_hints(&file(&[10.0], "abs", &["#FF2020"]), &lanes, &[]).is_empty());
    }

    #[test]
    fn a_multicolour_print_is_compared_tool_by_tool_through_the_tool_to_gate_map() {
        let state = state_with(&[spool(5, "PLA", 500.0, [0; 3]), spool(6, "PETG", 20.0, [0; 3])], None);
        let mmu = mmu_with(&[Some(5), Some(6)], 0, &[1, 0]);
        let mut metadata = file(&[30.0, 50.0], "PLA;PETG", &[]);
        metadata.referenced_tools = vec![0, 1];

        let hints = print_hints(&metadata, &lanes(Some(&mmu), &state), &mmu.tool_to_gate);

        // Tool 0 prints from gate 1 (PETG, 20 g left), tool 1 from gate 0 (PLA).
        assert_eq!(
            hints,
            [
                PrintHint::Material { tool: 0, expected: "PLA".into(), loaded: "PETG".into() },
                PrintHint::Weight { tool: 0, needed: 30.0, left: 20.0 },
                PrintHint::Material { tool: 1, expected: "PETG".into(), loaded: "PLA".into() },
            ]
        );
    }

    #[test]
    fn nothing_is_compared_without_the_facts() {
        // No weights, no colours, no spool in the gate: no hint, and a gate without a spool says nothing.
        let mmu = mmu_with(&[None], 0, &[0]);
        let metadata = FileMetadata { filament_type: Some("PLA".into()), ..FileMetadata::default() };

        assert!(print_hints(&metadata, &lanes(Some(&mmu), &SpoolmanState::default()), &[0]).is_empty());
        let bare = lanes(None, &state_with(&[Spool { id: 5, ..Spool::default() }], Some(5)));
        assert!(print_hints(&FileMetadata::default(), &bare, &[]).is_empty());
    }

    #[test]
    fn the_target_is_only_flagged_when_it_is_off_by_more_than_the_tolerance() {
        assert_eq!(temperature_hint(210.0, Some(240.0)), Some(240.0));
        assert_eq!(temperature_hint(238.0, Some(240.0)), None);
        assert_eq!(temperature_hint(0.0, Some(240.0)), None);
        assert_eq!(temperature_hint(210.0, None), None);
    }

    #[test]
    fn the_weight_of_a_length_follows_diameter_and_density() {
        let spool = Spool::from_json(&spool_18()).unwrap();

        // 1 m of 1.75 mm ABS+ (1.04 g/cm³) weighs about 2.5 g.
        assert!((spool.weight_of_length(1000.0).unwrap() - 2.5).abs() < 0.01);
        assert_eq!(Spool { diameter: None, ..spool }.weight_of_length(1000.0), None);
    }

    #[test]
    fn the_picker_lists_inserted_spools_first_then_the_most_recently_used() {
        let at = |id: u32, last: &str| Spool { id, last_used: last.into(), ..Spool::default() };
        let spools = vec![at(1, "2026-10-01"), at(2, "2026-10-20"), at(3, ""), at(4, "2026-10-10")];

        let order: Vec<u32> = picker_order(spools, &[3]).iter().map(|spool| spool.id).collect();

        assert_eq!(order, [3, 2, 4, 1]);
    }
}
