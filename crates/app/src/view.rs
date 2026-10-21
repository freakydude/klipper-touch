//! Reine Ableitungen aus dem Druckerzustand für die Anzeige; ohne Slint-Typen und damit testbar.

use crate::format;
use chrono::{DateTime, Local};
use klipper_touch_core::files::FileMetadata;
use klipper_touch_core::printer::{PrintJob, PrintState};

/// Ab diesem Fortschritt ist die Hochrechnung aus der bisherigen Druckzeit genauer als die Slicer-Schätzung.
const PROGRESS_FOR_EXTRAPOLATION: f64 = 0.05;
/// Querschnitt von 1,75-mm-Filament in mm².
const FILAMENT_CROSS_SECTION: f64 = std::f64::consts::PI * 0.875 * 0.875;
const NO_VALUE: &str = "-";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobTexts {
    pub name: String,
    pub elapsed: String,
    pub remaining: String,
    pub eta: String,
    pub layers: String,
    pub filament: String,
}

pub fn job_texts(job: &PrintJob, metadata: Option<&FileMetadata>, now: DateTime<Local>) -> JobTexts {
    let metadata = metadata.filter(|metadata| metadata.filename == job.filename);
    let remaining = if job.state.is_active() { remaining_seconds(job, metadata) } else { None };
    JobTexts {
        name: if job.filename.is_empty() { String::new() } else { format::job_name(&job.filename) },
        elapsed: if job.filename.is_empty() { NO_VALUE.to_owned() } else { format::duration(job.print_duration) },
        remaining: remaining.map_or_else(|| NO_VALUE.to_owned(), format::duration),
        eta: remaining.map_or_else(|| NO_VALUE.to_owned(), |seconds| format::finish_time(now, seconds)),
        layers: layers(job, metadata),
        filament: filament(job, metadata),
    }
}

fn remaining_seconds(job: &PrintJob, metadata: Option<&FileMetadata>) -> Option<f64> {
    if job.progress >= PROGRESS_FOR_EXTRAPOLATION && job.print_duration > 0.0 {
        return Some(job.print_duration / job.progress - job.print_duration);
    }
    metadata.and_then(|metadata| metadata.estimated_time).map(|estimate| (estimate - job.print_duration).max(0.0))
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

/// Volumenstrom in mm³/s aus der Extrudergeschwindigkeit.
pub fn volumetric_flow(extruder_velocity: f64) -> f64 {
    extruder_velocity.max(0.0) * FILAMENT_CROSS_SECTION
}

/// Zeile unter dem Dateinamen in der Dateiliste.
pub fn file_detail(modified: f64, size: u64, metadata: Option<&FileMetadata>) -> String {
    let mut parts = vec![format::unix_date(modified)];
    if let Some(metadata) = metadata {
        if let Some(estimate) = metadata.estimated_time {
            parts.push(format::duration(estimate));
        }
        if let Some(filament) = metadata.filament_total {
            parts.push(format::filament_length(filament));
        }
    }
    if parts.len() == 1 {
        parts.push(format::file_size(size));
    }
    parts.join("  ·  ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

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
    fn early_print_uses_slicer_estimate() {
        let job = PrintJob {
            state: PrintState::Printing,
            filename: "dir/cube.gcode".into(),
            progress: 0.01,
            print_duration: 600.0,
            filament_used: 1000.0,
            current_layer: Some(3),
            ..PrintJob::default()
        };

        let texts = job_texts(&job, Some(&metadata("dir/cube.gcode")), now());

        assert_eq!(texts.name, "cube");
        assert_eq!(texts.elapsed, "10m 00s");
        assert_eq!(texts.remaining, "50m 00s");
        assert_eq!(texts.eta, "12:50");
        assert_eq!(texts.layers, "3 / 100");
        assert_eq!(texts.filament, "1.00 m / 5.00 m");
    }

    #[test]
    fn later_print_extrapolates_from_progress() {
        let job = PrintJob {
            state: PrintState::Printing,
            filename: "a.gcode".into(),
            progress: 0.5,
            print_duration: 1800.0,
            ..PrintJob::default()
        };

        let texts = job_texts(&job, None, now());

        assert_eq!(texts.remaining, "30m 00s");
        assert_eq!(texts.layers, "-");
    }

    #[test]
    fn standby_shows_file_totals_and_no_eta() {
        let job = PrintJob { filename: "a.gcode".into(), ..PrintJob::default() };

        let texts = job_texts(&job, Some(&metadata("a.gcode")), now());

        assert_eq!(texts.remaining, "-");
        assert_eq!(texts.eta, "-");
        assert_eq!(texts.layers, "100");
        assert_eq!(texts.filament, "5.00 m");
    }

    #[test]
    fn metadata_of_other_file_is_ignored() {
        let job = PrintJob { filename: "a.gcode".into(), ..PrintJob::default() };

        assert_eq!(job_texts(&job, Some(&metadata("b.gcode")), now()).filament, "-");
    }

    #[test]
    fn flow_from_extruder_velocity() {
        assert!((volumetric_flow(10.0) - 24.05).abs() < 0.01);
        assert_eq!(volumetric_flow(-1.0), 0.0);
    }
}
