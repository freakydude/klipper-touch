//! Reading single fields of Klipper's objects; a missing or mistyped field gives the default.

use serde_json::Value;

pub(super) fn text<'a>(object: &'a Value, field: &str) -> &'a str {
    object.get(field).and_then(Value::as_str).unwrap_or_default()
}

pub(super) fn number(object: &Value, field: &str) -> f64 {
    number_or(object, field, 0.0)
}

pub(super) fn number_or(object: &Value, field: &str, default: f64) -> f64 {
    object.get(field).and_then(Value::as_f64).unwrap_or(default)
}

pub(super) fn flag(object: &Value, field: &str) -> bool {
    object.get(field).and_then(Value::as_bool).unwrap_or_default()
}

pub(super) fn count(object: &Value, field: &str) -> Option<u32> {
    object.get(field).and_then(Value::as_u64).and_then(|value| u32::try_from(value).ok())
}

pub(super) fn vector(object: &Value, field: &str) -> [f64; 4] {
    let mut values = [0.0; 4];
    if let Some(items) = object.get(field).and_then(Value::as_array) {
        for (slot, item) in values.iter_mut().zip(items) {
            *slot = item.as_f64().unwrap_or_default();
        }
    }
    values
}
