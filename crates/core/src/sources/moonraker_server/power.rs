//! Moonraker's `power` component: switchable devices such as the printer's mains relay.

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
}
