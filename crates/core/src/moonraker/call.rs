//! A request to Moonraker as the sources build it: method and parameters, sent as it is.

use serde_json::Value;

/// A concrete Moonraker call.
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub method: &'static str,
    pub params: Option<Value>,
}

impl Call {
    pub fn new(method: &'static str) -> Self {
        Self { method, params: None }
    }

    pub fn with(method: &'static str, params: Value) -> Self {
        Self { method, params: Some(params) }
    }
}
