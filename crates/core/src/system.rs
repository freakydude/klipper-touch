//! Host-Funktionen über Moonraker: Schaltgeräte, Updates, Konsole und Serverinfo.

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct PowerDevice {
    pub device: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub locked_while_printing: bool,
}

impl PowerDevice {
    pub fn is_on(&self) -> bool {
        self.status == "on"
    }
}

pub fn parse_power_devices(result: &Value) -> Vec<PowerDevice> {
    result.get("devices").cloned().and_then(|devices| serde_json::from_value(devices).ok()).unwrap_or_default()
}

/// Stand eines Software-Pakets im Update-Manager.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpdateInfo {
    pub name: String,
    pub version: String,
    pub remote_version: String,
    /// Anzahl Systempakete bzw. 0/1 für Software mit neuer Version.
    pub pending: u32,
    pub valid: bool,
}

impl UpdateInfo {
    pub fn has_update(&self) -> bool {
        self.pending > 0
    }
}

pub fn parse_update_status(result: &Value) -> Vec<UpdateInfo> {
    let Some(version_info) = result.get("version_info").and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut updates: Vec<UpdateInfo> = version_info.iter().map(|(name, info)| update_info(name, info)).collect();
    updates.sort_by(|left, right| (left.name != "system", &left.name).cmp(&(right.name != "system", &right.name)));
    updates
}

fn update_info(name: &str, info: &Value) -> UpdateInfo {
    let text = |field: &str| info.get(field).and_then(Value::as_str).unwrap_or_default().to_owned();
    if name == "system" {
        let count = info.get("package_count").and_then(Value::as_u64).unwrap_or_default();
        return UpdateInfo {
            name: name.to_owned(),
            version: String::new(),
            remote_version: String::new(),
            pending: u32::try_from(count).unwrap_or(u32::MAX),
            valid: true,
        };
    }
    let version = text("version");
    let remote_version = text("remote_version");
    let commits_behind = info.get("commits_behind_count").and_then(Value::as_u64).unwrap_or_default();
    let newer =
        (!remote_version.is_empty() && remote_version != "?" && remote_version != version) || commits_behind > 0;
    UpdateInfo {
        name: name.to_owned(),
        version,
        remote_version,
        pending: u32::from(newer),
        valid: info.get("is_valid").and_then(Value::as_bool).unwrap_or(true),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleKind {
    Command,
    Response,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsoleLine {
    pub kind: ConsoleKind,
    pub message: String,
    /// Unix-Zeit in Sekunden.
    pub time: f64,
}

pub fn parse_gcode_store(result: &Value) -> Vec<ConsoleLine> {
    result
        .get("gcode_store")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .map(|entry| ConsoleLine {
                    kind: if entry.get("type").and_then(Value::as_str) == Some("command") {
                        ConsoleKind::Command
                    } else {
                        ConsoleKind::Response
                    },
                    message: entry.get("message").and_then(Value::as_str).unwrap_or_default().to_owned(),
                    time: entry.get("time").and_then(Value::as_f64).unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_power_devices() {
        let devices = parse_power_devices(&json!({ "devices": [
            { "device": "printer", "status": "on", "locked_while_printing": true, "type": "gpio" },
            { "device": "light", "status": "off", "locked_while_printing": false, "type": "gpio" }
        ]}));

        assert_eq!(devices.len(), 2);
        assert!(devices[0].is_on() && devices[0].locked_while_printing);
        assert!(!devices[1].is_on());
    }

    #[test]
    fn update_status_lists_system_first_and_detects_new_versions() {
        let updates = parse_update_status(&json!({ "version_info": {
            "klipper": { "version": "v0.12.0-1", "remote_version": "v0.12.0-5", "is_valid": true },
            "moonraker": { "version": "v0.9.3", "remote_version": "v0.9.3" },
            "system": { "package_count": 4, "package_list": ["a", "b", "c", "d"] }
        }}));

        let names: Vec<&str> = updates.iter().map(|update| update.name.as_str()).collect();
        assert_eq!(names, ["system", "klipper", "moonraker"]);
        assert_eq!(updates[0].pending, 4);
        assert!(updates[1].has_update());
        assert!(!updates[2].has_update());
    }

    #[test]
    fn reads_gcode_store_and_server_info() {
        let lines = parse_gcode_store(&json!({ "gcode_store": [
            { "message": "G28", "time": 1.0, "type": "command" },
            { "message": "ok", "time": 2.0, "type": "response" }
        ]}));
        let info = parse_server_info(&json!({
            "klippy_connected": true, "klippy_state": "ready", "components": ["power"], "moonraker_version": "v0.9"
        }));

        assert_eq!(lines[0].kind, ConsoleKind::Command);
        assert_eq!(lines[1].message, "ok");
        assert!(info.klippy_connected);
        assert_eq!(info.components, ["power"]);
    }
}
