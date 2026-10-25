//! Parameters of G-code macros, read from their template in `configfile.settings`.
//!
//! Klipper does not declare macro parameters. A template uses `params.NAME`, optionally with
//! `|default(VALUE)`, and `'NAME' in params` to test for a parameter. Moonraker has no API for
//! this, so the template is scanned; Mainsail and KlipperScreen do the same. Which parameters a
//! macro really needs cannot be read from the template, so none is treated as required.

use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroParam {
    pub name: String,
    /// Literal from `|default(...)`; the UI shows it as a placeholder, Klipper uses it when the field stays empty.
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MacroDetails {
    pub params: Vec<MacroParam>,
    /// Overrides a built-in command (`rename_existing`), so it is not offered as a macro.
    pub renamed: bool,
}

/// Details of every `gcode_macro` section, keyed by the upper-case macro name.
pub fn details_from_settings(settings: Option<&Value>) -> BTreeMap<String, MacroDetails> {
    let Some(settings) = settings.and_then(Value::as_object) else { return BTreeMap::new() };
    settings
        .iter()
        .filter_map(|(section, value)| {
            let name = section.strip_prefix("gcode_macro ")?.trim().to_ascii_uppercase();
            let template = value.get("gcode").and_then(Value::as_str).unwrap_or_default();
            let details =
                MacroDetails { params: parse_params(template), renamed: value.get("rename_existing").is_some() };
            Some((name, details))
        })
        .collect()
}

pub fn parse_params(template: &str) -> Vec<MacroParam> {
    let mut params = Vec::new();
    let mut rest = template;
    while let Some(position) = rest.find("params.") {
        let preceded_by_identifier = rest[..position].chars().next_back().is_some_and(is_identifier_char);
        let after = &rest[position + "params.".len()..];
        let name_length = after.chars().take_while(|c| is_identifier_char(*c)).count();
        rest = &after[name_length..];
        if preceded_by_identifier || name_length == 0 {
            continue;
        }
        let default = default_argument(rest).and_then(literal_default);
        merge(&mut params, MacroParam { name: after[..name_length].to_owned(), default });
    }
    for name in tested_names(template) {
        merge(&mut params, MacroParam { name, default: None });
    }
    params
}

/// The argument of a `| default(...)` filter that directly follows a parameter, spaces around the pipe allowed.
fn default_argument(after_name: &str) -> Option<&str> {
    let tail = after_name.trim_start().strip_prefix('|')?.trim_start().strip_prefix("default(")?;
    tail.split(')').next()
}

/// The command that runs a macro with arguments that have a value. Macros named like G- or M-codes take their
/// letters directly (`M106 S255`), all others take `NAME=VALUE`.
pub fn macro_script(name: &str, arguments: &[(&str, &str)]) -> String {
    let gcode_style = is_gcode_name(name);
    let mut script = name.to_owned();
    for (key, value) in arguments {
        if gcode_style {
            script.push_str(&format!(" {key}{value}"));
        } else {
            script.push_str(&format!(" {key}={value}"));
        }
    }
    script
}

/// `G` or `M` followed by one to three digits.
fn is_gcode_name(name: &str) -> bool {
    let Some(digits) = name.strip_prefix(['G', 'M']) else { return false };
    (1..=3).contains(&digits.len()) && digits.bytes().all(|byte| byte.is_ascii_digit())
}

/// The first argument of a default, if it is a plain literal: a number, a quoted text or a boolean.
/// Expressions such as `pause_at_layer.layer` are only known to Klipper at runtime, so they get no placeholder.
fn literal_default(argument: &str) -> Option<String> {
    let first = argument.split(',').next()?.trim();
    let quoted = first.len() >= 2
        && ((first.starts_with('\'') && first.ends_with('\'')) || (first.starts_with('"') && first.ends_with('"')));
    let literal = quoted || first.parse::<f64>().is_ok() || matches!(first, "True" | "False" | "true" | "false");
    literal.then(|| first.trim_matches(['\'', '"']).to_owned())
}

/// Names tested as `'NAME' in params` or `'NAME' not in params`.
fn tested_names(template: &str) -> Vec<String> {
    template
        .match_indices(" in params")
        .filter_map(|(position, _)| {
            let before = template[..position].trim_end();
            let before = before.strip_suffix("not").map_or(before, str::trim_end);
            let quote = before.chars().last().filter(|c| *c == '\'' || *c == '"')?;
            let inner = &before[..before.len() - quote.len_utf8()];
            let name = &inner[inner.rfind(quote)? + 1..];
            (!name.is_empty() && name.chars().all(is_identifier_char)).then(|| name.to_owned())
        })
        .collect()
}

/// A parameter named more than once keeps its first spelling and takes any default found on the way.
fn merge(params: &mut Vec<MacroParam>, found: MacroParam) {
    match params.iter_mut().find(|param| param.name == found.name) {
        Some(existing) => existing.default = existing.default.take().or(found.default),
        None => params.push(found),
    }
}

fn is_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn param(name: &str, default: Option<&str>) -> MacroParam {
        MacroParam { name: name.to_owned(), default: default.map(str::to_owned) }
    }

    #[test]
    fn reads_names_and_literal_defaults() {
        let params = parse_params("{% set Z = params.Z|default(10) %}\nG1 Z{Z} F{params.SPEED|default('1200')}");

        assert_eq!(params, [param("Z", Some("10")), param("SPEED", Some("1200"))]);
    }

    #[test]
    fn default_filter_with_spaces_around_the_pipe() {
        let template = "{% set power_off_enabled = params.ENABLED | default(1) %}\n{% if power_off_enabled == 1 %}";

        assert_eq!(parse_params(template), [param("ENABLED", Some("1"))]);
    }

    #[test]
    fn runtime_defaults_get_no_placeholder() {
        let template = "{% set LAYER = params.LAYER|default(pause_at_layer.layer)|int %}\n\
                        {% set MACRO = params.MACRO|default(pause_at_layer.call, True) %}";

        assert_eq!(parse_params(template), [param("LAYER", None), param("MACRO", None)]);
    }

    #[test]
    fn boolean_default_keeps_its_first_argument() {
        assert_eq!(parse_params("{params.SAVE|default(True, true)}"), [param("SAVE", Some("True"))]);
    }

    #[test]
    fn parameter_without_default_has_no_placeholder() {
        assert_eq!(parse_params("G1 X{params.X|float}"), [param("X", None)]);
    }

    #[test]
    fn tested_names_are_parameters_too() {
        let params =
            parse_params("{% if 'PURGE' in params %}G1 E5{% endif %}{% if 'SKIP' not in params %}G28{% endif %}");

        assert_eq!(params, [param("PURGE", None), param("SKIP", None)]);
    }

    #[test]
    fn repeated_mentions_merge_and_keep_a_default() {
        let params = parse_params("{params.TEMP} {params.TEMP|default(230)} {% if 'TEMP' in params %}");

        assert_eq!(params, [param("TEMP", Some("230"))]);
    }

    #[test]
    fn shaper_style_template_lists_all_names() {
        let template = "{% set axis = params.AXIS|default('all') %}\n{% set scv = params.SCV %}\n\
                        {% set travel_speed = params.TRAVEL_SPEED|default(120) %}";

        assert_eq!(
            parse_params(template),
            [param("AXIS", Some("all")), param("SCV", None), param("TRAVEL_SPEED", Some("120"))]
        );
    }

    #[test]
    fn script_uses_name_equals_value_for_normal_macros() {
        assert_eq!(macro_script("PARK", &[("Z", "20"), ("SPEED", "80")]), "PARK Z=20 SPEED=80");
        assert_eq!(macro_script("PARK", &[]), "PARK");
    }

    #[test]
    fn script_uses_gcode_letters_for_g_and_m_macros() {
        assert_eq!(macro_script("M106", &[("S", "255")]), "M106 S255");
        assert_eq!(macro_script("M900", &[("K", "0.03")]), "M900 K0.03");
    }

    #[test]
    fn only_g_and_m_followed_by_digits_count_as_gcode() {
        assert!(is_gcode_name("G28"));
        assert!(!is_gcode_name("M"));
        assert!(!is_gcode_name("M1234"));
        assert!(!is_gcode_name("MMU_HOME"));
    }

    #[test]
    fn ignores_names_inside_other_identifiers() {
        assert!(parse_params("G1 X{myparams.X}").is_empty());
    }

    #[test]
    fn details_read_renamed_macros_and_upper_case_names() {
        let settings = json!({
            "gcode_macro PARK": { "gcode": "G1 Z{params.Z|default(10)}" },
            "gcode_macro pause": { "rename_existing": "BASE_PAUSE", "gcode": "PAUSE_BASE" },
            "extruder": { "min_temp": 10.0 }
        });

        let details = details_from_settings(Some(&settings));

        assert_eq!(details.len(), 2);
        assert_eq!(details["PARK"].params, [param("Z", Some("10"))]);
        assert!(details["PAUSE"].renamed);
    }
}
