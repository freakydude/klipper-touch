//! Virtual G-code files with metadata and thumbnail.

use serde_json::{json, Value};

const THUMBNAIL_SIZE: u32 = 300;

#[derive(Debug, Clone)]
pub struct GcodeFile {
    pub path: String,
    pub modified: f64,
    pub estimated_time: f64,
    /// Print duration of the last completed print; `None` if the file was never printed.
    pub last_print_duration: Option<f64>,
    pub layer_count: u32,
    pub first_layer_extruder_temperature: f64,
    pub first_layer_bed_temperature: f64,
}

impl GcodeFile {
    fn sample(path: &str, modified: f64, estimated_time: f64, layer_count: u32) -> Self {
        Self {
            path: path.to_owned(),
            modified,
            estimated_time,
            last_print_duration: None,
            layer_count,
            first_layer_extruder_temperature: 210.0,
            first_layer_bed_temperature: 60.0,
        }
    }

    fn printed(mut self, print_duration: f64) -> Self {
        self.last_print_duration = Some(print_duration);
        self
    }

    /// Weight per filament; the long job is printed in two materials.
    fn filament_weights(&self) -> Vec<f64> {
        let total = self.filament_total() * 0.003;
        if self.path == "long_job.gcode" {
            vec![total * 0.4, 55.0]
        } else {
            vec![total]
        }
    }

    fn filament_temps(&self) -> Vec<u32> {
        if self.path == "long_job.gcode" {
            vec![210, 235]
        } else {
            vec![210]
        }
    }

    fn filament_colors(&self) -> Vec<&'static str> {
        if self.path == "long_job.gcode" {
            vec!["#E6261F", "#2E7D32"]
        } else {
            vec!["#E6261F"]
        }
    }

    pub fn filament_total(&self) -> f64 {
        self.estimated_time * 2.5
    }

    fn size(&self) -> u64 {
        (self.estimated_time * 800.0) as u64
    }

    fn stem(&self) -> &str {
        self.path.strip_suffix(".gcode").unwrap_or(&self.path)
    }

    fn thumbnail_path(&self) -> String {
        format!(".thumbs/{}-{THUMBNAIL_SIZE}x{THUMBNAIL_SIZE}.png", self.stem())
    }

    fn thumbnail_svg(&self) -> String {
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}"><rect width="{size}" height="{size}" fill="#2b5876"/><text x="{middle}" y="{middle}" font-size="24" fill="white" text-anchor="middle">{label}</text></svg>"##,
            size = THUMBNAIL_SIZE,
            middle = THUMBNAIL_SIZE / 2,
            label = self.stem(),
        )
    }

    fn list_entry(&self) -> Value {
        json!({ "path": self.path, "modified": self.modified, "size": self.size(), "permissions": "rw" })
    }

    pub fn thumbnails(&self) -> Value {
        json!([{
            "width": THUMBNAIL_SIZE,
            "height": THUMBNAIL_SIZE,
            "size": self.thumbnail_svg().len(),
            "thumbnail_path": self.thumbnail_path(),
        }])
    }

    pub fn metadata(&self) -> Value {
        json!({
            "filename": self.path,
            "size": self.size(),
            "modified": self.modified,
            "slicer": "FakeSlicer",
            "slicer_version": "1.0.0",
            "estimated_time": self.estimated_time,
            "layer_count": self.layer_count,
            "layer_height": 0.2,
            "first_layer_height": 0.2,
            "object_height": f64::from(self.layer_count) * 0.2,
            "nozzle_diameter": 0.4,
            "filament_name": "Fake PLA",
            "filament_type": if self.path == "long_job.gcode" { "PLA;PETG" } else { "PLA" },
            "filament_total": self.filament_total(),
            "filament_weight_total": self.filament_weights().iter().sum::<f64>(),
            "filament_weights": self.filament_weights(),
            "filament_temps": self.filament_temps(),
            "filament_colors": self.filament_colors(),
            "referenced_tools": (0..self.filament_weights().len()).collect::<Vec<_>>(),
            "first_layer_extr_temp": self.first_layer_extruder_temperature,
            "first_layer_bed_temp": self.first_layer_bed_temperature,
            "gcode_start_byte": 0,
            "gcode_end_byte": self.size(),
            "job_id": null,
            "print_start_time": null,
            "uuid": format!("fake-{}", self.stem()),
            "thumbnails": [{
                "width": THUMBNAIL_SIZE,
                "height": THUMBNAIL_SIZE,
                "size": self.thumbnail_svg().len(),
                "relative_path": self.thumbnail_path(),
            }],
        })
    }
}

#[derive(Debug, Clone)]
pub struct Catalog {
    files: Vec<GcodeFile>,
}

impl Catalog {
    pub fn demo() -> Self {
        Self {
            files: vec![
                GcodeFile::sample("calibration_cube.gcode", 1_700_000_000.0, 1_500.0, 100).printed(1_380.0),
                GcodeFile::sample("benchy.gcode", 1_700_200_000.0, 7_200.0, 250).printed(7_650.0),
                GcodeFile::sample("long_job.gcode", 1_700_100_000.0, 36_000.0, 600),
            ],
        }
    }

    pub fn find(&self, path: &str) -> Option<&GcodeFile> {
        self.files.iter().find(|file| file.path == path)
    }

    pub fn list(&self) -> Value {
        Value::Array(self.files.iter().map(GcodeFile::list_entry).collect())
    }

    /// Job history, newest first: a completed job for every printed file, and a newer cancelled one for the benchy.
    pub fn history(&self) -> Value {
        let completed = self.files.iter().filter_map(|file| {
            let duration = file.last_print_duration?;
            Some(json!({
                "job_id": format!("{:06X}", file.modified as u64),
                "filename": file.path,
                "status": "completed",
                "start_time": file.modified + 60.0,
                "end_time": file.modified + 60.0 + duration,
                "print_duration": duration,
                "total_duration": duration + 240.0,
                "filament_used": file.filament_total() * 0.97,
                "exists": true,
                "auxiliary_data": [{ "provider": "spoolman", "name": "spool_ids", "value": [1], "description": "Spool IDs used", "units": null }],
            }))
        });
        let cancelled = json!({
            "job_id": "0000FF",
            "filename": "benchy.gcode",
            "status": "cancelled",
            "start_time": 1_700_300_000.0,
            "end_time": 1_700_300_600.0,
            "print_duration": 580.0,
            "total_duration": 600.0,
            "filament_used": 1_400.0,
            "exists": true,
        });
        let jobs: Vec<Value> = std::iter::once(cancelled).chain(completed).collect();
        json!({ "count": jobs.len(), "jobs": jobs })
    }

    pub fn thumbnail_svg(&self, thumbnail_path: &str) -> Option<String> {
        self.files.iter().find(|file| file.thumbnail_path() == thumbnail_path).map(GcodeFile::thumbnail_svg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_files_by_path() {
        let catalog = Catalog::demo();

        assert!(catalog.find("benchy.gcode").is_some());
        assert!(catalog.find("missing.gcode").is_none());
    }

    #[test]
    fn metadata_points_to_a_servable_thumbnail() {
        let catalog = Catalog::demo();
        let metadata = catalog.find("benchy.gcode").unwrap().metadata();
        let relative_path = metadata["thumbnails"][0]["relative_path"].as_str().unwrap();

        assert!(catalog.thumbnail_svg(relative_path).is_some());
    }

    #[test]
    fn history_lists_completed_jobs_after_the_newest_cancelled_one() {
        let history = Catalog::demo().history();

        assert_eq!(history["count"], 3);
        assert_eq!(history["jobs"][0]["status"], "cancelled");
        assert_eq!(history["jobs"][1]["status"], "completed");
    }

    #[test]
    fn list_contains_all_files_with_moonraker_fields() {
        let list = Catalog::demo().list();

        assert_eq!(list.as_array().unwrap().len(), 3);
        assert_eq!(list[0]["permissions"], "rw");
    }
}
