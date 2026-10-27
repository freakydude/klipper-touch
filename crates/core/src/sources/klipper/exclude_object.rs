//! The objects of the running print that `EXCLUDE_OBJECT` can skip (`[exclude_object]`).

use serde_json::Value;

/// An object of the running print that `EXCLUDE_OBJECT` can skip.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PrintObject {
    pub name: String,
    pub excluded: bool,
}

/// `objects` lists `{ "name": ... }` entries, `excluded_objects` lists plain names.
pub(super) fn read_print_objects(exclude_object: &Value) -> Vec<PrintObject> {
    let excluded: Vec<&str> = exclude_object
        .get("excluded_objects")
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let objects = exclude_object.get("objects").and_then(Value::as_array);
    objects
        .into_iter()
        .flatten()
        .filter_map(|object| object.get("name").and_then(Value::as_str))
        .map(|name| PrintObject { name: name.to_owned(), excluded: excluded.contains(&name) })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn marks_excluded_print_objects() {
        let exclude_object = json!({
            "objects": [{ "name": "Benchy_0" }, { "name": "Benchy_1" }],
            "excluded_objects": ["Benchy_1"],
        });
        assert_eq!(
            read_print_objects(&exclude_object),
            vec![
                PrintObject { name: "Benchy_0".into(), excluded: false },
                PrintObject { name: "Benchy_1".into(), excluded: true },
            ]
        );
        assert!(read_print_objects(&Value::Null).is_empty());
    }
}
