//! Filament on this printer: what Load and Unload run, and which filament sensors belong to the printer rather
//! than to Happy Hare.

use crate::sources::happy_hare::is_mmu_klipper_sensor;
use crate::sources::klipper::FilamentSensor;
use crate::sources::macro_conventions::FilamentMacros;
use crate::sources::{PrinterStatus, Sources};

/// Where Load and Unload come from, see [`handling`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilamentHandling {
    Mmu,
    Macros(FilamentMacros),
    None,
}

/// Happy Hare while it has the say, otherwise the macros `LOAD_FILAMENT` and `UNLOAD_FILAMENT` (as KlipperScreen
/// does), otherwise nothing. There is no generic fallback: the sequence depends on the length of the bowden tube and
/// the temperatures of the machine.
pub fn handling(sources: &Sources, status: &PrinterStatus) -> FilamentHandling {
    if status.happy_hare().is_some() {
        return FilamentHandling::Mmu;
    }
    sources.filament_macros.map_or(FilamentHandling::None, FilamentHandling::Macros)
}

/// The filament sensors of the printer itself. With Happy Hare in charge its gate and buffer sensors belong to its
/// own view; `toolhead` and `extruder` stay, they sit behind the MMU and matter without one.
pub fn printer_sensors(status: &PrinterStatus) -> impl Iterator<Item = &FilamentSensor> {
    let has_mmu = status.happy_hare().is_some();
    status.filament_sensors.iter().filter(move |sensor| !(has_mmu && is_mmu_klipper_sensor(&sensor.object)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::happy_hare::MmuStatus;

    fn happy_hare(enabled: bool, version: &str) -> MmuStatus {
        MmuStatus { enabled, version: version.into(), ..MmuStatus::default() }
    }

    fn status_with(mmu: Option<MmuStatus>) -> PrinterStatus {
        let sensor = |object: &str| FilamentSensor { object: object.into(), ..FilamentSensor::default() };
        PrinterStatus {
            mmu,
            filament_sensors: vec![
                sensor("filament_switch_sensor mmu_gate_0"),
                sensor("filament_switch_sensor toolhead"),
            ],
            ..PrinterStatus::default()
        }
    }

    #[test]
    fn load_and_unload_prefer_happy_hare_then_macros() {
        let macros =
            Sources { filament_macros: Some(FilamentMacros { load: true, unload: false }), ..Sources::default() };

        assert_eq!(handling(&macros, &status_with(Some(happy_hare(true, "4.1.0")))), FilamentHandling::Mmu);
        assert_eq!(
            handling(&macros, &status_with(None)),
            FilamentHandling::Macros(FilamentMacros { load: true, unload: false })
        );
        assert_eq!(handling(&Sources::default(), &status_with(None)), FilamentHandling::None);
    }

    #[test]
    fn a_disabled_or_too_old_happy_hare_leaves_loading_to_the_macros() {
        let macros =
            Sources { filament_macros: Some(FilamentMacros { load: true, unload: true }), ..Sources::default() };

        for mmu in [happy_hare(false, "4.1.0"), happy_hare(true, "4.0.2")] {
            assert!(matches!(handling(&macros, &status_with(Some(mmu))), FilamentHandling::Macros(_)));
        }
    }

    #[test]
    fn happy_hare_keeps_its_own_sensors() {
        let names = |status: &PrinterStatus| -> Vec<String> {
            printer_sensors(status).map(|sensor| sensor.object.clone()).collect()
        };

        assert_eq!(names(&status_with(Some(happy_hare(true, "4.1.0")))), ["filament_switch_sensor toolhead"]);
        assert_eq!(names(&status_with(Some(happy_hare(false, "4.1.0")))).len(), 2);
        assert_eq!(names(&status_with(None)).len(), 2);
    }
}
