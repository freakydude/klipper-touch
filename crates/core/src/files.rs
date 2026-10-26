//! G-code files, their metadata and thumbnails.

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct FileEntry {
    pub path: String,
    #[serde(default)]
    pub modified: f64,
    #[serde(default)]
    pub size: u64,
}

impl FileEntry {
    pub fn is_gcode(&self) -> bool {
        let lower = self.path.to_ascii_lowercase();
        [".gcode", ".g", ".gco"].iter().any(|extension| lower.ends_with(extension))
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Thumbnail {
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    #[serde(default)]
    pub relative_path: String,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(default)]
pub struct FileMetadata {
    pub filename: String,
    pub modified: f64,
    pub size: u64,
    pub slicer: Option<String>,
    pub estimated_time: Option<f64>,
    pub filament_total: Option<f64>,
    pub filament_weight_total: Option<f64>,
    pub filament_type: Option<String>,
    pub layer_height: Option<f64>,
    pub object_height: Option<f64>,
    pub nozzle_diameter: Option<f64>,
    pub first_layer_extr_temp: Option<f64>,
    pub first_layer_bed_temp: Option<f64>,
    pub thumbnails: Vec<Thumbnail>,
}

impl FileMetadata {
    pub fn largest_thumbnail(&self) -> Option<&Thumbnail> {
        self.thumbnails
            .iter()
            .filter(|thumbnail| !thumbnail.relative_path.is_empty())
            .max_by_key(|thumbnail| u64::from(thumbnail.width) * u64::from(thumbnail.height))
    }

    /// HTTP path of the largest thumbnail relative to the Moonraker server.
    pub fn thumbnail_path(&self) -> Option<String> {
        let thumbnail = self.largest_thumbnail()?;
        let directory = self.filename.rsplit_once('/').map_or("", |(directory, _)| directory);
        let relative = if directory.is_empty() {
            thumbnail.relative_path.clone()
        } else {
            format!("{directory}/{}", thumbnail.relative_path)
        };
        Some(format!("server/files/gcodes/{}", encode_path(&relative)))
    }
}

/// What the last completed print of a file measured, from the Moonraker job history.
#[derive(Debug, Clone, PartialEq)]
pub struct LastPrint {
    /// Seconds spent printing, without paused and idle time.
    pub print_duration: f64,
    /// Filament length in millimeters.
    pub filament_used: f64,
}

#[derive(Deserialize)]
struct HistoryJob {
    #[serde(default)]
    filename: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    print_duration: f64,
    #[serde(default)]
    filament_used: f64,
}

/// The newest completed job per file; `server.history.list` returns the newest job first. Cancelled and failed
/// jobs are left out because their duration and filament say nothing about a whole print.
pub fn parse_last_completed_prints(result: &Value) -> HashMap<String, LastPrint> {
    let jobs: Vec<HistoryJob> =
        result.get("jobs").and_then(|jobs| serde_json::from_value(jobs.clone()).ok()).unwrap_or_default();
    let mut last_prints = HashMap::new();
    for job in jobs.into_iter().filter(|job| job.status == "completed" && job.print_duration > 0.0) {
        last_prints
            .entry(job.filename)
            .or_insert(LastPrint { print_duration: job.print_duration, filament_used: job.filament_used });
    }
    last_prints
}

pub fn parse_file_list(result: &Value) -> Vec<FileEntry> {
    let mut files: Vec<FileEntry> = serde_json::from_value::<Vec<FileEntry>>(result.clone())
        .unwrap_or_default()
        .into_iter()
        .filter(FileEntry::is_gcode)
        .collect();
    files.sort_by(|left, right| right.modified.total_cmp(&left.modified).then_with(|| left.path.cmp(&right.path)));
    files
}

pub fn parse_metadata(result: &Value) -> Option<FileMetadata> {
    serde_json::from_value(result.clone()).ok()
}

/// Encodes each path segment for the URL; slashes are kept.
fn encode_path(path: &str) -> String {
    path.split('/').map(encode_segment).collect::<Vec<_>>().join("/")
}

fn encode_segment(segment: &str) -> String {
    segment
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (byte as char).to_string(),
            other => format!("%{other:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn file_list_keeps_gcode_sorted_newest_first() {
        let files = parse_file_list(&json!([
            { "path": "old.gcode", "modified": 1.0, "size": 10 },
            { "path": "notes.txt", "modified": 5.0, "size": 1 },
            { "path": "sub/new.gcode", "modified": 3.0, "size": 20 }
        ]));

        let paths: Vec<&str> = files.iter().map(|file| file.path.as_str()).collect();
        assert_eq!(paths, ["sub/new.gcode", "old.gcode"]);
    }

    #[test]
    fn picks_largest_thumbnail_and_builds_encoded_path() {
        let metadata = parse_metadata(&json!({
            "filename": "my parts/cube.gcode",
            "estimated_time": 1500.0,
            "thumbnails": [
                { "width": 32, "height": 32, "relative_path": ".thumbs/cube-32x32.png" },
                { "width": 300, "height": 300, "relative_path": ".thumbs/cube-300x300.png" }
            ]
        }))
        .unwrap();

        assert_eq!(metadata.estimated_time, Some(1500.0));
        assert_eq!(metadata.largest_thumbnail().unwrap().width, 300);
        assert_eq!(
            metadata.thumbnail_path().as_deref(),
            Some("server/files/gcodes/my%20parts/.thumbs/cube-300x300.png")
        );
    }

    #[test]
    fn last_completed_print_is_the_newest_one_that_finished() {
        let last_prints = parse_last_completed_prints(&json!({
            "count": 4,
            "jobs": [
                { "filename": "a.gcode", "status": "cancelled", "print_duration": 50.0, "filament_used": 100.0 },
                { "filename": "a.gcode", "status": "completed", "print_duration": 900.0, "filament_used": 2000.0 },
                { "filename": "a.gcode", "status": "completed", "print_duration": 1000.0, "filament_used": 2100.0 },
                { "filename": "b.gcode", "status": "error", "print_duration": 10.0, "filament_used": 5.0 }
            ]
        }));

        assert_eq!(last_prints.len(), 1);
        assert_eq!(last_prints["a.gcode"], LastPrint { print_duration: 900.0, filament_used: 2000.0 });
    }

    #[test]
    fn history_without_jobs_is_empty() {
        assert!(parse_last_completed_prints(&json!({})).is_empty());
    }

    #[test]
    fn metadata_without_thumbnails_has_no_path() {
        let metadata = parse_metadata(&json!({ "filename": "a.gcode" })).unwrap();

        assert_eq!(metadata.thumbnail_path(), None);
    }
}
