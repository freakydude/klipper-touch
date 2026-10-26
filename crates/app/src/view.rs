//! Pure display derivations from the printer state; free of Slint types and therefore testable.

use crate::format;
use chrono::{DateTime, Local};
use klipper_touch_core::estimates;
use klipper_touch_core::files::{FileInfo, FileMetadata, LastPrint};
use klipper_touch_core::printer::{
    FlowGuardState, GateAvailability, LoadedFilament, MmuSensor, PrintJob, PrintState, SyncFeedbackState,
};
use klipper_touch_core::{Provenance, Sourced};

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
    let total = estimates::total_layers(job, metadata).map(|total| total.value);
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
            format!("{} / {}", format::filament_meters(job.filament_used), format::filament_length(total))
        }
        _ if job.filament_used > 0.0 => format::filament_length(job.filament_used),
        _ => NO_VALUE.to_owned(),
    }
}

/// What a row of the file list shows; an empty text is a fact the file does not have.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileFacts {
    pub modified: String,
    pub material: String,
    pub nozzle: String,
    pub time: String,
    pub extruder_temperature: String,
    pub bed_temperature: String,
    pub chamber_temperature: String,
}

/// The facts of a file for its row. Values of the last completed print win over what the slicer predicted;
/// predictions are marked with `~`. The temperatures are the slicer's settings, not predictions.
pub fn file_facts(info: &FileInfo) -> FileFacts {
    let mut facts = FileFacts { modified: format::unix_date(info.entry.modified), ..FileFacts::default() };
    if let Some(metadata) = info.metadata {
        facts.material = file_filament(metadata, info.last_print).unwrap_or_default();
        facts.nozzle =
            metadata.nozzle_diameter.filter(|diameter| *diameter > 0.0).map_or_else(String::new, |d| d.to_string());
        facts.time = file_print_time(metadata, info.last_print).unwrap_or_default();
        facts.extruder_temperature = degrees(metadata.first_layer_extr_temp);
        facts.bed_temperature = degrees(metadata.first_layer_bed_temp);
        facts.chamber_temperature = degrees(metadata.chamber_temp);
    }
    facts
}

/// A temperature of 0 means that the slicer sets none.
fn degrees(temperature: Option<f64>) -> String {
    temperature.filter(|degrees| *degrees > 0.0).map_or_else(String::new, |degrees| format!("{degrees:.0}°"))
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

pub fn mmu_flowguard_state(state: FlowGuardState) -> &'static str {
    match state {
        FlowGuardState::Off => "off",
        FlowGuardState::Triggered => "triggered",
        FlowGuardState::Monitoring => "monitoring",
        FlowGuardState::Ready => "ready",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use klipper_touch_core::files::FileEntry;

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
        assert_eq!(texts.filament, "1.00 / 5.00 m");
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

    fn facts_of(metadata: Option<&FileMetadata>, last_print: Option<&LastPrint>) -> FileFacts {
        let entry = FileEntry { path: "a.gcode".into(), modified: 1_700_000_000.0, size: 100 };
        file_facts(&FileInfo { entry: &entry, metadata, last_print })
    }

    fn slicer_settings() -> FileMetadata {
        FileMetadata {
            first_layer_extr_temp: Some(215.0),
            first_layer_bed_temp: Some(60.0),
            chamber_temp: Some(45.0),
            ..file_metadata()
        }
    }

    #[test]
    fn unprinted_file_shows_slicer_values_marked_as_estimates() {
        let facts = facts_of(Some(&slicer_settings()), None);

        assert_eq!(
            facts,
            FileFacts {
                modified: format::unix_date(1_700_000_000.0),
                material: "PLA ~30 g".into(),
                nozzle: "0.4".into(),
                time: "~1h 00m".into(),
                extruder_temperature: "215°".into(),
                bed_temperature: "60°".into(),
                chamber_temperature: "45°".into(),
            }
        );
    }

    #[test]
    fn printed_file_shows_measured_time_and_scaled_weight() {
        let last_print = LastPrint { print_duration: 3300.0, filament_used: 9_000.0 };

        let facts = facts_of(Some(&file_metadata()), Some(&last_print));

        assert_eq!((facts.material.as_str(), facts.time.as_str()), ("PLA 27 g", "55m 00s"));
    }

    #[test]
    fn file_without_metadata_shows_only_its_date() {
        let facts = facts_of(None, None);

        assert_eq!(facts, FileFacts { modified: format::unix_date(1_700_000_000.0), ..FileFacts::default() });
    }

    #[test]
    fn missing_slicer_fields_and_zero_temperatures_are_left_out() {
        let metadata = FileMetadata {
            filename: "a.gcode".into(),
            nozzle_diameter: Some(0.6),
            chamber_temp: Some(0.0),
            first_layer_bed_temp: Some(0.0),
            ..FileMetadata::default()
        };

        let facts = facts_of(Some(&metadata), None);

        assert_eq!(facts.nozzle, "0.6");
        assert_eq!((facts.material.as_str(), facts.time.as_str()), ("", ""));
        assert_eq!(
            (facts.extruder_temperature.as_str(), facts.bed_temperature.as_str(), facts.chamber_temperature.as_str()),
            ("", "", "")
        );
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
        assert_eq!(
            [FlowGuardState::Off, FlowGuardState::Triggered, FlowGuardState::Monitoring, FlowGuardState::Ready]
                .map(mmu_flowguard_state),
            ["off", "triggered", "monitoring", "ready"]
        );
    }
}
