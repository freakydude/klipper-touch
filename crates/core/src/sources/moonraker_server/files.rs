//! G-code files, their metadata and thumbnails.

use crate::sources::klipper::{PrintJob, PrintState};
use crate::sources::spoolman::Spool;
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

/// A file's preview ready to show. PNG and JPEG are decoded here, in a worker thread, so the UI only copies pixels;
/// SVG (from the fake Moonraker) stays as text for the UI toolkit.
#[derive(Debug, Clone, PartialEq)]
pub enum ThumbnailImage {
    Rgba { width: u32, height: u32, pixels: Vec<u8> },
    Svg(Vec<u8>),
}

impl ThumbnailImage {
    pub fn decode(data: &[u8]) -> Option<Self> {
        if looks_like_svg(data) {
            return Some(Self::Svg(data.to_vec()));
        }
        let rgba = image::load_from_memory(data).ok()?.into_rgba8();
        Some(Self::Rgba { width: rgba.width(), height: rgba.height(), pixels: rgba.into_raw() })
    }
}

fn looks_like_svg(data: &[u8]) -> bool {
    let start = &data[..data.len().min(256)];
    let text = String::from_utf8_lossy(start);
    let trimmed = text.trim_start();
    trimmed.starts_with("<svg") || (trimmed.starts_with("<?xml") && text.contains("<svg"))
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
    pub chamber_temp: Option<f64>,
    /// Per filament of the slicer, in grams; the index is the tool.
    pub filament_weights: Vec<f64>,
    pub filament_temps: Vec<f64>,
    /// `#RRGGBB` per filament.
    pub filament_colors: Vec<String>,
    /// Tools the print really uses.
    pub referenced_tools: Vec<usize>,
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
    /// The one spool the last print used, if Spoolman is known to have it.
    pub spool: Option<&'a Spool>,
}

impl FileInfo<'_> {
    /// A file whose metadata is not loaded yet may still come with a thumbnail; one whose metadata lists none will not.
    pub fn thumbnail_expected(&self) -> bool {
        self.metadata.is_none_or(|metadata| metadata.thumbnail_path().is_some())
    }
}

/// The file list together with what arrives later for its files: metadata and the last completed prints. The data
/// comes from three queries; this joins them and decides which files still need details.
#[derive(Debug, Clone, Default)]
pub struct FileCatalog {
    entries: Vec<FileEntry>,
    metadata: HashMap<String, FileMetadata>,
    last_prints: HashMap<String, LastPrint>,
    spools: HashMap<u32, Spool>,
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

    /// Spools of Spoolman that last prints may have used; a spool that is known stays until it is replaced.
    pub fn store_spools(&mut self, spools: impl IntoIterator<Item = Spool>) {
        self.spools.extend(spools.into_iter().map(|spool| (spool.id, spool)));
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
        let last_print = self.last_print(&entry.path);
        let spool = last_print
            .filter(|last_print| last_print.spool_ids.len() == 1)
            .and_then(|last_print| self.spools.get(&last_print.spool_ids[0]));
        FileInfo { entry, metadata: self.metadata(&entry.path), last_print, spool }
    }
}

/// What the last completed print of a file measured, from the Moonraker job history.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LastPrint {
    /// Seconds spent printing, without paused and idle time.
    pub print_duration: f64,
    /// Filament length in millimeters.
    pub filament_used: f64,
    /// Spoolman spools the job used (`auxiliary_data`); empty without Spoolman.
    pub spool_ids: Vec<u32>,
}

#[derive(Deserialize)]
struct AuxiliaryEntry {
    #[serde(default)]
    name: String,
    #[serde(default)]
    value: Value,
}

/// The newest job of the history once it has ended, for the end of the print the print page shows.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FinishedJob {
    pub filename: String,
    /// Moonraker's status: `completed`, `cancelled`, `error`, `klippy_shutdown`, `klippy_disconnect`, ...
    pub status: String,
    /// Unix seconds.
    pub end_time: f64,
}

impl FinishedJob {
    /// When `job` ended, if this history entry is that job: the same file and an outcome that matches its state.
    /// Every outcome besides completed and cancelled is a failure (shutdown, disconnect, server exit).
    pub fn end_of(&self, job: &PrintJob) -> Option<f64> {
        let matches = match job.state {
            PrintState::Complete => self.status == "completed",
            PrintState::Cancelled => self.status == "cancelled",
            PrintState::Error => !matches!(self.status.as_str(), "completed" | "cancelled" | "in_progress"),
            _ => false,
        };
        (matches && self.filename == job.filename).then_some(self.end_time)
    }
}

/// The job history as far as the UI uses it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrintHistory {
    pub last_prints: HashMap<String, LastPrint>,
    pub newest: Option<FinishedJob>,
}

pub fn parse_print_history(result: &Value) -> PrintHistory {
    let jobs: Vec<HistoryJob> =
        result.get("jobs").and_then(|jobs| serde_json::from_value(jobs.clone()).ok()).unwrap_or_default();
    PrintHistory { last_prints: last_completed_prints(&jobs), newest: jobs.first().and_then(HistoryJob::finished) }
}

#[derive(Deserialize)]
struct HistoryJob {
    #[serde(default)]
    filename: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    end_time: Option<f64>,
    #[serde(default)]
    print_duration: f64,
    #[serde(default)]
    filament_used: f64,
    #[serde(default)]
    auxiliary_data: Vec<AuxiliaryEntry>,
}

impl HistoryJob {
    fn spool_ids(&self) -> Vec<u32> {
        self.auxiliary_data
            .iter()
            .filter(|entry| entry.name == "spool_ids")
            .filter_map(|entry| entry.value.as_array())
            .flatten()
            .filter_map(|id| id.as_u64().and_then(|id| u32::try_from(id).ok()))
            .collect()
    }

    /// A job that is still running has no end time yet (`null`).
    fn finished(&self) -> Option<FinishedJob> {
        let end_time = self.end_time.filter(|_| self.status != "in_progress")?;
        Some(FinishedJob { filename: self.filename.clone(), status: self.status.clone(), end_time })
    }
}

/// The newest completed job per file; `server.history.list` returns the newest job first. Cancelled and failed
/// jobs are left out because their duration and filament say nothing about a whole print.
fn last_completed_prints(jobs: &[HistoryJob]) -> HashMap<String, LastPrint> {
    let mut last_prints = HashMap::new();
    for job in jobs.iter().filter(|job| job.status == "completed" && job.print_duration > 0.0) {
        last_prints.entry(job.filename.clone()).or_insert(LastPrint {
            print_duration: job.print_duration,
            filament_used: job.filament_used,
            spool_ids: job.spool_ids(),
        });
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

    #[test]
    fn thumbnails_are_decoded_to_pixels_and_svg_stays_text() {
        let mut png = Vec::new();
        image::RgbaImage::from_pixel(2, 3, image::Rgba([255, 0, 0, 255]))
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();

        assert!(matches!(
            ThumbnailImage::decode(&png),
            Some(ThumbnailImage::Rgba { width: 2, height: 3, ref pixels }) if pixels.len() == 2 * 3 * 4
        ));
        assert!(matches!(ThumbnailImage::decode(b"  <svg xmlns='x'/>"), Some(ThumbnailImage::Svg(_))));
        assert!(matches!(ThumbnailImage::decode(b"<?xml version='1.0'?><svg/>"), Some(ThumbnailImage::Svg(_))));
        assert_eq!(ThumbnailImage::decode(b"\x89PNG broken"), None);
    }
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

    fn last_completed_prints_of(result: Value) -> HashMap<String, LastPrint> {
        parse_print_history(&result).last_prints
    }

    #[test]
    fn the_newest_job_tells_when_the_shown_job_ended() {
        let history = parse_print_history(&json!({ "jobs": [
            { "filename": "a.gcode", "status": "cancelled", "end_time": 1_700_000_600.0 },
            { "filename": "a.gcode", "status": "completed", "end_time": 1_600_000_000.0 }
        ] }));
        let newest = history.newest.unwrap();
        let job = |state| PrintJob { state, filename: "a.gcode".into(), ..PrintJob::default() };

        assert_eq!(newest.end_of(&job(PrintState::Cancelled)), Some(1_700_000_600.0));
        assert_eq!(newest.end_of(&job(PrintState::Complete)), None);
        assert_eq!(newest.end_of(&PrintJob { filename: "b.gcode".into(), ..job(PrintState::Cancelled) }), None);
    }

    #[test]
    fn every_other_outcome_is_the_end_of_a_failed_job() {
        let newest = FinishedJob { filename: "a.gcode".into(), status: "klippy_shutdown".into(), end_time: 5.0 };
        let failed = PrintJob { state: PrintState::Error, filename: "a.gcode".into(), ..PrintJob::default() };

        assert_eq!(newest.end_of(&failed), Some(5.0));
    }

    #[test]
    fn a_running_job_has_not_ended() {
        let history = parse_print_history(&json!({ "jobs": [
            { "filename": "a.gcode", "status": "in_progress", "end_time": null }
        ] }));

        assert_eq!(history.newest, None);
    }

    #[test]
    fn last_completed_print_is_the_newest_one_that_finished() {
        let last_prints = last_completed_prints_of(json!({
            "count": 4,
            "jobs": [
                { "filename": "a.gcode", "status": "cancelled", "print_duration": 50.0, "filament_used": 100.0 },
                { "filename": "a.gcode", "status": "completed", "print_duration": 900.0, "filament_used": 2000.0 },
                { "filename": "a.gcode", "status": "completed", "print_duration": 1000.0, "filament_used": 2100.0 },
                { "filename": "b.gcode", "status": "error", "print_duration": 10.0, "filament_used": 5.0 }
            ]
        }));

        assert_eq!(last_prints.len(), 1);
        assert_eq!(
            last_prints["a.gcode"],
            LastPrint { print_duration: 900.0, filament_used: 2000.0, ..LastPrint::default() }
        );
    }

    #[test]
    fn the_spools_of_a_job_come_from_the_auxiliary_data() {
        let last_prints = last_completed_prints_of(json!({ "jobs": [{
            "filename": "a.gcode", "status": "completed", "print_duration": 900.0, "filament_used": 2000.0,
            "auxiliary_data": [
                { "provider": "other", "name": "other", "value": [9] },
                { "provider": "spoolman", "name": "spool_ids", "value": [18, 21], "units": null }
            ]
        }] }));

        assert_eq!(last_prints["a.gcode"].spool_ids, [18, 21]);
    }

    #[test]
    fn history_without_jobs_is_empty() {
        assert_eq!(parse_print_history(&json!({})), PrintHistory::default());
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
            LastPrint { print_duration: 5.0, filament_used: 1.0, ..LastPrint::default() },
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
    fn the_spool_of_the_last_print_is_known_only_for_a_print_with_one_spool() {
        let mut catalog = FileCatalog::default();
        catalog.set_entries(vec![entry("a.gcode"), entry("b.gcode")]);
        catalog.store_last_prints(HashMap::from([
            ("a.gcode".to_owned(), LastPrint { print_duration: 5.0, filament_used: 1.0, spool_ids: vec![18] }),
            ("b.gcode".to_owned(), LastPrint { print_duration: 5.0, filament_used: 1.0, spool_ids: vec![18, 21] }),
        ]));
        catalog.store_spools([Spool { id: 18, ..Spool::default() }]);

        assert_eq!(catalog.info(&catalog.entries()[0]).spool.map(|spool| spool.id), Some(18));
        assert_eq!(catalog.info(&catalog.entries()[1]).spool, None);
    }

    #[test]
    fn metadata_of_a_file_outside_the_list_is_kept() {
        let mut catalog = FileCatalog::default();
        catalog.store_metadata(metadata("running.gcode"));

        assert!(catalog.metadata("running.gcode").is_some());
        assert!(catalog.needing_details().is_empty());
    }

    #[test]
    fn a_thumbnail_is_expected_until_the_metadata_says_there_is_none() {
        let mut catalog = FileCatalog::default();
        let with = FileMetadata {
            filename: "with.gcode".into(),
            thumbnails: vec![Thumbnail { width: 32, height: 32, relative_path: "t.png".into() }],
            ..FileMetadata::default()
        };
        catalog.set_entries(vec![entry("unknown.gcode"), entry("with.gcode"), entry("without.gcode")]);
        catalog.store_metadata(with);
        catalog.store_metadata(metadata("without.gcode"));

        let expected: Vec<bool> =
            catalog.entries().iter().map(|entry| catalog.info(entry).thumbnail_expected()).collect();

        assert_eq!(expected, [true, true, false]);
    }
}
