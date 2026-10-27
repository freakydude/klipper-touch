//! `server.info`: Klippy's state as Moonraker sees it and the loaded Moonraker components.

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ServerInfo {
    pub klippy_connected: bool,
    pub klippy_state: String,
    pub components: Vec<String>,
    pub moonraker_version: String,
}

pub fn parse_server_info(result: &Value) -> ServerInfo {
    let text = |field: &str| result.get(field).and_then(Value::as_str).unwrap_or_default().to_owned();
    ServerInfo {
        klippy_connected: result.get("klippy_connected").and_then(Value::as_bool).unwrap_or_default(),
        klippy_state: text("klippy_state"),
        components: result
            .get("components")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_str).map(str::to_owned).collect())
            .unwrap_or_default(),
        moonraker_version: text("moonraker_version"),
    }
}

/// Loaded Moonraker components that enable their own operator functions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerComponents {
    pub power: bool,
    pub update_manager: bool,
    pub history: bool,
    /// Moonraker's Spoolman component is loaded, which means that Spoolman is configured.
    pub spoolman: bool,
}

impl ServerComponents {
    pub fn from_names<'a>(components: impl IntoIterator<Item = &'a str>) -> Self {
        let mut found = Self::default();
        for component in components {
            match component {
                "power" => found.power = true,
                "update_manager" => found.update_manager = true,
                "history" => found.history = true,
                "spoolman" => found.spoolman = true,
                _ => {}
            }
        }
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_server_info() {
        let info = parse_server_info(&json!({
            "klippy_connected": true, "klippy_state": "ready", "components": ["power"], "moonraker_version": "v0.9"
        }));

        assert!(info.klippy_connected);
        assert_eq!(info.components, ["power"]);
    }

    #[test]
    fn reads_moonraker_components() {
        let components = ServerComponents::from_names(["file_manager", "power", "update_manager", "machine"]);

        assert_eq!(components, ServerComponents { power: true, update_manager: true, history: false, spoolman: false });
        assert!(ServerComponents::from_names(["spoolman"]).spoolman);
    }
}
