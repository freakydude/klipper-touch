//! Klipper's calibration helpers, simplified: `manual_probe` (`PROBE_CALIBRATE`, `MANUAL_PROBE`, `TESTZ`), the paper
//! test of `bed_screws`, `SCREWS_TILT_CALCULATE` with fixed results, the bed mesh profiles and what `SAVE_CONFIG`
//! would write (`configfile.save_config_pending_items`). Behaviour follows `klippy/extras/manual_probe.py`,
//! `bed_screws.py` and `configfile.py` (read 2026-10-28); measured values are made up.

use crate::gcode::CommandError;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

/// Largest step of a bisecting `TESTZ`, as in `manual_probe.py`.
const BISECT_MAX: f64 = 0.2;
/// Height the nozzle waits at when a manual probe starts.
const START_HEIGHT: f64 = 2.0;
/// The configured `z_offset` of the simulated probe.
pub const PROBE_Z_OFFSET: f64 = 1.0;
/// The bed screws of a bed slinger (names and positions of a Sidewinder X2).
pub const SCREWS: [(&str, [f64; 2]); 4] = [
    ("back left", [27.0, 272.5]),
    ("front left", [27.0, 72.5]),
    ("front right", [227.0, 72.5]),
    ("back right", [227.0, 272.5]),
];
/// `SCREWS_TILT_CALCULATE` results per screw: sign and turns; the first screw is the base.
const SCREW_TILT: [(&str, &str); 4] = [("CW", "00:00"), ("CW", "00:18"), ("CCW", "01:05"), ("CW", "00:02")];
const DEFAULT_MESH: &str = "default";

#[derive(Debug, Clone, PartialEq)]
pub enum CalibrationCommand {
    ProbeCalibrate,
    ManualProbe,
    TestZ(TestZ),
    Accept,
    Adjusted,
    Abort,
    BedScrewsAdjust,
    ScrewsTiltCalculate,
    BedMeshCalibrate { profile: Option<String> },
    BedMeshLoad { profile: String },
    BedMeshClear,
}

/// `TESTZ Z=`: a relative move, `+`/`-` halfway to the nearest height tried, `++`/`--` all the way.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestZ {
    By(f64),
    Up { half: bool },
    Down { half: bool },
}

impl TestZ {
    pub fn parse(value: &str) -> Result<Self, CommandError> {
        match value {
            "+" => Ok(Self::Up { half: true }),
            "++" => Ok(Self::Up { half: false }),
            "-" => Ok(Self::Down { half: true }),
            "--" => Ok(Self::Down { half: false }),
            number => number
                .parse()
                .map(Self::By)
                .map_err(|_| CommandError::new(format!("Unable to parse '{number}' as a float"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbePurpose {
    /// `PROBE_CALIBRATE`: on `ACCEPT` the probe's `z_offset` becomes pending.
    ProbeOffset,
    /// `MANUAL_PROBE`: the height is only reported.
    Height,
}

#[derive(Debug, Clone, PartialEq)]
struct ManualProbe {
    purpose: ProbePurpose,
    z: f64,
    tried: Vec<f64>,
}

impl ManualProbe {
    fn new(purpose: ProbePurpose) -> Self {
        Self { purpose, z: START_HEIGHT, tried: Vec::new() }
    }

    /// Like `cmd_TESTZ`: remember the current height, then move.
    fn test(&mut self, step: TestZ) {
        let z = self.z;
        if !self.tried.contains(&z) {
            self.tried.push(z);
            self.tried.sort_by(f64::total_cmp);
        }
        self.z = match step {
            TestZ::By(distance) => z + distance,
            TestZ::Up { half } => {
                let above = self.above(z).unwrap_or(f64::MAX);
                (if half { (above + z) / 2.0 } else { above }).min(z + BISECT_MAX)
            }
            TestZ::Down { half } => {
                let below = self.below(z).unwrap_or(f64::MIN);
                (if half { (below + z) / 2.0 } else { below }).max(z - BISECT_MAX)
            }
        };
    }

    fn above(&self, z: f64) -> Option<f64> {
        self.tried.iter().copied().find(|tried| *tried > z)
    }

    fn below(&self, z: f64) -> Option<f64> {
        self.tried.iter().copied().rev().find(|tried| *tried < z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BedScrews {
    current: usize,
    accepted: i64,
}

/// What the simulated printer has and where its helpers stand.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Calibration {
    probe: bool,
    bed_mesh: bool,
    screws: bool,
    manual_probe: Option<ManualProbe>,
    bed_screws: Option<BedScrews>,
    screw_results: bool,
    active_mesh: Option<String>,
    meshes: Vec<String>,
    pending: BTreeMap<String, Map<String, Value>>,
}

impl Calibration {
    /// `screws` adds both `[bed_screws]` and `[screws_tilt_adjust]`; a bed mesh starts with the profile `default`.
    pub fn new(probe: bool, bed_mesh: bool, screws: bool) -> Self {
        let meshes = if bed_mesh { vec![DEFAULT_MESH.to_owned()] } else { Vec::new() };
        Self { probe, bed_mesh, screws, meshes, ..Self::default() }
    }

    /// `homed`: whether all axes are homed; Klipper refuses to probe or move to a screw otherwise.
    pub fn apply(&mut self, command: CalibrationCommand, homed: bool) -> Result<(), CommandError> {
        match command {
            CalibrationCommand::ProbeCalibrate if self.probe => self.start_probe(ProbePurpose::ProbeOffset, homed)?,
            CalibrationCommand::ManualProbe => self.start_probe(ProbePurpose::Height, homed)?,
            CalibrationCommand::TestZ(step) => self.probing()?.test(step),
            CalibrationCommand::Accept if self.manual_probe.is_some() => self.accept_probe(),
            CalibrationCommand::Accept | CalibrationCommand::Adjusted if self.bed_screws.is_some() => {
                self.accept_screw(command == CalibrationCommand::Adjusted)
            }
            CalibrationCommand::Abort if self.manual_probe.is_some() || self.bed_screws.is_some() => {
                self.manual_probe = None;
                self.bed_screws = None;
            }
            CalibrationCommand::BedScrewsAdjust if self.screws => {
                if self.is_waiting() {
                    return Err(CommandError::new("Already in bed_screws helper; use ABORT to exit"));
                }
                must_be_homed(homed)?;
                self.bed_screws = Some(BedScrews { current: 0, accepted: 0 });
            }
            CalibrationCommand::ScrewsTiltCalculate if self.screws && self.probe => {
                must_be_homed(homed)?;
                self.screw_results = true;
            }
            CalibrationCommand::BedMeshCalibrate { profile } if self.bed_mesh && self.probe => {
                must_be_homed(homed)?;
                let profile = profile.unwrap_or_else(|| DEFAULT_MESH.to_owned());
                if !self.meshes.contains(&profile) {
                    self.meshes.push(profile.clone());
                }
                self.pending.insert(format!("bed_mesh {profile}"), mesh_section());
                self.active_mesh = Some(profile);
            }
            CalibrationCommand::BedMeshLoad { profile } if self.bed_mesh => {
                if !self.meshes.contains(&profile) {
                    return Err(CommandError::new(format!("bed_mesh: Unknown profile [{profile}]")));
                }
                self.active_mesh = Some(profile);
            }
            CalibrationCommand::BedMeshClear if self.bed_mesh => self.active_mesh = None,
            other => return Err(CommandError::new(format!("Unknown command:\"{}\"", command_name(&other)))),
        }
        Ok(())
    }

    fn start_probe(&mut self, purpose: ProbePurpose, homed: bool) -> Result<(), CommandError> {
        if self.is_waiting() {
            return Err(CommandError::new("Already in a manual Z probe. Use ABORT to abort it."));
        }
        must_be_homed(homed)?;
        self.manual_probe = Some(ManualProbe::new(purpose));
        Ok(())
    }

    fn probing(&mut self) -> Result<&mut ManualProbe, CommandError> {
        self.manual_probe.as_mut().ok_or_else(|| CommandError::new("Unknown command:\"TESTZ\""))
    }

    fn accept_probe(&mut self) {
        let Some(probe) = self.manual_probe.take() else { return };
        if probe.purpose == ProbePurpose::ProbeOffset {
            // The probe triggered at the start height; the nozzle touched the paper at `z`.
            let z_offset = PROBE_Z_OFFSET + START_HEIGHT - probe.z;
            self.pending
                .entry("probe".to_owned())
                .or_default()
                .insert("z_offset".to_owned(), json!(format!("{z_offset:.3}")));
        }
    }

    /// Like `bed_screws.py`: all screws in a row without adjustment end the helper; `ADJUSTED` starts the count over.
    fn accept_screw(&mut self, adjusted: bool) {
        let Some(mut screws) = self.bed_screws.take() else { return };
        screws.accepted = if adjusted { 0 } else { screws.accepted + 1 };
        if screws.accepted < SCREWS.len() as i64 {
            screws.current = (screws.current + 1) % SCREWS.len();
            self.bed_screws = Some(screws);
        }
    }

    fn is_waiting(&self) -> bool {
        self.manual_probe.is_some() || self.bed_screws.is_some()
    }

    /// `SAVE_CONFIG` writes everything pending.
    pub fn save(&mut self) {
        self.pending.clear();
    }

    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn pending_json(&self) -> Value {
        json!(self.pending)
    }

    pub fn status(&self) -> Map<String, Value> {
        let mut objects = Map::new();
        let probe = self.manual_probe.as_ref();
        let tried = |pick: fn(&ManualProbe, f64) -> Option<f64>| probe.and_then(|probe| pick(probe, probe.z));
        objects.insert(
            "manual_probe".to_owned(),
            json!({
                "is_active": probe.is_some(),
                "z_position": probe.map(|probe| round(probe.z)),
                "z_position_lower": tried(ManualProbe::below),
                "z_position_upper": tried(ManualProbe::above),
            }),
        );
        if self.probe {
            objects.insert(
                "probe".to_owned(),
                json!({ "name": "probe", "last_query": false, "last_probe_position": [0.0, 0.0, 0.0, 0.0] }),
            );
        }
        if self.bed_mesh {
            let profiles: Map<String, Value> = self
                .meshes
                .iter()
                .map(|name| (name.clone(), json!({ "points": [[0.0]], "mesh_params": {} })))
                .collect();
            objects.insert(
                "bed_mesh".to_owned(),
                json!({ "profile_name": self.active_mesh.clone().unwrap_or_default(), "profiles": profiles }),
            );
        }
        if self.screws {
            let screws = self.bed_screws;
            objects.insert(
                "bed_screws".to_owned(),
                json!({
                    "is_active": screws.is_some(),
                    "state": screws.map(|_| "adjust"),
                    "current_screw": screws.map_or(0, |screws| screws.current),
                    "accepted_screws": screws.map_or(0, |screws| screws.accepted),
                }),
            );
            objects.insert("screws_tilt_adjust".to_owned(), self.screw_tilt_json());
        }
        objects
    }

    fn screw_tilt_json(&self) -> Value {
        let results: Map<String, Value> = if self.screw_results {
            SCREW_TILT
                .iter()
                .enumerate()
                .map(|(index, (sign, adjust))| {
                    let z = 2.5 - index as f64 * 0.04;
                    (
                        format!("screw{}", index + 1),
                        json!({ "z": z, "sign": sign, "adjust": adjust, "is_base": index == 0 }),
                    )
                })
                .collect()
        } else {
            Map::new()
        };
        json!({ "error": false, "max_deviation": null, "results": results })
    }

    pub fn config_sections(&self) -> Map<String, Value> {
        let mut sections = Map::new();
        if self.probe {
            sections.insert(
                "probe".to_owned(),
                json!({ "x_offset": 23.0, "y_offset": -22.5, "z_offset": PROBE_Z_OFFSET, "speed": 5.0 }),
            );
        }
        if self.bed_mesh {
            sections.insert(
                "bed_mesh".to_owned(),
                json!({ "probe_count": [5, 5], "mesh_min": [30.0, 30.0], "mesh_max": [270.0, 270.0] }),
            );
        }
        if self.screws {
            let mut screws = Map::new();
            for (index, (name, position)) in SCREWS.iter().enumerate() {
                screws.insert(format!("screw{}", index + 1), json!(position));
                screws.insert(format!("screw{}_name", index + 1), json!(name));
            }
            sections.insert("bed_screws".to_owned(), Value::Object(screws.clone()));
            screws.insert("screw_thread".to_owned(), json!("CW-M3"));
            sections.insert("screws_tilt_adjust".to_owned(), Value::Object(screws));
        }
        sections
    }
}

fn must_be_homed(homed: bool) -> Result<(), CommandError> {
    if homed {
        Ok(())
    } else {
        Err(CommandError::new("Must home axis first"))
    }
}

fn command_name(command: &CalibrationCommand) -> &'static str {
    match command {
        CalibrationCommand::ProbeCalibrate => "PROBE_CALIBRATE",
        CalibrationCommand::ManualProbe => "MANUAL_PROBE",
        CalibrationCommand::TestZ(_) => "TESTZ",
        CalibrationCommand::Accept => "ACCEPT",
        CalibrationCommand::Adjusted => "ADJUSTED",
        CalibrationCommand::Abort => "ABORT",
        CalibrationCommand::BedScrewsAdjust => "BED_SCREWS_ADJUST",
        CalibrationCommand::ScrewsTiltCalculate => "SCREWS_TILT_CALCULATE",
        CalibrationCommand::BedMeshCalibrate { .. } => "BED_MESH_CALIBRATE",
        CalibrationCommand::BedMeshLoad { .. } => "BED_MESH_PROFILE",
        CalibrationCommand::BedMeshClear => "BED_MESH_CLEAR",
    }
}

/// The options Klipper stores for a mesh profile, with made-up points.
fn mesh_section() -> Map<String, Value> {
    let mut section = Map::new();
    for (option, value) in [
        ("version", "1"),
        ("points", "\n0.02, 0.01, -0.01\n0.03, 0.00, -0.02\n0.04, 0.01, -0.03"),
        ("x_count", "3"),
        ("y_count", "3"),
        ("min_x", "30.0"),
        ("max_x", "270.0"),
        ("min_y", "30.0"),
        ("max_y", "270.0"),
    ] {
        section.insert(option.to_owned(), json!(value));
    }
    section
}

fn round(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn printer() -> Calibration {
        Calibration::new(true, true, true)
    }

    #[test]
    fn manual_probe_bisects_between_heights_tried() {
        let mut calibration = printer();
        calibration.apply(CalibrationCommand::ProbeCalibrate, true).unwrap();
        calibration.apply(CalibrationCommand::TestZ(TestZ::By(-1.0)), true).unwrap();
        calibration.apply(CalibrationCommand::TestZ(TestZ::Up { half: true }), true).unwrap();
        let status = &calibration.status()["manual_probe"];
        assert_eq!(status["z_position"], 1.2);
        assert_eq!(status["z_position_lower"], 1.0);
        assert_eq!(status["z_position_upper"], 2.0);

        calibration.apply(CalibrationCommand::Accept, true).unwrap();
        assert_eq!(calibration.status()["manual_probe"]["is_active"], false);
        assert_eq!(calibration.pending_json()["probe"]["z_offset"], "1.800");
    }

    #[test]
    fn mode_commands_are_unknown_outside_the_mode() {
        let mut calibration = printer();
        assert!(calibration.apply(CalibrationCommand::Accept, true).is_err());
        assert!(calibration.apply(CalibrationCommand::TestZ(TestZ::By(0.1)), true).is_err());
        assert!(calibration.apply(CalibrationCommand::ProbeCalibrate, false).is_err());
    }

    #[test]
    fn bed_screws_end_after_every_screw_was_accepted_in_a_row() {
        let mut calibration = printer();
        calibration.apply(CalibrationCommand::BedScrewsAdjust, true).unwrap();
        calibration.apply(CalibrationCommand::Accept, true).unwrap();
        calibration.apply(CalibrationCommand::Adjusted, true).unwrap();
        assert_eq!(calibration.status()["bed_screws"]["current_screw"], 2);
        assert_eq!(calibration.status()["bed_screws"]["accepted_screws"], 0);
        for _ in SCREWS {
            calibration.apply(CalibrationCommand::Accept, true).unwrap();
        }
        assert_eq!(calibration.status()["bed_screws"]["is_active"], false);
    }

    #[test]
    fn mesh_calibration_is_active_and_pending() {
        let mut calibration = printer();
        calibration.apply(CalibrationCommand::BedMeshCalibrate { profile: None }, true).unwrap();
        assert_eq!(calibration.status()["bed_mesh"]["profile_name"], "default");
        assert!(calibration.has_pending());
        calibration.apply(CalibrationCommand::BedMeshClear, true).unwrap();
        assert_eq!(calibration.status()["bed_mesh"]["profile_name"], "");
        calibration.save();
        assert!(!calibration.has_pending());
    }
}
