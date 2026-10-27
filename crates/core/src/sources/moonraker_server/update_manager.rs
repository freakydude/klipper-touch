//! Moonraker's `update_manager` component: versions of the installed software and the system packages.

use serde_json::Value;

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}
