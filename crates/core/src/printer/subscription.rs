//! Which objects and fields are subscribed, depending on the detected capabilities.

use super::capabilities::Capabilities;
use serde_json::{json, Map, Value};

const HEATER_FIELDS: &[&str] = &["temperature", "target", "power"];

/// Subscription parameters for `printer.objects.subscribe`; only available objects are included.
pub fn subscription(available: &[String], capabilities: &Capabilities) -> Value {
    let mut objects = Map::new();
    let mut add = |name: &str, fields: Option<&[&str]>| {
        if available.iter().any(|candidate| candidate == name) {
            objects.insert(name.to_owned(), fields.map_or(Value::Null, |fields| json!(fields)));
        }
    };

    add("webhooks", None);
    add("print_stats", None);
    add("display_status", Some(&["progress", "message"]));
    add("virtual_sdcard", Some(&["progress", "is_active"]));
    add("idle_timeout", Some(&["state"]));
    add(
        "toolhead",
        Some(&[
            "position",
            "homed_axes",
            "axis_minimum",
            "axis_maximum",
            "max_velocity",
            "max_accel",
            "square_corner_velocity",
            "minimum_cruise_ratio",
        ]),
    );
    add("motion_report", Some(&["live_position", "live_velocity", "live_extruder_velocity"]));
    add("gcode_move", Some(&["homing_origin", "speed_factor", "extrude_factor"]));
    add("extruder", Some(&["temperature", "target", "power", "pressure_advance", "smooth_time", "can_extrude"]));
    // `settings` holds the heater limits (min_temp/max_temp); sent once, it does not change at runtime.
    add("configfile", Some(&["save_config_pending", "settings"]));
    add("heater_bed", Some(HEATER_FIELDS));
    add("quad_gantry_level", Some(&["applied"]));
    add("z_tilt", Some(&["applied"]));
    add("firmware_retraction", Some(&["retract_length", "retract_speed", "unretract_extra_length", "unretract_speed"]));
    add("exclude_object", Some(&["objects", "excluded_objects"]));
    for heater in &capabilities.extra_heaters {
        add(heater, None);
    }
    for sensor in &capabilities.temperature_sensors {
        add(sensor, Some(&["temperature"]));
    }
    for fan in &capabilities.fans {
        add(fan, Some(&["speed", "rpm"]));
    }
    for led in &capabilities.leds {
        add(led, Some(&["color_data"]));
    }
    for effect in &capabilities.led_effects {
        add(effect, Some(&["enabled"]));
    }
    for sensor in &capabilities.filament_sensors {
        add(sensor, Some(&["enabled", "filament_detected"]));
    }
    // All fields: the filament view needs gates, sensors, buffer and FlowGuard, and Happy Hare adds fields over time.
    add("mmu", None);
    add("mmu_machine", None);

    json!({ "objects": objects })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscribes_only_available_objects() {
        let available: Vec<String> =
            ["webhooks", "extruder", "heater_generic chamber", "gcode_macro START"].map(String::from).to_vec();
        let capabilities = Capabilities::from_objects(available.iter().map(String::as_str));

        let parameters = subscription(&available, &capabilities);

        let objects = parameters["objects"].as_object().unwrap();
        let names: Vec<&str> = objects.keys().map(String::as_str).collect();
        assert_eq!(names, ["extruder", "heater_generic chamber", "webhooks"]);
        assert_eq!(objects["webhooks"], Value::Null);
        assert_eq!(objects["extruder"][0], "temperature");
    }
}
