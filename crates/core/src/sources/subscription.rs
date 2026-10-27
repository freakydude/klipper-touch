//! The parameters of `printer.objects.subscribe`. Every source adds its objects; only those Klipper lists are kept.

use serde_json::{json, Map, Value};

pub(in crate::sources) struct Subscription<'a> {
    available: &'a [String],
    objects: Map<String, Value>,
}

impl<'a> Subscription<'a> {
    pub(in crate::sources) fn new(available: &'a [String]) -> Self {
        Self { available, objects: Map::new() }
    }

    /// Subscribes `name` with `fields`, or with all fields for `None`, if the printer has the object.
    pub(in crate::sources) fn add(&mut self, name: &str, fields: Option<&[&str]>) {
        if self.available.iter().any(|candidate| candidate == name) {
            self.objects.insert(name.to_owned(), fields.map_or(Value::Null, |fields| json!(fields)));
        }
    }

    pub(in crate::sources) fn into_parameters(self) -> Value {
        json!({ "objects": self.objects })
    }
}
