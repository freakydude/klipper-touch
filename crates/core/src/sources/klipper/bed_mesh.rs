//! The active bed mesh as Klipper probed it: `bed_mesh.probed_matrix` between `mesh_min` and `mesh_max`. Klipper
//! fills the matrix to a full rectangle and does not mark which values it did not measure where they stand; the
//! configuration tells (`klippy/extras/bed_mesh.py`, read 2026-10-29):
//! - round beds (`mesh_radius`): only points inside the circle are probed, each row is padded with its outermost value;
//! - `faulty_region_N_min/max`: a point inside is the average of up to four points probed at the region's border.

use serde_json::Value;

/// Distance in mm within which a point counts as inside the circle or a faulty region (Klipper compares exactly or
/// with 0.00001; positions come rounded through JSON).
const TOLERANCE: f64 = 0.01;
/// Klipper numbers faulty regions from 1 and stops at the first missing one (`bed_mesh.py`: `range(1, 100, 1)`).
const MAX_FAULTY_REGIONS: usize = 99;

/// One value of `probed_matrix`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MeshPoint {
    /// Probed at this point.
    Measured(f64),
    /// Inside a faulty region: the average of points probed at the region's border.
    Averaged(f64),
    /// Outside the circle of a round bed: a copy of the row's outermost value, not a measurement.
    Filled,
}

impl MeshPoint {
    /// The height Klipper uses here, unless it is only a copy.
    pub fn height(self) -> Option<f64> {
        match self {
            Self::Measured(height) | Self::Averaged(height) => Some(height),
            Self::Filled => None,
        }
    }
}

/// Points of the mesh covered by one faulty region, as columns and rows of the matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshSpan {
    pub column: usize,
    pub row: usize,
    pub columns: usize,
    pub rows: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BedMesh {
    /// Front left corner in mm, the first point of row 0.
    pub min: [f64; 2],
    /// Back right corner in mm.
    pub max: [f64; 2],
    /// Row 0 at the front, each row from left to right.
    pub rows: Vec<Vec<MeshPoint>>,
    /// The points each faulty region replaced.
    pub faulty: Vec<MeshSpan>,
}

impl BedMesh {
    pub fn columns(&self) -> usize {
        self.rows.first().map_or(0, Vec::len)
    }

    fn heights(&self) -> impl Iterator<Item = f64> + '_ {
        self.rows.iter().flatten().filter_map(|point| point.height())
    }

    pub fn lowest(&self) -> Option<f64> {
        self.heights().reduce(f64::min)
    }

    pub fn highest(&self) -> Option<f64> {
        self.heights().reduce(f64::max)
    }

    /// Highest minus lowest point.
    pub fn range(&self) -> Option<f64> {
        Some(self.highest()? - self.lowest()?)
    }

    pub fn averaged(&self) -> usize {
        self.rows.iter().flatten().filter(|point| matches!(point, MeshPoint::Averaged(_))).count()
    }
}

/// `object` is the status of `bed_mesh`, `settings` its section of the configuration. `None` without an active mesh.
pub(super) fn read(object: &Value, settings: Option<&Value>) -> Option<BedMesh> {
    let matrix = object.get("probed_matrix")?.as_array()?;
    let rows: Vec<Vec<f64>> = matrix
        .iter()
        .map(|row| row.as_array().map(|row| row.iter().filter_map(Value::as_f64).collect()))
        .collect::<Option<_>>()?;
    let columns = rows.first().map_or(0, Vec::len);
    if columns < 2 || rows.len() < 2 || rows.iter().any(|row| row.len() != columns) {
        return None;
    }
    let min = pair(object.get("mesh_min"))?;
    let max = pair(object.get("mesh_max"))?;
    let grid = Grid { min, max, columns, rows: rows.len() };
    let round = settings.and_then(|settings| settings.get("mesh_radius")).is_some();
    let regions = faulty_regions(settings);
    let points = rows
        .iter()
        .enumerate()
        .map(|(row, values)| {
            values
                .iter()
                .enumerate()
                .map(|(column, &height)| {
                    let position = grid.position(column, row);
                    if round && !grid.in_circle(position) {
                        MeshPoint::Filled
                    } else if regions.iter().any(|region| region.contains(position)) {
                        MeshPoint::Averaged(height)
                    } else {
                        MeshPoint::Measured(height)
                    }
                })
                .collect()
        })
        .collect();
    let faulty = regions.iter().filter_map(|region| grid.span(region)).collect();
    Some(BedMesh { min, max, rows: points, faulty })
}

/// Where the points of the matrix lie: evenly spaced from `min` to `max`, as Klipper generates them.
struct Grid {
    min: [f64; 2],
    max: [f64; 2],
    columns: usize,
    rows: usize,
}

impl Grid {
    fn step(&self, axis: usize, count: usize) -> f64 {
        (self.max[axis] - self.min[axis]) / (count - 1) as f64
    }

    fn position(&self, column: usize, row: usize) -> [f64; 2] {
        [self.min[0] + column as f64 * self.step(0, self.columns), self.min[1] + row as f64 * self.step(1, self.rows)]
    }

    /// The circle of a round bed spans the mesh from edge to edge.
    fn in_circle(&self, [x, y]: [f64; 2]) -> bool {
        let radius = (self.max[0] - self.min[0]) / 2.0;
        let center = [(self.min[0] + self.max[0]) / 2.0, (self.min[1] + self.max[1]) / 2.0];
        (x - center[0]).hypot(y - center[1]) <= radius + TOLERANCE
    }

    fn span(&self, region: &Region) -> Option<MeshSpan> {
        let inside = |index: usize, axis: usize, count: usize| {
            let position = self.min[axis] + index as f64 * self.step(axis, count);
            position >= region.min[axis] - TOLERANCE && position <= region.max[axis] + TOLERANCE
        };
        let columns: Vec<usize> = (0..self.columns).filter(|&column| inside(column, 0, self.columns)).collect();
        let rows: Vec<usize> = (0..self.rows).filter(|&row| inside(row, 1, self.rows)).collect();
        Some(MeshSpan { column: *columns.first()?, row: *rows.first()?, columns: columns.len(), rows: rows.len() })
    }
}

struct Region {
    min: [f64; 2],
    max: [f64; 2],
}

impl Region {
    fn contains(&self, [x, y]: [f64; 2]) -> bool {
        (self.min[0] - TOLERANCE..=self.max[0] + TOLERANCE).contains(&x)
            && (self.min[1] - TOLERANCE..=self.max[1] + TOLERANCE).contains(&y)
    }
}

fn faulty_regions(settings: Option<&Value>) -> Vec<Region> {
    let Some(settings) = settings else { return Vec::new() };
    (1..=MAX_FAULTY_REGIONS)
        .map_while(|index| {
            let corner = |end: &str| pair(settings.get(format!("faulty_region_{index}_{end}")));
            let [first, second] = [corner("min")?, corner("max")?];
            // Klipper sorts the corners itself.
            Some(Region {
                min: [first[0].min(second[0]), first[1].min(second[1])],
                max: [first[0].max(second[0]), first[1].max(second[1])],
            })
        })
        .collect()
}

/// `[x, y]` as a list, or as the text `"x, y"` of the configuration.
fn pair(value: Option<&Value>) -> Option<[f64; 2]> {
    let numbers: Vec<f64> = match value? {
        Value::Array(items) => items.iter().filter_map(Value::as_f64).collect(),
        Value::String(text) => text.split(',').filter_map(|part| part.trim().parse().ok()).collect(),
        _ => return None,
    };
    match numbers[..] {
        [x, y] => Some([x, y]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn status(matrix: Value) -> Value {
        json!({ "mesh_min": [30.0, 30.0], "mesh_max": [270.0, 270.0], "probed_matrix": matrix })
    }

    #[test]
    fn reads_the_matrix_with_row_zero_at_the_front() {
        let mesh = read(&status(json!([[-0.1, 0.0, 0.1], [0.0, 0.05, 0.2]])), None).unwrap();
        assert_eq!(mesh.columns(), 3);
        assert_eq!(mesh.rows[1][2], MeshPoint::Measured(0.2));
        assert_eq!(mesh.lowest(), Some(-0.1));
        assert_eq!(mesh.highest(), Some(0.2));
        assert!((mesh.range().unwrap() - 0.3).abs() < 1e-9);
    }

    #[test]
    fn no_mesh_without_a_matrix() {
        // Klipper reports `[[]]` when no mesh is active.
        assert_eq!(read(&status(json!([[]])), None), None);
        assert_eq!(read(&status(json!([[0.1, 0.2], [0.3]])), None), None);
    }

    #[test]
    fn corners_of_a_round_bed_are_copies() {
        let mut object = status(json!([[0.1, 0.1, 0.1], [0.2, 0.0, 0.3], [0.1, 0.1, 0.1]]));
        object["mesh_min"] = json!([-100.0, -100.0]);
        object["mesh_max"] = json!([100.0, 100.0]);
        let mesh = read(&object, Some(&json!({ "mesh_radius": 100.0 }))).unwrap();
        assert_eq!(mesh.rows[0][0], MeshPoint::Filled);
        assert_eq!(mesh.rows[0][1], MeshPoint::Measured(0.1));
        assert_eq!(mesh.rows[1][0], MeshPoint::Measured(0.2));
        assert_eq!(mesh.highest(), Some(0.3));
    }

    #[test]
    fn points_in_a_faulty_region_are_averaged() {
        let matrix = json!([[0.0, 0.0, 0.0], [0.0, 0.1, 0.0], [0.0, 0.0, 0.0]]);
        let settings = json!({ "faulty_region_1_min": [160.0, 140.0], "faulty_region_1_max": "140, 160" });
        let mesh = read(&status(matrix), Some(&settings)).unwrap();
        assert_eq!(mesh.rows[1][1], MeshPoint::Averaged(0.1));
        assert_eq!(mesh.averaged(), 1);
        assert_eq!(mesh.faulty, [MeshSpan { column: 1, row: 1, columns: 1, rows: 1 }]);
    }

    #[test]
    fn a_region_between_points_replaces_none() {
        let settings = json!({ "faulty_region_1_min": [40.0, 40.0], "faulty_region_1_max": [60.0, 60.0] });
        let mesh = read(&status(json!([[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]])), Some(&settings)).unwrap();
        assert_eq!(mesh.averaged(), 0);
        assert!(mesh.faulty.is_empty());
    }

    #[test]
    fn large_meshes_are_read_whole() {
        let row: Vec<f64> = (0..300).map(|index| index as f64 / 1000.0).collect();
        let mesh = read(&status(json!(vec![row; 300])), None).unwrap();
        assert_eq!((mesh.columns(), mesh.rows.len()), (300, 300));
        assert_eq!(mesh.highest(), Some(0.299));
    }
}
