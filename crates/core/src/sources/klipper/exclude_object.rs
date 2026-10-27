//! The objects of the running print that `EXCLUDE_OBJECT` can skip (`[exclude_object]`), and where they lie on the
//! bed when the slicer defined it (`EXCLUDE_OBJECT_DEFINE` with `CENTER` and `POLYGON`).

use super::motion::BedPoint;
use serde_json::Value;

/// An object of the running print that `EXCLUDE_OBJECT` can skip.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrintObject {
    pub name: String,
    pub excluded: bool,
    /// Klipper prints it right now (`current_object`).
    pub printing: bool,
    /// Outline on the bed (`POLYGON`); empty if the slicer did not define one.
    pub outline: Vec<BedPoint>,
    /// `CENTER`, if the slicer defined it.
    pub center: Option<BedPoint>,
}

impl PrintObject {
    /// Whether the slicer told where the object lies.
    pub fn is_located(&self) -> bool {
        self.has_outline() || self.center.is_some()
    }

    pub fn has_outline(&self) -> bool {
        self.outline.len() >= 3
    }

    /// How far the point is from the object in mm: 0 inside the outline; without an outline from the centre.
    fn distance(&self, point: BedPoint) -> Option<f64> {
        if !self.has_outline() {
            return self.center.map(|center| center.distance(point));
        }
        if contains(&self.outline, point) {
            return Some(0.0);
        }
        edges(&self.outline).map(|(from, to)| segment_distance(point, from, to)).min_by(f64::total_cmp)
    }
}

/// The object at a point of the bed: the one whose outline holds the point or comes closest within `reach` mm, so a
/// finger also hits parts that are only a few pixels on a map. Of overlapping objects the last one wins (drawn on
/// top).
pub fn object_at(objects: &[PrintObject], point: BedPoint, reach: f64) -> Option<usize> {
    objects
        .iter()
        .enumerate()
        .filter_map(|(index, object)| object.distance(point).map(|distance| (index, distance)))
        .filter(|(_, distance)| *distance <= reach)
        .min_by(|(index, distance), (other_index, other_distance)| {
            distance.total_cmp(other_distance).then(other_index.cmp(index))
        })
        .map(|(index, _)| index)
}

fn edges(outline: &[BedPoint]) -> impl Iterator<Item = (BedPoint, BedPoint)> + '_ {
    outline.iter().copied().zip(outline.iter().copied().cycle().skip(1))
}

/// Even-odd rule: a ray to the right crosses the outline an odd number of times.
fn contains(outline: &[BedPoint], point: BedPoint) -> bool {
    edges(outline)
        .filter(|(from, to)| (from.y > point.y) != (to.y > point.y))
        .filter(|(from, to)| point.x < from.x + (point.y - from.y) / (to.y - from.y) * (to.x - from.x))
        .count()
        % 2
        == 1
}

fn segment_distance(point: BedPoint, from: BedPoint, to: BedPoint) -> f64 {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let length_squared = dx * dx + dy * dy;
    if length_squared == 0.0 {
        return point.distance(from);
    }
    let share = (((point.x - from.x) * dx + (point.y - from.y) * dy) / length_squared).clamp(0.0, 1.0);
    point.distance(BedPoint { x: from.x + share * dx, y: from.y + share * dy })
}

/// `objects` lists `{ "name", "center", "polygon" }` entries (the last two only if the slicer defined them),
/// `excluded_objects` plain names, `current_object` a name or null.
pub(super) fn read_print_objects(exclude_object: &Value) -> Vec<PrintObject> {
    let excluded: Vec<&str> = exclude_object
        .get("excluded_objects")
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let current = exclude_object.get("current_object").and_then(Value::as_str);
    let objects = exclude_object.get("objects").and_then(Value::as_array);
    objects
        .into_iter()
        .flatten()
        .filter_map(|object| {
            let name = object.get("name").and_then(Value::as_str)?;
            Some(PrintObject {
                name: name.to_owned(),
                excluded: excluded.contains(&name),
                printing: current == Some(name),
                outline: object
                    .get("polygon")
                    .and_then(Value::as_array)
                    .map(|points| points.iter().filter_map(point).collect())
                    .unwrap_or_default(),
                center: object.get("center").and_then(point),
            })
        })
        .collect()
}

fn point(value: &Value) -> Option<BedPoint> {
    match value.as_array()?.as_slice() {
        [x, y, ..] => Some(BedPoint { x: x.as_f64()?, y: y.as_f64()? }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn square(name: &str, left: f64, bottom: f64, size: f64) -> PrintObject {
        let corners = [(left, bottom), (left + size, bottom), (left + size, bottom + size), (left, bottom + size)];
        PrintObject {
            name: name.into(),
            outline: corners.into_iter().map(|(x, y)| BedPoint { x, y }).collect(),
            center: Some(BedPoint { x: left + size / 2.0, y: bottom + size / 2.0 }),
            ..PrintObject::default()
        }
    }

    #[test]
    fn marks_excluded_print_objects() {
        let exclude_object = json!({
            "objects": [{ "name": "Benchy_0" }, { "name": "Benchy_1" }],
            "excluded_objects": ["Benchy_1"],
        });
        assert_eq!(
            read_print_objects(&exclude_object),
            vec![
                PrintObject { name: "Benchy_0".into(), ..PrintObject::default() },
                PrintObject { name: "Benchy_1".into(), excluded: true, ..PrintObject::default() },
            ]
        );
        assert!(read_print_objects(&Value::Null).is_empty());
    }

    #[test]
    fn reads_outline_centre_and_the_object_being_printed() {
        let exclude_object = json!({
            "objects": [
                { "name": "CUBE", "center": [15.0, 15.0], "polygon": [[10.0, 10.0], [20.0, 10.0], [20.0, 20.0], [10.0, 20.0]] },
                { "name": "PIN", "center": [50, 60] },
            ],
            "excluded_objects": [],
            "current_object": "PIN",
        });
        let objects = read_print_objects(&exclude_object);
        assert_eq!(objects[0], square("CUBE", 10.0, 10.0, 10.0));
        assert!(!objects[0].printing);
        assert_eq!(objects[1].center, Some(BedPoint { x: 50.0, y: 60.0 }), "integers count as numbers");
        assert!(objects[1].printing && objects[1].is_located() && !objects[1].has_outline());
    }

    #[test]
    fn a_point_hits_the_object_whose_outline_holds_it() {
        let objects = [square("A", 0.0, 0.0, 10.0), square("B", 50.0, 0.0, 10.0)];
        assert_eq!(object_at(&objects, BedPoint { x: 55.0, y: 5.0 }, 0.0), Some(1));
        assert_eq!(object_at(&objects, BedPoint { x: 30.0, y: 5.0 }, 5.0), None, "too far from both");
    }

    #[test]
    fn a_point_near_a_small_object_hits_it_within_reach() {
        let objects = [square("SMALL", 100.0, 100.0, 1.0)];
        assert_eq!(object_at(&objects, BedPoint { x: 104.0, y: 100.5 }, 5.0), Some(0));
        assert_eq!(object_at(&objects, BedPoint { x: 104.0, y: 100.5 }, 2.0), None);
    }

    #[test]
    fn the_nearer_object_wins_and_on_overlap_the_one_on_top() {
        let objects = [square("LOW", 0.0, 0.0, 20.0), square("TOP", 10.0, 10.0, 20.0), square("FAR", 40.0, 0.0, 5.0)];
        assert_eq!(object_at(&objects, BedPoint { x: 15.0, y: 15.0 }, 5.0), Some(1), "inside both");
        assert_eq!(object_at(&objects, BedPoint { x: 5.0, y: 5.0 }, 5.0), Some(0), "inside only the lower one");
        assert_eq!(object_at(&objects, BedPoint { x: 38.0, y: 2.0 }, 10.0), Some(2), "closer to FAR than to TOP");
    }

    #[test]
    fn an_object_without_outline_is_hit_near_its_centre() {
        let pin =
            PrintObject { name: "PIN".into(), center: Some(BedPoint { x: 50.0, y: 50.0 }), ..PrintObject::default() };
        let unknown = PrintObject { name: "UNKNOWN".into(), ..PrintObject::default() };
        let objects = [unknown, pin];
        assert_eq!(object_at(&objects, BedPoint { x: 53.0, y: 50.0 }, 4.0), Some(1));
        assert_eq!(object_at(&objects, BedPoint { x: 0.0, y: 0.0 }, 1000.0), Some(1), "an unknown place is never hit");
    }
}
