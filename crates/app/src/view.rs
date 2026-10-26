//! Pure display derivations from the printer state; free of Slint types and therefore testable.

use crate::format;
use chrono::{DateTime, Local};
use klipper_touch_core::estimates;
use klipper_touch_core::files::{FileMetadata, LastPrint};
use klipper_touch_core::printer::{
    FlowGuard, GateAvailability, Heater, LoadedFilament, MmuSensor, PrintJob, PrintState, SyncFeedbackState,
};
use klipper_touch_core::{Provenance, Sourced};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A sent target that Klipper has not reported back by then is given up (e.g. rejected command).
const PENDING_TARGET_TIMEOUT: Duration = Duration::from_secs(3);

/// Cross section of 1.75 mm filament in mm².
const FILAMENT_CROSS_SECTION: f64 = std::f64::consts::PI * 0.875 * 0.875;
const NO_VALUE: &str = "-";
const ORIGIN_PACE_ONLY: &str = "measured pace only";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobTexts {
    pub name: String,
    pub elapsed: String,
    pub remaining: String,
    pub eta: String,
    pub layers: String,
    pub filament: String,
    /// Source of the expected print time behind the remaining time while a job runs; for the log.
    pub expectation_origin: Option<&'static str>,
}

pub fn job_texts(
    job: &PrintJob,
    metadata: Option<&FileMetadata>,
    last_print: Option<&LastPrint>,
    now: DateTime<Local>,
) -> JobTexts {
    let metadata = metadata.filter(|metadata| metadata.filename == job.filename);
    let expected = if job.state.is_active() { estimates::expected_print_seconds(metadata, last_print) } else { None };
    let remaining = job
        .state
        .is_active()
        .then(|| estimates::remaining_print_seconds(job, expected.map(|expected| expected.value)))
        .flatten();
    JobTexts {
        name: if job.filename.is_empty() { String::new() } else { format::job_name(&job.filename) },
        elapsed: if job.filename.is_empty() { NO_VALUE.to_owned() } else { format::duration(job.print_duration) },
        remaining: remaining.map_or_else(|| NO_VALUE.to_owned(), format::duration),
        eta: remaining.map_or_else(|| NO_VALUE.to_owned(), |seconds| format::finish_time(now, seconds)),
        layers: layers(job, metadata),
        filament: filament(job, metadata),
        expectation_origin: job
            .state
            .is_active()
            .then(|| expected.map_or(ORIGIN_PACE_ONLY, |expected| expected.origin)),
    }
}

fn layers(job: &PrintJob, metadata: Option<&FileMetadata>) -> String {
    let total = job.total_layer.or_else(|| {
        let metadata = metadata?;
        let height = metadata.object_height?;
        let layer = metadata.layer_height.filter(|layer| *layer > 0.0)?;
        Some((height / layer).round() as u32)
    });
    match (job.current_layer, total) {
        (Some(current), Some(total)) => format!("{current} / {total}"),
        (Some(current), None) => current.to_string(),
        (None, Some(total)) if job.state == PrintState::Standby => format!("{total}"),
        _ => NO_VALUE.to_owned(),
    }
}

fn filament(job: &PrintJob, metadata: Option<&FileMetadata>) -> String {
    let total = metadata.and_then(|metadata| metadata.filament_total);
    match (job.state, total) {
        (PrintState::Standby, Some(total)) => format::filament_length(total),
        (_, Some(total)) if job.filament_used > 0.0 => {
            format!("{} / {}", format::filament_length(job.filament_used), format::filament_length(total))
        }
        _ if job.filament_used > 0.0 => format::filament_length(job.filament_used),
        _ => NO_VALUE.to_owned(),
    }
}

/// Volumetric flow in mm³/s from the extruder velocity.
pub fn volumetric_flow(extruder_velocity: f64) -> f64 {
    extruder_velocity.max(0.0) * FILAMENT_CROSS_SECTION
}

/// Line below the file name in the file list: modified, filament, nozzle, print time. Values of the last completed
/// print win over what the slicer predicted; predictions are marked with `~`.
pub fn file_detail(
    modified: f64,
    size: u64,
    metadata: Option<&FileMetadata>,
    last_print: Option<&LastPrint>,
) -> String {
    let mut parts = vec![format::unix_date(modified)];
    if let Some(metadata) = metadata {
        parts.extend(file_filament(metadata, last_print));
        parts.extend(metadata.nozzle_diameter.map(|diameter| format!("{diameter} nozzle")));
        parts.extend(file_print_time(metadata, last_print));
    }
    if parts.len() == 1 {
        parts.push(format::file_size(size));
    }
    parts.join("  ·  ")
}

fn file_filament(metadata: &FileMetadata, last_print: Option<&LastPrint>) -> Option<String> {
    let weight =
        estimates::filament_weight_grams(metadata, last_print).map(|grams| show(grams, format::filament_weight));
    let parts: Vec<String> =
        [metadata.filament_type.clone().filter(|kind| !kind.is_empty()), weight].into_iter().flatten().collect();
    (!parts.is_empty()).then(|| parts.join(" "))
}

fn file_print_time(metadata: &FileMetadata, last_print: Option<&LastPrint>) -> Option<String> {
    estimates::expected_print_seconds(Some(metadata), last_print).map(|seconds| show(seconds, format::duration))
}

/// A prediction carries a `~` so that it is not mistaken for a measurement.
fn show(value: Sourced<f64>, format: impl FnOnce(f64) -> String) -> String {
    let text = format(value.value);
    match value.provenance {
        Provenance::Measured => text,
        Provenance::Estimated => format!("~{text}"),
    }
}

/// Linux names wireless interfaces `wlan0`, `wlp2s0`, ...
pub fn is_wireless_interface(name: &str) -> bool {
    name.starts_with("wl")
}

/// Names of the MMU sensors as the operator reads them; unknown ones lose the `mmu_` prefix and get spaces.
pub fn mmu_sensor_label(name: &str) -> String {
    let known = match name {
        "mmu_entry" => "Entry",
        "mmu_exit" => "Exit",
        "mmu_shared_exit" => "Shared exit",
        "encoder" => "Encoder",
        "extruder" => "Extruder",
        "toolhead" => "Toolhead",
        "filament_compression" => "Compression",
        "filament_tension" => "Tension",
        "filament_proportional" => "Proportional",
        "mmu_gear_touch" => "Gear touch",
        "mmu_ext_touch" => "Extruder touch",
        "mmu_sel_touch" => "Selector touch",
        "mmu_sel_home" => "Selector home",
        "mmu_nfc" => "NFC",
        _ => "",
    };
    if !known.is_empty() {
        return known.to_owned();
    }
    let words = name.strip_prefix("mmu_").unwrap_or(name).replace('_', " ");
    let mut characters = words.chars();
    characters.next().map_or_else(String::new, |first| first.to_uppercase().chain(characters).collect())
}

/// A Klipper filament sensor that belongs to the MMU itself (gates and buffer). `toolhead` and `extruder` stay common:
/// they sit behind the MMU and matter without one. `object` is the Klipper object name, for example
/// `filament_switch_sensor mmu_entry_0`.
pub fn is_mmu_klipper_sensor(object: &str) -> bool {
    let name = object.split_once(' ').map_or(object, |(_, name)| name);
    let name = name.rsplit_once(':').map_or(name, |(_, name)| name);
    name.starts_with("mmu_") || matches!(name, "filament_compression" | "filament_tension" | "filament_proportional")
}

/// The sensors of the MMU for the chip list. The three buffer sensors are left out when a buffer gauge shows them.
pub fn mmu_chip_sensors(sensors: &[MmuSensor], has_buffer: bool) -> Vec<&MmuSensor> {
    const BUFFER_SENSORS: [&str; 3] = ["filament_compression", "filament_tension", "filament_proportional"];
    sensors.iter().filter(|sensor| !(has_buffer && BUFFER_SENSORS.contains(&sensor.name.as_str()))).collect()
}

/// `triggered` means filament is present at the sensor; `None` is a disabled sensor.
pub fn mmu_sensor_state(triggered: Option<bool>) -> &'static str {
    match triggered {
        None => "disabled",
        Some(false) => "open",
        Some(true) => "triggered",
    }
}

pub fn mmu_availability(availability: GateAvailability) -> &'static str {
    match availability {
        GateAvailability::Unknown => "unknown",
        GateAvailability::Empty => "empty",
        GateAvailability::Available => "available",
        GateAvailability::Buffered => "buffered",
    }
}

pub fn mmu_filament(filament: LoadedFilament) -> &'static str {
    match filament {
        LoadedFilament::Loaded => "loaded",
        LoadedFilament::Unloaded => "unloaded",
        LoadedFilament::Unknown => "unknown",
    }
}

pub fn mmu_buffer_state(state: SyncFeedbackState) -> &'static str {
    match state {
        SyncFeedbackState::Compressed => "compressed",
        SyncFeedbackState::Neutral => "neutral",
        SyncFeedbackState::Tension => "tension",
        SyncFeedbackState::Inactive => "inactive",
        SyncFeedbackState::Disabled => "disabled",
    }
}

/// One word for the operator: a trigger wins over monitoring, which wins over ready.
pub fn mmu_flowguard_state(flowguard: &FlowGuard) -> &'static str {
    if !flowguard.enabled {
        "off"
    } else if !flowguard.trigger.is_empty() {
        "triggered"
    } else if flowguard.active {
        "monitoring"
    } else {
        "ready"
    }
}

/// Heater targets sent but not yet reported by Klipper, so quick taps on + and − build on each other
/// and the new target shows at once.
#[derive(Debug, Default)]
pub struct PendingTargets {
    targets: HashMap<String, (f64, Instant)>,
}

impl PendingTargets {
    pub fn request(&mut self, object: &str, target: f64, now: Instant) {
        self.targets.insert(object.to_owned(), (target, now));
    }

    /// Shows pending targets on the reported heaters; forgets confirmed and outdated ones.
    pub fn apply(&mut self, heaters: &mut [Heater], now: Instant) {
        self.targets.retain(|object, (target, sent)| {
            now.duration_since(*sent) < PENDING_TARGET_TIMEOUT
                && heaters.iter().any(|heater| &heater.object == object && heater.target != *target)
        });
        for heater in heaters {
            if let Some((target, _)) = self.targets.get(&heater.object) {
                heater.target = *target;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn pending_targets_cover_the_delay_until_klipper_reports_back() {
        let start = Instant::now();
        let reported = |target| vec![Heater { object: "extruder".into(), target, ..Heater::default() }];
        let mut pending = PendingTargets::default();
        pending.request("extruder", 45.0, start);
        pending.request("extruder", 50.0, start);

        let mut heaters = reported(45.0);
        pending.apply(&mut heaters, start + Duration::from_millis(200));
        assert_eq!(heaters[0].target, 50.0);

        let mut heaters = reported(50.0);
        pending.apply(&mut heaters, start + Duration::from_millis(400));
        pending.request("extruder", 55.0, start + Duration::from_millis(500));
        let mut heaters = reported(50.0);
        pending.apply(&mut heaters, start + PENDING_TARGET_TIMEOUT + Duration::from_secs(1));
        assert_eq!(heaters[0].target, 50.0);
    }

    #[test]
    fn recognizes_wireless_interfaces() {
        assert!(is_wireless_interface("wlan0"));
        assert!(is_wireless_interface("wlp3s0"));
        assert!(!is_wireless_interface("eth0"));
        assert!(!is_wireless_interface("enp1s0"));
    }

    fn now() -> DateTime<Local> {
        Local.with_ymd_and_hms(2026, 10, 21, 12, 0, 0).unwrap()
    }

    fn metadata(filename: &str) -> FileMetadata {
        FileMetadata {
            filename: filename.to_owned(),
            estimated_time: Some(3600.0),
            filament_total: Some(5000.0),
            layer_height: Some(0.2),
            object_height: Some(20.0),
            ..FileMetadata::default()
        }
    }

    #[test]
    fn print_on_the_expected_pace_counts_down_the_slicer_estimate() {
        let job = PrintJob {
            state: PrintState::Printing,
            filename: "dir/cube.gcode".into(),
            progress: 0.1,
            print_duration: 360.0,
            filament_used: 1000.0,
            current_layer: Some(3),
            ..PrintJob::default()
        };

        let texts = job_texts(&job, Some(&metadata("dir/cube.gcode")), None, now());

        assert_eq!(texts.name, "cube");
        assert_eq!(texts.elapsed, "6m 00s");
        assert_eq!(texts.remaining, "54m 00s");
        assert_eq!(texts.eta, "12:54");
        assert_eq!(texts.layers, "3 / 100");
        assert_eq!(texts.filament, "1.00 m / 5.00 m");
    }

    #[test]
    fn last_measured_print_is_the_expectation_before_the_slicer_estimate() {
        let job = PrintJob {
            state: PrintState::Printing,
            filename: "a.gcode".into(),
            progress: 0.1,
            print_duration: 420.0,
            ..PrintJob::default()
        };
        let last_print = LastPrint { print_duration: 4200.0, filament_used: 4800.0 };

        let texts = job_texts(&job, Some(&metadata("a.gcode")), Some(&last_print), now());

        assert_eq!(texts.remaining, "1h 03m");
        assert_eq!(texts.expectation_origin, Some(estimates::ORIGIN_LAST_PRINT));
    }

    #[test]
    fn without_expectation_the_pace_is_extrapolated_once_progress_exists() {
        let job = PrintJob {
            state: PrintState::Printing,
            filename: "a.gcode".into(),
            progress: 0.5,
            print_duration: 1800.0,
            ..PrintJob::default()
        };

        let texts = job_texts(&job, None, None, now());

        assert_eq!(texts.remaining, "30m 00s");
        assert_eq!(texts.layers, "-");
        assert_eq!(texts.expectation_origin, Some(ORIGIN_PACE_ONLY));
    }

    #[test]
    fn standby_shows_file_totals_and_no_eta() {
        let job = PrintJob { filename: "a.gcode".into(), ..PrintJob::default() };

        let texts = job_texts(&job, Some(&metadata("a.gcode")), None, now());

        assert_eq!(texts.remaining, "-");
        assert_eq!(texts.eta, "-");
        assert_eq!(texts.expectation_origin, None);
        assert_eq!(texts.layers, "100");
        assert_eq!(texts.filament, "5.00 m");
    }

    #[test]
    fn metadata_of_other_file_is_ignored() {
        let job = PrintJob { filename: "a.gcode".into(), ..PrintJob::default() };

        assert_eq!(job_texts(&job, Some(&metadata("b.gcode")), None, now()).filament, "-");
    }

    fn file_metadata() -> FileMetadata {
        FileMetadata {
            filename: "a.gcode".into(),
            estimated_time: Some(3600.0),
            filament_total: Some(10_000.0),
            filament_weight_total: Some(30.0),
            filament_type: Some("PLA".into()),
            nozzle_diameter: Some(0.4),
            ..FileMetadata::default()
        }
    }

    fn date_of(modified: f64) -> String {
        format::unix_date(modified)
    }

    #[test]
    fn unprinted_file_shows_slicer_values_marked_as_estimates() {
        let detail = file_detail(1_700_000_000.0, 100, Some(&file_metadata()), None);

        assert_eq!(detail, format!("{}  ·  PLA ~30 g  ·  0.4 nozzle  ·  ~1h 00m", date_of(1_700_000_000.0)));
    }

    #[test]
    fn printed_file_shows_measured_time_and_scaled_weight() {
        let last_print = LastPrint { print_duration: 3300.0, filament_used: 9_000.0 };

        let detail = file_detail(1_700_000_000.0, 100, Some(&file_metadata()), Some(&last_print));

        assert_eq!(detail, format!("{}  ·  PLA 27 g  ·  0.4 nozzle  ·  55m 00s", date_of(1_700_000_000.0)));
    }

    #[test]
    fn file_without_metadata_shows_its_size() {
        assert_eq!(file_detail(1_700_000_000.0, 512, None, None), format!("{}  ·  512 B", date_of(1_700_000_000.0)));
    }

    #[test]
    fn missing_slicer_fields_are_left_out() {
        let metadata =
            FileMetadata { filename: "a.gcode".into(), nozzle_diameter: Some(0.6), ..FileMetadata::default() };

        let detail = file_detail(1_700_000_000.0, 100, Some(&metadata), None);

        assert_eq!(detail, format!("{}  ·  0.6 nozzle", date_of(1_700_000_000.0)));
    }

    #[test]
    fn mmu_sensors_get_readable_names() {
        assert_eq!(mmu_sensor_label("mmu_shared_exit"), "Shared exit");
        assert_eq!(mmu_sensor_label("filament_tension"), "Tension");
        assert_eq!(mmu_sensor_label("mmu_future_probe"), "Future probe");
        assert_eq!(mmu_sensor_label("hotend"), "Hotend");
        assert_eq!(mmu_sensor_label(""), "");
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
    fn buffer_sensors_are_hidden_only_when_the_gauge_shows_them() {
        let sensors = ["mmu_entry", "filament_tension", "filament_proportional", "toolhead"]
            .map(|name| MmuSensor { name: name.into(), triggered: Some(false) });
        let names = |has_buffer| -> Vec<String> {
            mmu_chip_sensors(&sensors, has_buffer).iter().map(|sensor| sensor.name.clone()).collect()
        };

        assert_eq!(names(true), ["mmu_entry", "toolhead"]);
        assert_eq!(names(false).len(), 4);
    }

    #[test]
    fn mmu_states_become_words_for_the_ui() {
        assert_eq!([None, Some(false), Some(true)].map(mmu_sensor_state), ["disabled", "open", "triggered"]);
        let flowguard = |enabled, active, trigger: &str| FlowGuard {
            enabled,
            active,
            trigger: trigger.into(),
            ..FlowGuard::default()
        };
        assert_eq!(mmu_flowguard_state(&flowguard(false, true, "clog")), "off");
        assert_eq!(mmu_flowguard_state(&flowguard(true, true, "clog")), "triggered");
        assert_eq!(mmu_flowguard_state(&flowguard(true, true, "")), "monitoring");
        assert_eq!(mmu_flowguard_state(&flowguard(true, false, "")), "ready");
    }

    #[test]
    fn flow_from_extruder_velocity() {
        assert!((volumetric_flow(10.0) - 24.05).abs() < 0.01);
        assert_eq!(volumetric_flow(-1.0), 0.0);
    }
}
