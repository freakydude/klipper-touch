//! The temperature history overlay: which heater or sensor it shows, its values from Moonraker's store plus the
//! live values since, and how they reach the UI.

use crate::{chart, AppWindow, Navigation, Printer};
use freakymate_core::history::TemperatureSeries;
use freakymate_core::printer::PrinterStatus;
use slint::{ComponentHandle, SharedString};
use std::cell::RefCell;
use std::collections::HashMap;

struct Shown {
    object: String,
    label: String,
    series: TemperatureSeries,
}

/// The current value of a heater (temperature, target, power) or of a sensor (temperature only).
struct Reading {
    label: String,
    temperature: Option<f64>,
    target: Option<f64>,
    power: Option<f64>,
}

pub struct HistoryView {
    window: slint::Weak<AppWindow>,
    shown: RefCell<Option<Shown>>,
}

impl HistoryView {
    pub fn new(window: &AppWindow) -> Self {
        Self { window: window.as_weak(), shown: RefCell::default() }
    }

    pub fn open(&self, object: &str, status: &PrinterStatus) {
        let label = reading(status, object).map_or_else(|| object.to_owned(), |reading| reading.label);
        *self.shown.borrow_mut() =
            Some(Shown { object: object.to_owned(), label, series: TemperatureSeries::default() });
        if let Some(window) = self.window.upgrade() {
            window.global::<Navigation>().set_history_open(true);
        }
        self.render(status);
    }

    pub fn close(&self) {
        *self.shown.borrow_mut() = None;
        if let Some(window) = self.window.upgrade() {
            window.global::<Navigation>().set_history_open(false);
        }
    }

    /// The answer of Moonraker's store replaces what was collected so far; an object without stored values keeps
    /// its live values.
    pub fn store(&self, mut store: HashMap<String, TemperatureSeries>, status: &PrinterStatus) {
        {
            let mut shown = self.shown.borrow_mut();
            let Some(shown) = shown.as_mut() else { return };
            if let Some(series) = store.remove(&shown.object) {
                shown.series = series;
            }
        }
        self.render(status);
    }

    /// Once a second while open: the current value is added.
    pub fn sample(&self, status: &PrinterStatus) {
        {
            let mut shown = self.shown.borrow_mut();
            let Some(shown) = shown.as_mut() else { return };
            let Some(reading) = reading(status, &shown.object) else { return };
            let Some(temperature) = reading.temperature else { return };
            shown.series.push(temperature, reading.target, reading.power);
        }
        self.render(status);
    }

    fn render(&self, status: &PrinterStatus) {
        let Some(window) = self.window.upgrade() else { return };
        let printer = window.global::<Printer>();
        let shown = self.shown.borrow();
        let Some(shown) = shown.as_ref() else { return };
        let series = &shown.series;
        let has_target = series.targets.iter().any(|target| *target > 0.0);
        let scale = chart::scale(&series.temperatures, if has_target { &series.targets } else { &[] });
        printer.set_history_title(shown.label.as_str().into());
        printer.set_history_current(current_text(reading(status, &shown.object).as_ref()).into());
        printer.set_history_temperature_path(chart::line(&series.temperatures, scale).into());
        printer.set_history_target_path(
            if has_target { chart::line(&series.targets, scale) } else { String::new() }.into(),
        );
        printer
            .set_history_power_path(if has_target { chart::power_line(&series.powers) } else { String::new() }.into());
        printer.set_history_high(SharedString::from(format!("{:.0} °C", scale.high)));
        printer.set_history_low(SharedString::from(format!("{:.0} °C", scale.low)));
        printer.set_history_span(chart::span_text().into());
    }
}

fn reading(status: &PrinterStatus, object: &str) -> Option<Reading> {
    if let Some(heater) = status.heater(object) {
        return Some(Reading {
            label: heater.label.clone(),
            temperature: Some(heater.temperature),
            target: Some(heater.target),
            power: Some(heater.power),
        });
    }
    status.sensors.iter().find(|sensor| sensor.object == object).map(|sensor| Reading {
        label: sensor.label.clone(),
        temperature: sensor.temperature,
        target: None,
        power: None,
    })
}

/// E.g. `212.4 °C / 215 °C`, or `–` while a sensor has no value.
fn current_text(reading: Option<&Reading>) -> String {
    let Some(temperature) = reading.and_then(|reading| reading.temperature) else { return "–".to_owned() };
    match reading.and_then(|reading| reading.target).filter(|target| *target > 0.0) {
        Some(target) => format!("{temperature:.1} °C / {target:.0} °C"),
        None => format!("{temperature:.1} °C"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_current_value_names_the_target_only_when_heating() {
        let reading = |target| Reading { label: String::new(), temperature: Some(212.44), target, power: None };
        assert_eq!(current_text(Some(&reading(Some(215.0)))), "212.4 °C / 215 °C");
        assert_eq!(current_text(Some(&reading(Some(0.0)))), "212.4 °C");
        assert_eq!(current_text(None), "–");
    }
}
