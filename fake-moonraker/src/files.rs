//! Virtuelle G-Code-Dateien samt Metadaten und Vorschaubild.

use serde_json::{json, Value};

const THUMBNAIL_SIZE: u32 = 300;

#[derive(Debug, Clone)]
pub struct GcodeFile {
    pub path: String,
    pub estimated_time: f64,
    pub layer_count: u32,
    pub first_layer_extruder_temperature: f64,
    pub first_layer_bed_temperature: f64,
}

impl GcodeFile {
    fn sample(path: &str, estimated_time: f64, layer_count: u32) -> Self {
        Self {
            path: path.to_owned(),
            estimated_time,
            layer_count,
            first_layer_extruder_temperature: 210.0,
            first_layer_bed_temperature: 60.0,
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
        json!({ "path": self.path, "modified": 1_700_000_000.0, "size": self.size(), "permissions": "rw" })
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
            "modified": 1_700_000_000.0,
            "slicer": "FakeSlicer",
            "slicer_version": "1.0.0",
            "estimated_time": self.estimated_time,
            "layer_count": self.layer_count,
            "layer_height": 0.2,
            "first_layer_height": 0.2,
            "object_height": f64::from(self.layer_count) * 0.2,
            "nozzle_diameter": 0.4,
            "filament_name": "Fake PLA",
            "filament_type": "PLA",
            "filament_total": self.filament_total(),
            "filament_weight_total": self.filament_total() * 0.003,
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
                GcodeFile::sample("calibration_cube.gcode", 1_500.0, 100),
                GcodeFile::sample("benchy.gcode", 7_200.0, 250),
                GcodeFile::sample("long_job.gcode", 36_000.0, 600),
            ],
        }
    }

    pub fn find(&self, path: &str) -> Option<&GcodeFile> {
        self.files.iter().find(|file| file.path == path)
    }

    pub fn list(&self) -> Value {
        Value::Array(self.files.iter().map(GcodeFile::list_entry).collect())
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
    fn list_contains_all_files_with_moonraker_fields() {
        let list = Catalog::demo().list();

        assert_eq!(list.as_array().unwrap().len(), 3);
        assert_eq!(list[0]["permissions"], "rw");
    }
}
