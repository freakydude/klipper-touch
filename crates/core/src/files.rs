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

/// This many files of the list get their metadata and thumbnail right away; the rest stays plain.
const DETAILS_LIMIT: usize = 40;

/// Everything known about one file of the list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FileInfo<'a> {
    pub entry: &'a FileEntry,
    pub metadata: Option<&'a FileMetadata>,
    pub last_print: Option<&'a LastPrint>,
}

/// The file list together with what arrives later for its files: metadata and the last completed prints. The data
/// comes from three queries; this joins them and decides which files still need details.
#[derive(Debug, Clone, Default)]
pub struct FileCatalog {
    entries: Vec<FileEntry>,
    metadata: HashMap<String, FileMetadata>,
    last_prints: HashMap<String, LastPrint>,
}

impl FileCatalog {
    pub fn set_entries(&mut self, entries: Vec<FileEntry>) {
        self.entries = entries;
    }

    pub fn entries(&self) -> &[FileEntry] {
        &self.entries
    }

    /// Files at the top of the list whose metadata has not been loaded yet.
    pub fn needing_details(&self) -> Vec<String> {
        self.entries
            .iter()
            .take(DETAILS_LIMIT)
            .filter(|entry| !self.metadata.contains_key(&entry.path))
            .map(|entry| entry.path.clone())
            .collect()
    }

    /// Metadata also arrives for the file of the running job, which need not be in the list.
    pub fn store_metadata(&mut self, metadata: FileMetadata) {
        self.metadata.insert(metadata.filename.clone(), metadata);
    }

    pub fn store_last_prints(&mut self, last_prints: HashMap<String, LastPrint>) {
        self.last_prints = last_prints;
    }

    pub fn metadata(&self, path: &str) -> Option<&FileMetadata> {
        self.metadata.get(path)
    }

    pub fn last_print(&self, path: &str) -> Option<&LastPrint> {
        self.last_prints.get(path)
    }

    pub fn position(&self, path: &str) -> Option<usize> {
        self.entries.iter().position(|entry| entry.path == path)
    }

    pub fn info<'a>(&'a self, entry: &'a FileEntry) -> FileInfo<'a> {
        FileInfo { entry, metadata: self.metadata(&entry.path), last_print: self.last_print(&entry.path) }
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

    fn entry(path: &str) -> FileEntry {
        FileEntry { path: path.to_owned(), modified: 1.0, size: 10 }
    }

    fn metadata(path: &str) -> FileMetadata {
        FileMetadata { filename: path.to_owned(), estimated_time: Some(60.0), ..FileMetadata::default() }
    }

    #[test]
    fn the_catalog_joins_entry_metadata_and_last_print_of_a_file() {
        let mut catalog = FileCatalog::default();
        catalog.set_entries(vec![entry("a.gcode"), entry("b.gcode")]);
        catalog.store_metadata(metadata("a.gcode"));
        catalog.store_last_prints(HashMap::from([(
            "a.gcode".to_owned(),
            LastPrint { print_duration: 5.0, filament_used: 1.0 },
        )]));

        let a = catalog.info(&catalog.entries()[0]);
        let b = catalog.info(&catalog.entries()[1]);

        assert_eq!((a.metadata.is_some(), a.last_print.is_some()), (true, true));
        assert_eq!((b.metadata.is_some(), b.last_print.is_some()), (false, false));
        assert_eq!(catalog.position("b.gcode"), Some(1));
        assert_eq!(catalog.position("c.gcode"), None);
    }

    #[test]
    fn only_files_without_metadata_at_the_top_of_the_list_need_details() {
        let mut catalog = FileCatalog::default();
        let names: Vec<String> = (0..DETAILS_LIMIT + 5).map(|index| format!("f{index}.gcode")).collect();
        catalog.set_entries(names.iter().map(|name| entry(name)).collect());
        catalog.store_metadata(metadata("f0.gcode"));

        let needing = catalog.needing_details();

        assert_eq!(needing.len(), DETAILS_LIMIT - 1);
        assert!(!needing.contains(&"f0.gcode".to_owned()));
        assert!(!needing.contains(&format!("f{DETAILS_LIMIT}.gcode")));
    }

    #[test]
    fn metadata_of_a_file_outside_the_list_is_kept() {
        let mut catalog = FileCatalog::default();
        catalog.store_metadata(metadata("running.gcode"));

        assert!(catalog.metadata("running.gcode").is_some());
        assert!(catalog.needing_details().is_empty());
    }
}
