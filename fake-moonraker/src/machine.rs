//! Which optional modules the simulated printer has.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MachineProfile {
    pub quad_gantry_level: bool,
    pub chamber_heater: bool,
    pub mmu_gates: Option<usize>,
    /// Number of filament sensors, taken from the front of `FILAMENT_SENSORS`.
    pub filament_sensors: usize,
}

/// Klipper object names of the simulated filament sensors; the second reports a runout.
pub const FILAMENT_SENSORS: [&str; 5] = [
    "filament_switch_sensor toolhead_runout",
    "filament_motion_sensor extruder_encoder",
    "filament_switch_sensor mmu_pre_gate_0",
    "filament_switch_sensor mmu_pre_gate_1",
    "filament_switch_sensor mmu_gear_extruder_entry",
];

impl MachineProfile {
    pub const MAX_MMU_GATES: usize = 16;

    pub fn basic() -> Self {
        Self { quad_gantry_level: false, chamber_heater: false, mmu_gates: None, filament_sensors: 0 }
    }

    pub fn voron() -> Self {
        Self { quad_gantry_level: true, chamber_heater: true, mmu_gates: None, filament_sensors: 2 }
    }

    pub fn happy_hare(gates: usize) -> Self {
        Self { quad_gantry_level: true, chamber_heater: true, mmu_gates: Some(gates), filament_sensors: 5 }
    }
}

impl Default for MachineProfile {
    fn default() -> Self {
        Self::voron()
    }
}
