//! The whole printer state as the subscription reports it: Klipper's own objects and those of its plugins.

use super::happy_hare::{self, MmuStatus};
use super::klipper::{
    Activity, ExtruderState, Fan, FilamentSensor, Heater, KlipperParts, KlippyState, Led, Motion, PrintJob,
    PrintObject, Retraction, TemperatureSensor,
};
use super::led_effect::{self, LedEffect};
use super::objects::ObjectStore;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrinterStatus {
    pub klippy_state: KlippyState,
    pub klippy_message: String,
    pub activity: Activity,
    pub job: PrintJob,
    pub motion: Motion,
    pub extruder: ExtruderState,
    /// Extruder first, then bed, then additional heaters in object order.
    pub heaters: Vec<Heater>,
    pub sensors: Vec<TemperatureSensor>,
    /// Part fan first, then the other fans; see [`Fan`].
    pub fans: Vec<Fan>,
    pub leds: Vec<Led>,
    /// Effects of the optional led_effect plugin.
    pub led_effects: Vec<LedEffect>,
    /// `None` without `[firmware_retraction]`.
    pub retraction: Option<Retraction>,
    pub save_config_pending: bool,
    /// `Some(applied)` if quad gantry level or Z tilt is present.
    pub leveling_applied: Option<bool>,
    pub filament_sensors: Vec<FilamentSensor>,
    /// Objects of the print with their exclude state; empty without the `exclude_object` module.
    pub print_objects: Vec<PrintObject>,
    /// Happy Hare; `None` without the `mmu` object.
    pub mmu: Option<MmuStatus>,
}

impl PrinterStatus {
    pub fn from_store(store: &ObjectStore) -> Self {
        let klipper = KlipperParts::read(store);
        Self {
            klippy_state: klipper.klippy_state,
            klippy_message: klipper.klippy_message,
            activity: klipper.activity,
            job: klipper.job,
            motion: klipper.motion,
            extruder: klipper.extruder,
            heaters: klipper.heaters,
            sensors: klipper.sensors,
            fans: klipper.fans,
            leds: klipper.leds,
            led_effects: led_effect::read_effects(store),
            retraction: klipper.retraction,
            save_config_pending: klipper.save_config_pending,
            leveling_applied: klipper.leveling_applied,
            filament_sensors: klipper.filament_sensors,
            print_objects: klipper.print_objects,
            mmu: happy_hare::read_mmu(store),
        }
    }

    pub fn heater(&self, object: &str) -> Option<&Heater> {
        self.heaters.iter().find(|heater| heater.object == object)
    }

    /// Happy Hare while it has the say, see [`MmuStatus::is_in_charge`]; every rule that depends on Happy Hare asks
    /// here.
    pub fn happy_hare(&self) -> Option<&MmuStatus> {
        self.mmu.as_ref().filter(|mmu| mmu.is_in_charge())
    }
}
