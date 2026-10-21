//! Rohzustand der abonnierten Klipper-Objekte, fortgeschrieben aus `notify_status_update`.

use serde_json::{Map, Value};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ObjectStore {
    objects: Map<String, Value>,
}

impl ObjectStore {
    /// Ersetzt den gesamten Stand, z. B. mit der Antwort auf `printer.objects.subscribe`.
    pub fn replace(&mut self, status: &Value) {
        self.objects = status.as_object().cloned().unwrap_or_default();
    }

    /// Übernimmt geänderte Felder; nicht genannte Felder bleiben erhalten.
    pub fn merge(&mut self, changes: &Value) {
        let Some(changes) = changes.as_object() else { return };
        for (name, change) in changes {
            match self.objects.get_mut(name) {
                Some(current) => merge_value(current, change),
                None => {
                    self.objects.insert(name.clone(), change.clone());
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn object(&self, name: &str) -> Option<&Value> {
        self.objects.get(name)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.objects.keys().map(String::as_str)
    }
}

fn merge_value(current: &mut Value, change: &Value) {
    match (current, change) {
        (Value::Object(current), Value::Object(change)) => {
            for (key, value) in change {
                match current.get_mut(key) {
                    Some(existing) => merge_value(existing, value),
                    None => {
                        current.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (current, change) => *current = change.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_keeps_unchanged_fields_and_nested_values() {
        let mut store = ObjectStore::default();
        store.replace(&json!({
            "extruder": { "temperature": 20.0, "target": 0.0 },
            "print_stats": { "info": { "current_layer": 1, "total_layer": 10 } }
        }));

        store.merge(&json!({
            "extruder": { "temperature": 25.5 },
            "print_stats": { "info": { "current_layer": 2 } },
            "fan": { "speed": 0.5 }
        }));

        assert_eq!(store.object("extruder"), Some(&json!({ "temperature": 25.5, "target": 0.0 })));
        assert_eq!(store.object("print_stats"), Some(&json!({ "info": { "current_layer": 2, "total_layer": 10 } })));
        assert_eq!(store.object("fan"), Some(&json!({ "speed": 0.5 })));
    }

    #[test]
    fn arrays_are_replaced_as_a_whole() {
        let mut store = ObjectStore::default();
        store.replace(&json!({ "toolhead": { "position": [1.0, 2.0, 3.0, 4.0] } }));

        store.merge(&json!({ "toolhead": { "position": [5.0, 6.0, 7.0, 8.0] } }));

        assert_eq!(store.object("toolhead"), Some(&json!({ "position": [5.0, 6.0, 7.0, 8.0] })));
    }
}
