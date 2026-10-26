//! Host functions via Moonraker: power devices, updates, console and server info.

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

/// State of a software package in the update manager.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpdateInfo {
    pub name: String,
    pub version: String,
    pub remote_version: String,
    /// Number of system packages, or 0/1 for software with a newer version.
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
    /// Unix time in seconds.
    pub time: f64,
}

impl ConsoleLine {
    /// Klipper marks error responses with `!!`.
    pub fn is_error(&self) -> bool {
        self.message.starts_with("!!")
    }

    /// Klipper marks information responses with `//`.
    pub fn is_information(&self) -> bool {
        self.message.starts_with("//")
    }

    /// The text as shown. A response loses its marker (`!!` error, `//` information; the colour tells them apart)
    /// and the HTML of plugins; a command is shown as typed. Klipper puts the marker before every line of a message.
    pub fn text(&self) -> String {
        match self.kind {
            ConsoleKind::Command => self.message.clone(),
            ConsoleKind::Response => {
                let lines: Vec<&str> = self.message.split('\n').map(strip_response_marker).collect();
                strip_markup(&lines.join("\n"))
            }
        }
    }
}

fn strip_response_marker(line: &str) -> &str {
    line.strip_prefix("!!").or_else(|| line.strip_prefix("//")).map_or(line, str::trim_start)
}

/// Happy Hare and other plugins put HTML into their responses for Mainsail (`<span style=...>`, `&nbsp;`); the
/// touch UI shows the text only. A `<` that does not start a tag stays.
pub fn strip_markup(text: &str) -> String {
    let mut plain = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('<') {
        let (before, tag_and_after) = rest.split_at(start);
        plain.push_str(before);
        let starts_tag =
            tag_and_after[1..].chars().next().is_some_and(|next| next.is_ascii_alphabetic() || next == '/');
        match tag_and_after.find('>').filter(|_| starts_tag) {
            Some(end) => rest = &tag_and_after[end + 1..],
            None => {
                plain.push('<');
                rest = &tag_and_after[1..];
            }
        }
    }
    plain.push_str(rest);
    plain.replace("&nbsp;", " ").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&amp;", "&")
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

/// Network interface of the printer host as reported by Moonraker (read-only).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HostInterface {
    pub name: String,
    pub mac_address: String,
    /// IPv4 and global IPv6 addresses.
    pub addresses: Vec<String>,
}

impl HostInterface {
    /// Linux names wireless interfaces `wlan0`, `wlp2s0`, ...
    pub fn is_wireless(&self) -> bool {
        self.name.starts_with("wl")
    }
}

pub fn parse_host_network(result: &Value) -> Vec<HostInterface> {
    let Some(network) = result.pointer("/system_info/network").and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut interfaces: Vec<HostInterface> = network
        .iter()
        .map(|(name, interface)| HostInterface {
            name: name.clone(),
            mac_address: interface.get("mac_address").and_then(Value::as_str).unwrap_or_default().to_owned(),
            addresses: interface
                .get("ip_addresses")
                .and_then(Value::as_array)
                .map(|addresses| {
                    addresses
                        .iter()
                        .filter(|address| !address.get("is_link_local").and_then(Value::as_bool).unwrap_or(false))
                        .filter_map(|address| address.get("address").and_then(Value::as_str).map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect();
    interfaces.sort_by(|left, right| left.name.cmp(&right.name));
    interfaces
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

    #[test]
    fn reads_host_network_without_link_local_addresses() {
        let interfaces = parse_host_network(&json!({ "system_info": { "network": {
            "wlan0": { "mac_address": "aa:bb", "ip_addresses": [
                { "family": "ipv4", "address": "192.168.1.127", "is_link_local": false },
                { "family": "ipv6", "address": "fe80::1", "is_link_local": true }
            ]},
            "eth0": { "mac_address": "cc:dd", "ip_addresses": [] }
        }}}));

        assert_eq!(interfaces.len(), 2);
        assert_eq!(interfaces[0].name, "eth0");
        assert_eq!(interfaces[1].addresses, ["192.168.1.127"]);
        assert_eq!(interfaces[1].mac_address, "aa:bb");
    }

    fn response(message: &str) -> ConsoleLine {
        ConsoleLine { kind: ConsoleKind::Response, message: message.to_owned(), time: 0.0 }
    }

    #[test]
    fn console_text_loses_the_klipper_marker_and_plugin_markup() {
        assert_eq!(response("!! Lost connection").text(), "Lost connection");
        assert_eq!(response("// Klipper state: Ready").text(), "Klipper state: Ready");
        assert_eq!(response(r#"// <b><span style="color:#87CEEB">EMPTY</span></b>&nbsp;gate"#).text(), "EMPTY gate");
    }

    #[test]
    fn every_line_of_a_message_loses_its_marker() {
        let shutdown = response(
            "// Shutdown due to webhooks request\n// Once the underlying issue is corrected, use the\n\
             // \"FIRMWARE_RESTART\" command to reset the firmware, reload the\n// config, and restart the host software.\n\
             // Printer is shutdown",
        );

        assert_eq!(
            shutdown.text(),
            "Shutdown due to webhooks request\nOnce the underlying issue is corrected, use the\n\
             \"FIRMWARE_RESTART\" command to reset the firmware, reload the\nconfig, and restart the host software.\n\
             Printer is shutdown"
        );
    }

    #[test]
    fn plugin_markup_over_several_lines_loses_marker_and_tags() {
        let art = response("// <span style=\"color:#C0C0C0\">(\\_/)</span>\n// <span style=\"color:#C0C0C0\">(\")_(\")</span> <b>Happy Hare</b>");

        assert_eq!(art.text(), "(\\_/)\n(\")_(\") Happy Hare");
    }

    #[test]
    fn lines_without_a_marker_stay_as_they_are() {
        assert_eq!(response("// first\nplain line\n!! last").text(), "first\nplain line\nlast");
        assert_eq!(response("!! Lost connection\n!! see log").text(), "Lost connection\nsee log");
    }

    #[test]
    fn a_command_is_shown_as_typed() {
        let command = ConsoleLine { kind: ConsoleKind::Command, message: "// G28".to_owned(), time: 0.0 };

        assert_eq!(command.text(), "// G28");
    }

    #[test]
    fn only_bang_bang_responses_are_errors() {
        assert!(response("!! Timer too close").is_error());
        assert!(!response("// ok").is_error());
    }

    #[test]
    fn markup_of_plugin_responses_is_removed() {
        assert_eq!(
            strip_markup(r#"<span style="color:#C0C0C0">(\_/)</span> <b>Happy Hare</b> v4.1.0"#),
            r"(\_/) Happy Hare v4.1.0"
        );
        assert_eq!(strip_markup("Gate&nbsp;0 &amp; tool &lt;T0&gt;"), "Gate 0 & tool <T0>");
        assert_eq!(strip_markup("Move out of range: x < 0 and y <5"), "Move out of range: x < 0 and y <5");
        assert_eq!(strip_markup("half <span"), "half <span");
        assert_eq!(strip_markup(""), "");
    }

    #[test]
    fn wireless_interfaces_are_recognized_by_their_linux_name() {
        let interface = |name: &str| HostInterface { name: name.to_owned(), ..HostInterface::default() };

        assert!(interface("wlan0").is_wireless() && interface("wlp3s0").is_wireless());
        assert!(!interface("eth0").is_wireless() && !interface("enp1s0").is_wireless());
    }
}
