//! The temperature chart: lines as SVG path commands in a 1000 x 1000 view box, and the scale around them. Free of
//! Slint types and therefore testable.

use freakymate_core::history::HISTORY_LENGTH;

const VIEW_BOX: f64 = 1000.0;
/// Points per line at most; more than the chart has pixels would only cost time on the Pi.
const MAX_POINTS: usize = 400;
/// The scale spans at least this many degrees and is rounded to it.
const SCALE_STEP: f64 = 10.0;
/// Heater power takes the lowest quarter of the chart.
const POWER_HEIGHT: f64 = 0.25;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartScale {
    pub low: f64,
    pub high: f64,
}

/// From the lowest to the highest value of both lines, rounded outwards to 10 °C.
pub fn scale(temperatures: &[f64], targets: &[f64]) -> ChartScale {
    let values = temperatures.iter().chain(targets.iter().filter(|target| **target > 0.0)).copied();
    let (low, high) =
        values.fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| (low.min(value), high.max(value)));
    if !low.is_finite() {
        return ChartScale { low: 0.0, high: 100.0 };
    }
    let low = (low / SCALE_STEP).floor() * SCALE_STEP;
    let high = ((high / SCALE_STEP).ceil() * SCALE_STEP).max(low + SCALE_STEP);
    ChartScale { low, high }
}

/// A line through the values, the newest at the right edge; the time axis is that of the full history, so a short
/// history fills only the right part.
pub fn line(values: &[f64], scale: ChartScale) -> String {
    let span = (scale.high - scale.low).max(f64::EPSILON);
    path(values, |value| VIEW_BOX - ((value - scale.low) / span).clamp(0.0, 1.0) * VIEW_BOX)
}

/// Heater power from 0 to 1 along the bottom of the chart.
pub fn power_line(powers: &[f64]) -> String {
    path(powers, |power| VIEW_BOX - power.clamp(0.0, 1.0) * POWER_HEIGHT * VIEW_BOX)
}

fn path(values: &[f64], to_y: impl Fn(f64) -> f64) -> String {
    if values.len() < 2 {
        return String::new();
    }
    let stride = values.len().div_ceil(MAX_POINTS);
    let last = values.len() - 1;
    let x_of = |index: usize| VIEW_BOX - (last - index) as f64 / (HISTORY_LENGTH - 1) as f64 * VIEW_BOX;
    let mut indices: Vec<usize> = (0..values.len()).step_by(stride).collect();
    if indices.last() != Some(&last) {
        indices.push(last);
    }
    indices
        .iter()
        .enumerate()
        .map(|(position, &index)| {
            let command = if position == 0 { 'M' } else { 'L' };
            format!("{command} {:.1} {:.1}", x_of(index).max(0.0), to_y(values[index]))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// How far back the chart reaches, e.g. "−20 min".
pub fn span_text() -> String {
    format!("−{} min", HISTORY_LENGTH / 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scale_covers_both_lines_in_steps_of_ten() {
        assert_eq!(scale(&[21.5, 180.2], &[0.0, 200.0]), ChartScale { low: 20.0, high: 200.0 });
        assert_eq!(scale(&[45.0, 45.0], &[]), ChartScale { low: 40.0, high: 50.0 });
        assert_eq!(scale(&[], &[]), ChartScale { low: 0.0, high: 100.0 });
    }

    #[test]
    fn a_line_ends_at_the_right_edge_and_scales_its_values() {
        let path = line(&[20.0, 120.0], ChartScale { low: 20.0, high: 220.0 });
        let x_before = VIEW_BOX - VIEW_BOX / (HISTORY_LENGTH - 1) as f64;

        assert_eq!(path, format!("M {x_before:.1} 1000.0 L 1000.0 500.0"));
        assert_eq!(line(&[20.0], ChartScale { low: 0.0, high: 100.0 }), "", "one point is no line");
    }

    #[test]
    fn a_long_history_is_thinned_out_but_keeps_its_newest_point() {
        let values: Vec<f64> = (0..HISTORY_LENGTH).map(|second| second as f64).collect();
        let path = line(&values, ChartScale { low: 0.0, high: HISTORY_LENGTH as f64 });

        assert!(path.split(" ").filter(|word| *word == "L").count() <= MAX_POINTS);
        assert!(path.starts_with("M 0.0 1000.0"));
        assert!(path.ends_with("L 1000.0 0.8"));
    }

    #[test]
    fn power_stays_in_the_bottom_quarter() {
        assert_eq!(power_line(&[0.0, 1.0]).rsplit(' ').next(), Some("750.0"));
    }
}
