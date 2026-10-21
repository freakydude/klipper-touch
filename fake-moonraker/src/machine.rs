//! Welche optionalen Module der simulierte Drucker besitzt.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MachineProfile {
    pub quad_gantry_level: bool,
    pub chamber_heater: bool,
    pub mmu_gates: Option<usize>,
}

impl MachineProfile {
    pub const MAX_MMU_GATES: usize = 16;

    pub fn basic() -> Self {
        Self { quad_gantry_level: false, chamber_heater: false, mmu_gates: None }
    }

    pub fn voron() -> Self {
        Self { quad_gantry_level: true, chamber_heater: true, mmu_gates: None }
    }

    pub fn happy_hare(gates: usize) -> Self {
        Self { quad_gantry_level: true, chamber_heater: true, mmu_gates: Some(gates) }
    }
}

impl Default for MachineProfile {
    fn default() -> Self {
        Self::voron()
    }
}
