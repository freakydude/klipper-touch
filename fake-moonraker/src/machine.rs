//! Which optional modules the simulated printer has.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MachineProfile {
    pub quad_gantry_level: bool,
    pub chamber_heater: bool,
    pub mmu_gates: Option<usize>,
    /// Klipper `exclude_object` module with the objects of the print.
    pub exclude_object: bool,
    /// Moonraker's `spoolman` component with a few spools.
    pub spoolman: bool,
    /// Extra fans, LEDs with the led_effect plugin, more temperature sensors and firmware retraction.
    pub devices: bool,
    /// Number of filament sensors, taken from the front of the list of the profile (`FILAMENT_SENSORS` or `MMU_FILAMENT_SENSORS`).
    pub filament_sensors: usize,
}

/// Klipper object names of the simulated filament sensors; the second reports a runout.
pub const FILAMENT_SENSORS: [&str; 2] =
    ["filament_switch_sensor toolhead_runout", "filament_motion_sensor extruder_encoder"];

/// Happy Hare sensors as Klipper objects, as listed by a real single-lane type-B machine with a sync-feedback buffer
/// (Happy Hare 4.1.0, `objects.list`, 2026-10-25). Names come from `mmu_constants.py`; the gate suffix is kept.
/// The machine has no shared exit sensor and no encoder.
pub const MMU_FILAMENT_SENSORS: [&str; 6] = [
    "filament_switch_sensor mmu_entry_0",
    "filament_switch_sensor mmu_exit_0",
    "filament_switch_sensor toolhead",
    "filament_switch_sensor extruder",
    "filament_switch_sensor filament_compression",
    "filament_switch_sensor filament_tension",
];

impl MachineProfile {
    pub const MAX_MMU_GATES: usize = 16;

    pub fn basic() -> Self {
        Self {
            quad_gantry_level: false,
            chamber_heater: false,
            mmu_gates: None,
            exclude_object: false,
            spoolman: false,
            devices: false,
            filament_sensors: 0,
        }
    }

    pub fn voron() -> Self {
        Self {
            quad_gantry_level: true,
            chamber_heater: true,
            mmu_gates: None,
            exclude_object: true,
            spoolman: true,
            devices: true,
            filament_sensors: 2,
        }
    }

    pub fn happy_hare(gates: usize) -> Self {
        Self {
            quad_gantry_level: true,
            chamber_heater: true,
            mmu_gates: Some(gates),
            exclude_object: true,
            spoolman: true,
            devices: true,
            filament_sensors: 5,
        }
    }
}

impl Default for MachineProfile {
    fn default() -> Self {
        Self::voron()
    }
}
