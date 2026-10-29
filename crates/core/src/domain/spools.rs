//! Spools in the printer: the lanes they sit in and what differs between a print and its spools. A lane is one place
//! for a spool: a gate of Happy Hare, or the one spool of a printer without MMU (the active spool of Moonraker).

use crate::sources::moonraker_server::files::FileMetadata;
use crate::sources::spoolman::{parse_color, Spool, SpoolmanState};
use crate::sources::PrinterStatus;

/// A target that is further off than this from the temperature of the spool gets a hint, in °C.
const TEMPERATURE_TOLERANCE: f64 = 5.0;
/// Colours further apart than this (distance in RGB, at most 441) count as different.
const COLOR_DISTANCE: f64 = 120.0;

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

/// The lanes of the printer: a gate each while Happy Hare has the say, otherwise the one active spool.
pub fn lanes(status: &PrinterStatus, state: &SpoolmanState) -> Vec<Lane> {
    let spool = |id: Option<u32>| id.and_then(|id| state.spools.get(&id)).cloned();
    match status.happy_hare().filter(|mmu| !mmu.gates.is_empty()) {
        Some(mmu) => mmu
            .gates
            .iter()
            .map(|gate| Lane {
                gate: Some(gate.index),
                tool: gate.tools.first().copied(),
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

/// Compares what the file asks for per tool with the spool in the lane of that tool (through Happy Hare's tool to
/// gate map). A fact that is missing on one side (no weights in the file, no weight in Spoolman, no spool in the
/// gate) is not compared.
pub fn print_hints(metadata: &FileMetadata, status: &PrinterStatus, lanes: &[Lane]) -> Vec<PrintHint> {
    let tool_to_gate = status.happy_hare().map_or(&[][..], |mmu| mmu.tool_to_gate.as_slice());
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
        // A multi-colour spool fits when one of its colours is close.
        let nearest = spool
            .colors
            .iter()
            .map(|loaded| wanted.map(|wanted| color_distance(wanted, *loaded)))
            .min_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
        if nearest.flatten().is_some_and(|distance| distance > COLOR_DISTANCE) {
            hints.push(PrintHint::Color { tool });
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
    use crate::sources::happy_hare::{MmuGate, MmuStatus};

    fn spool(id: u32, material: &str, left: f64, color: [u8; 3]) -> Spool {
        Spool {
            id,
            material: material.into(),
            remaining_weight: Some(left),
            initial_weight: Some(1000.0),
            colors: vec![color],
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

    /// A printer with Happy Hare in charge, one gate per entry of `spool_ids`.
    fn mmu_with(spool_ids: &[Option<u32>], selected: i32, tool_to_gate: &[usize]) -> PrinterStatus {
        let mmu = MmuStatus {
            version: "4.1.0".into(),
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
                    tools: vec![index],
                    ..MmuGate::default()
                })
                .collect(),
            ..MmuStatus::default()
        };
        PrinterStatus { mmu: Some(mmu), ..PrinterStatus::default() }
    }

    fn without_mmu() -> PrinterStatus {
        PrinterStatus::default()
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
    fn a_printer_without_mmu_has_one_lane_with_the_active_spool() {
        let state = state_with(&[spool(5, "PLA", 800.0, [0; 3])], Some(5));

        let lanes = lanes(&without_mmu(), &state);

        assert_eq!(lanes.len(), 1);
        assert_eq!((lanes[0].gate, lanes[0].spool_id, lanes[0].active), (None, Some(5), true));
        assert_eq!(active_spool(&lanes).map(|spool| spool.id), Some(5));
    }

    #[test]
    fn with_an_mmu_every_gate_is_a_lane_and_the_selected_one_is_active() {
        let state = state_with(&[spool(5, "PLA", 800.0, [0; 3]), spool(6, "PETG", 300.0, [0; 3])], Some(6));
        let status = mmu_with(&[Some(5), Some(6), None], 1, &[0, 1, 2]);

        let lanes = lanes(&status, &state);

        assert_eq!(lanes.iter().map(|lane| lane.spool_id).collect::<Vec<_>>(), [Some(5), Some(6), None]);
        assert_eq!(lanes.iter().map(|lane| lane.active).collect::<Vec<_>>(), [false, true, false]);
        assert_eq!(active_spool(&lanes).map(|spool| spool.id), Some(6));
    }

    #[test]
    fn a_disabled_mmu_leaves_the_one_active_spool() {
        let state = state_with(&[spool(5, "PLA", 800.0, [0; 3])], Some(5));
        let mut status = mmu_with(&[Some(9), None], 0, &[0, 1]);
        status.mmu.as_mut().unwrap().enabled = false;

        let lanes = lanes(&status, &state);

        assert_eq!(lanes.len(), 1);
        assert_eq!((lanes[0].gate, lanes[0].spool_id), (None, Some(5)));
    }

    #[test]
    fn a_spool_that_spoolman_has_not_answered_yet_is_a_lane_without_spool() {
        let lanes = lanes(&mmu_with(&[Some(9)], 0, &[0]), &SpoolmanState::default());

        assert_eq!((lanes[0].spool_id, lanes[0].spool.is_none()), (Some(9), true));
    }

    #[test]
    fn a_multi_colour_spool_fits_a_file_when_one_of_its_colours_is_close() {
        let mut spool = spool(5, "PLA", 800.0, [0xFF, 0x00, 0x04]);
        spool.colors.push([0x26, 0xA6, 0x9A]);
        let status = without_mmu();
        let lanes = lanes(&status, &state_with(&[spool], Some(5)));

        assert!(print_hints(&file(&[10.0], "PLA", &["#26A69A"]), &status, &lanes).is_empty());
        assert_eq!(print_hints(&file(&[10.0], "PLA", &["#0000FF"]), &status, &lanes), [PrintHint::Color { tool: 0 }]);
    }

    #[test]
    fn a_matching_spool_gives_no_hint() {
        let status = without_mmu();
        let lanes = lanes(&status, &state_with(&[spool(5, "PLA", 800.0, [0x26, 0xA6, 0x9A])], Some(5)));

        assert!(print_hints(&file(&[64.0], "PLA", &["#26A69A"]), &status, &lanes).is_empty());
    }

    #[test]
    fn material_weight_and_colour_that_do_not_fit_are_named() {
        let status = without_mmu();
        let lanes = lanes(&status, &state_with(&[spool(5, "PETG", 40.0, [0xE3, 0x1F, 0x2B])], Some(5)));

        let hints = print_hints(&file(&[62.0], "PLA", &["#26A69A"]), &status, &lanes);

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
        let status = without_mmu();
        let lanes = lanes(&status, &state_with(&[spool(5, "ABS+", 500.0, [0xE3, 0x1F, 0x2B])], Some(5)));

        assert!(print_hints(&file(&[10.0], "abs", &["#FF2020"]), &status, &lanes).is_empty());
    }

    #[test]
    fn a_multicolour_print_is_compared_tool_by_tool_through_the_tool_to_gate_map() {
        let state = state_with(&[spool(5, "PLA", 500.0, [0; 3]), spool(6, "PETG", 20.0, [0; 3])], None);
        let status = mmu_with(&[Some(5), Some(6)], 0, &[1, 0]);
        let mut metadata = file(&[30.0, 50.0], "PLA;PETG", &[]);
        metadata.referenced_tools = vec![0, 1];

        let hints = print_hints(&metadata, &status, &lanes(&status, &state));

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
        let status = mmu_with(&[None], 0, &[0]);
        let metadata = FileMetadata { filament_type: Some("PLA".into()), ..FileMetadata::default() };

        assert!(print_hints(&metadata, &status, &lanes(&status, &SpoolmanState::default())).is_empty());
        let bare_status = without_mmu();
        let bare = lanes(&bare_status, &state_with(&[Spool { id: 5, ..Spool::default() }], Some(5)));
        assert!(print_hints(&FileMetadata::default(), &bare_status, &bare).is_empty());
    }

    #[test]
    fn the_target_is_only_flagged_when_it_is_off_by_more_than_the_tolerance() {
        assert_eq!(temperature_hint(210.0, Some(240.0)), Some(240.0));
        assert_eq!(temperature_hint(238.0, Some(240.0)), None);
        assert_eq!(temperature_hint(0.0, Some(240.0)), None);
        assert_eq!(temperature_hint(210.0, None), None);
    }

    #[test]
    fn the_picker_lists_inserted_spools_first_then_the_most_recently_used() {
        let at = |id: u32, last: &str| Spool { id, last_used: last.into(), ..Spool::default() };
        let spools = vec![at(1, "2026-10-01"), at(2, "2026-10-20"), at(3, ""), at(4, "2026-10-10")];

        let order: Vec<u32> = picker_order(spools, &[3]).iter().map(|spool| spool.id).collect();

        assert_eq!(order, [3, 2, 4, 1]);
    }
}
