//! What the print page tells about a pause or an error, and whether a pause can be planned at a layer.
//!
//! Only sources that state something are used; the reason of a pause is never guessed. A pause started on this
//! display, in Mainsail, by a macro or by the file all read the same. Macros are black boxes: they are called by
//! name with their documented parameters, and nothing is assumed about what they do inside.

use super::{Capabilities, PrintState, PrinterStatus};

/// Mainsail's macro that pauses at the next layer change (mainsail-config `client.cfg`, optional):
/// `SET_PAUSE_NEXT_LAYER [ENABLE=[0|1]] [MACRO=<name>]`.
pub const PAUSE_NEXT_LAYER_MACRO: &str = "SET_PAUSE_NEXT_LAYER";
/// Mainsail's macro that pauses when a layer is reached (optional):
/// `SET_PAUSE_AT_LAYER [ENABLE=[0|1]] [LAYER=<number>] [MACRO=<name>]`.
pub const PAUSE_AT_LAYER_MACRO: &str = "SET_PAUSE_AT_LAYER";

/// Something a source states while the print is paused or failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterruptionFact {
    /// `print_stats.message`, set by Klipper, for example the error that stopped the print.
    KlipperMessage(String),
    /// Happy Hare's `reason_for_pause`.
    MmuReason(String),
    /// An enabled filament sensor sees no filament. A fact, not necessarily the cause of the pause.
    SensorEmpty { label: String },
}

/// A paused or failed print and what is known about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interruption {
    /// The print stopped with an error; otherwise it is paused.
    pub failed: bool,
    pub facts: Vec<InterruptionFact>,
}

/// The pause or error of the print with every fact a source states; `None` while the print is not interrupted.
/// Happy Hare watches its own sensors and reports the reason itself, so the sensor facts only count without it.
pub fn interruption(status: &PrinterStatus) -> Option<Interruption> {
    let failed = match status.job.state {
        PrintState::Paused => false,
        PrintState::Error => true,
        _ => return None,
    };
    let mut facts = Vec::new();
    if let Some(message) = non_empty(&status.job.message) {
        facts.push(InterruptionFact::KlipperMessage(message));
    }
    match &status.mmu {
        Some(mmu) => facts.extend(non_empty(&mmu.reason_for_pause).map(InterruptionFact::MmuReason)),
        None => facts.extend(
            status
                .filament_sensors
                .iter()
                .filter(|sensor| sensor.enabled && !sensor.detected)
                .map(|sensor| InterruptionFact::SensorEmpty { label: sensor.label.clone() }),
        ),
    }
    Some(Interruption { failed, facts })
}

fn non_empty(text: &str) -> Option<String> {
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

/// Ways to plan a pause at a layer for the running print.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerPauseOffer {
    pub next_layer: bool,
    pub at_layer: bool,
    pub current_layer: u32,
    /// Known only if the slicer reports it (`SET_PRINT_STATS_INFO TOTAL_LAYER=`).
    pub total_layers: Option<u32>,
}

/// A pause at a layer can be planned while a job runs, the slicer reports the layers (the macros act on that
/// report) and at least one of Mainsail's macros exists.
pub fn layer_pause_offer(capabilities: &Capabilities, status: &PrinterStatus) -> Option<LayerPauseOffer> {
    if !status.job.state.is_active() {
        return None;
    }
    let current_layer = status.job.current_layer?;
    let next_layer = capabilities.has_macro(PAUSE_NEXT_LAYER_MACRO);
    let at_layer = capabilities.has_macro(PAUSE_AT_LAYER_MACRO);
    (next_layer || at_layer).then_some(LayerPauseOffer {
        next_layer,
        at_layer,
        current_layer,
        total_layers: status.job.total_layer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::printer::{FilamentSensor, MmuStatus, PrintJob};

    fn status(state: PrintState) -> PrinterStatus {
        PrinterStatus { job: PrintJob { state, ..PrintJob::default() }, ..PrinterStatus::default() }
    }

    fn sensor(label: &str, detected: bool) -> FilamentSensor {
        FilamentSensor {
            object: format!("filament_switch_sensor {label}"),
            label: label.into(),
            enabled: true,
            detected,
        }
    }

    #[test]
    fn a_pause_without_a_stated_reason_is_just_paused() {
        let interruption = interruption(&status(PrintState::Paused)).unwrap();

        assert!(!interruption.failed);
        assert!(interruption.facts.is_empty());
    }

    #[test]
    fn printing_and_finished_prints_are_not_interrupted() {
        for state in [PrintState::Standby, PrintState::Printing, PrintState::Complete, PrintState::Cancelled] {
            assert_eq!(interruption(&status(state)), None);
        }
    }

    #[test]
    fn an_error_carries_klippers_message() {
        let mut status = status(PrintState::Error);
        status.job.message = "Move out of range: 310.000 0.000 1.000 [12.000]\n".into();

        let interruption = interruption(&status).unwrap();

        assert!(interruption.failed);
        assert_eq!(
            interruption.facts,
            [InterruptionFact::KlipperMessage("Move out of range: 310.000 0.000 1.000 [12.000]".into())]
        );
    }

    #[test]
    fn empty_enabled_sensors_are_facts_without_mmu() {
        let mut status = status(PrintState::Paused);
        let mut disabled = sensor("Spare", false);
        disabled.enabled = false;
        status.filament_sensors = vec![sensor("Runout", false), sensor("Toolhead", true), disabled];

        let facts = interruption(&status).unwrap().facts;

        assert_eq!(facts, [InterruptionFact::SensorEmpty { label: "Runout".into() }]);
    }

    #[test]
    fn with_mmu_happy_hare_states_the_reason_and_the_sensors_are_left_out() {
        let mut status = status(PrintState::Paused);
        status.filament_sensors = vec![sensor("Toolhead", false)];
        status.mmu = Some(MmuStatus { reason_for_pause: "Runout at gate 1".into(), ..MmuStatus::default() });

        let facts = interruption(&status).unwrap().facts;

        assert_eq!(facts, [InterruptionFact::MmuReason("Runout at gate 1".into())]);
    }

    fn capabilities(macros: &[&str]) -> Capabilities {
        Capabilities { macros: macros.iter().map(|name| (*name).to_owned()).collect(), ..Capabilities::default() }
    }

    fn printing_at_layer(layer: Option<u32>) -> PrinterStatus {
        let mut status = status(PrintState::Printing);
        status.job.current_layer = layer;
        status.job.total_layer = Some(200);
        status
    }

    #[test]
    fn a_layer_pause_needs_a_macro_a_running_job_and_reported_layers() {
        let both = capabilities(&[PAUSE_NEXT_LAYER_MACRO, PAUSE_AT_LAYER_MACRO]);

        assert_eq!(
            layer_pause_offer(&both, &printing_at_layer(Some(12))),
            Some(LayerPauseOffer { next_layer: true, at_layer: true, current_layer: 12, total_layers: Some(200) })
        );
        assert_eq!(layer_pause_offer(&capabilities(&[]), &printing_at_layer(Some(12))), None);
        assert_eq!(layer_pause_offer(&both, &printing_at_layer(None)), None);
        assert_eq!(layer_pause_offer(&both, &status(PrintState::Complete)), None);
    }

    #[test]
    fn each_macro_is_offered_on_its_own() {
        let offer = layer_pause_offer(&capabilities(&[PAUSE_AT_LAYER_MACRO]), &printing_at_layer(Some(3))).unwrap();

        assert!(!offer.next_layer);
        assert!(offer.at_layer);
    }
}
