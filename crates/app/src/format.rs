//! Display text formats (duration, time of day, length, file names).

use chrono::{DateTime, Local, TimeZone};

/// `45s`, `12m 05s`, `2h 07m`.
pub fn duration(seconds: f64) -> String {
    let total = seconds.max(0.0).round() as u64;
    let (hours, minutes, secs) = (total / 3600, (total % 3600) / 60, total % 60);
    match (hours, minutes) {
        (0, 0) => format!("{secs}s"),
        (0, _) => format!("{minutes}m {secs:02}s"),
        _ => format!("{hours}h {minutes:02}m"),
    }
}

pub fn clock(time: DateTime<Local>) -> String {
    time.format("%H:%M").to_string()
}

/// Time of day at which a remaining duration ends; with date if it is no longer today.
pub fn finish_time(now: DateTime<Local>, remaining_seconds: f64) -> String {
    let finish = now + chrono::Duration::seconds(remaining_seconds.max(0.0).round() as i64);
    if finish.date_naive() == now.date_naive() {
        clock(finish)
    } else {
        finish.format("%d.%m. %H:%M").to_string()
    }
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

pub fn unix_clock(unix_seconds: f64) -> String {
    Local
        .timestamp_opt(unix_seconds as i64, 0)
        .single()
        .map_or_else(String::new, |time| time.format("%H:%M:%S").to_string())
}

pub fn unix_date(unix_seconds: f64) -> String {
    Local
        .timestamp_opt(unix_seconds as i64, 0)
        .single()
        .map_or_else(String::new, |time| time.format("%d.%m.%Y %H:%M").to_string())
}

/// Filament length in meters from millimeters.
pub fn filament_length(millimeters: f64) -> String {
    format!("{:.2} m", millimeters.max(0.0) / 1000.0)
}

/// Filament weight in grams; one decimal below 10 g.
pub fn filament_weight(grams: f64) -> String {
    if grams < 10.0 {
        format!("{:.1} g", grams.max(0.0))
    } else {
        format!("{grams:.0} g")
    }
}

pub fn file_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Display name of a G-code file: without folder and extension.
pub fn job_name(path: &str) -> String {
    let file = path.rsplit('/').next().unwrap_or(path);
    let lower = file.to_ascii_lowercase();
    [".gcode", ".gco", ".g"]
        .iter()
        .find(|extension| lower.ends_with(*extension))
        .map_or(file, |extension| &file[..file.len() - extension.len()])
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_are_compact() {
        assert_eq!(duration(45.4), "45s");
        assert_eq!(duration(725.0), "12m 05s");
        assert_eq!(duration(7_620.0), "2h 07m");
        assert_eq!(duration(-3.0), "0s");
    }

    #[test]
    fn finish_time_adds_date_only_on_other_days() {
        let now = Local.with_ymd_and_hms(2026, 10, 21, 10, 0, 0).unwrap();

        assert_eq!(finish_time(now, 5_400.0), "11:30");
        assert_eq!(finish_time(now, 20.0 * 3600.0), "22.10. 06:00");
    }

    #[test]
    fn lengths_sizes_and_names() {
        assert_eq!(filament_length(1234.0), "1.23 m");
        assert_eq!(filament_weight(4.56), "4.6 g");
        assert_eq!(filament_weight(23.4), "23 g");
        assert_eq!(file_size(512), "512 B");
        assert_eq!(file_size(1_572_864), "1.5 MB");
        assert_eq!(job_name("parts/Voron Cube.gcode"), "Voron Cube");
        assert_eq!(job_name("plain"), "plain");
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
}
