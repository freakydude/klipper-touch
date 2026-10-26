//! Turns core values into the row structs of the Slint models, and keeps those models up to date.

use crate::{format, view};
use crate::{ConsoleItem, HeaterItem, HostInterfaceItem, PowerItem, SpoolItem, UpdateItem};
use freakymate_core::printer::Heater;
use freakymate_core::spoolman::{self, Lane, Spool};
use freakymate_core::system::{ConsoleKind, ConsoleLine, HostInterface, PowerDevice, UpdateInfo};
use freakymate_core::ConnectionState;
use slint::{Model, SharedString, VecModel};

pub(crate) fn connection_name(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Connecting => "connecting",
        ConnectionState::Connected => "connected",
        ConnectionState::Disconnected => "disconnected",
    }
}

/// Brings the rows up to date without rebuilding them: a rebuild (`set_vec`) destroys the buttons in the rows, and a tap
/// that is still going on when the next status arrives would be lost. Only a different number of rows rebuilds the list.
pub(crate) fn replace_rows<T: Clone + PartialEq + 'static>(model: &VecModel<T>, rows: Vec<T>) {
    if model.row_count() != rows.len() {
        model.set_vec(rows);
        return;
    }
    for (index, row) in rows.into_iter().enumerate() {
        if model.row_data(index).as_ref() != Some(&row) {
            model.set_row_data(index, row);
        }
    }
}

/// The picker rows: spools in a lane first, then the most recently used, each with its place.
pub(crate) fn spool_rows(spools: &[Spool], lanes: &[Lane]) -> Vec<SpoolItem> {
    let inserted: Vec<u32> = lanes.iter().filter_map(|lane| lane.spool_id).collect();
    spoolman::picker_order(spools.to_vec(), &inserted)
        .iter()
        .map(|spool| {
            let lane = lanes.iter().find(|lane| lane.spool_id == Some(spool.id));
            let place = match lane {
                Some(lane) => lane.gate.map_or_else(|| "In use".to_owned(), |gate| format!("Gate {gate}")),
                None => String::new(),
            };
            let bar = color_bar(&spool.colors);
            SpoolItem {
                id: spool.id as i32,
                title: spool.title().into(),
                detail: view::spool_detail(spool).into(),
                color: bar.0,
                color_2: bar.1,
                color_3: bar.2,
                color_count: bar.3,
                place: place.into(),
                low: spool.is_low(),
            }
        })
        .collect()
}

pub(crate) fn color_of([red, green, blue]: [u8; 3]) -> slint::Color {
    slint::Color::from_rgb_u8(red, green, blue)
}

/// The colours of a bar: up to three, and how many there are.
pub(crate) fn color_bar(colors: &[[u8; 3]]) -> (slint::Color, slint::Color, slint::Color, i32) {
    let color = |index: usize| colors.get(index).copied().map_or_else(slint::Color::default, color_of);
    (color(0), color(1), color(2), colors.len().min(3) as i32)
}

/// The row of a heater; the hint says that the target is off from what the spool in use recommends.
pub(crate) fn heater_item(heater: &Heater, spool: Option<&Spool>) -> HeaterItem {
    let recommended = spool.and_then(|spool| match heater.object.as_str() {
        "extruder" => spool.extruder_temp,
        "heater_bed" => spool.bed_temp,
        _ => None,
    });
    let spool_hint = spoolman::temperature_hint(heater.target, recommended)
        .map_or_else(SharedString::new, |degrees| format!("Spool {degrees:.0}°").into());
    HeaterItem {
        spool_hint,
        object: heater.object.as_str().into(),
        label: heater.label.as_str().into(),
        temperature: heater.temperature as f32,
        target: heater.target as f32,
        power: heater.power as f32,
        can_raise: heater.can_raise(),
    }
}

/// One row per message line: the console list has fixed row heights, so a line break inside a row would overlap its neighbours.
/// Only the first row of a message shows the time.
pub(crate) fn console_items(line: &ConsoleLine) -> Vec<ConsoleItem> {
    let time = format::unix_clock(line.time);
    let is_command = line.kind == ConsoleKind::Command;
    let is_error = line.is_error();
    let text = line.text();
    let mut rows: Vec<&str> = text.lines().collect();
    if rows.is_empty() {
        rows.push("");
    }
    rows.into_iter()
        .enumerate()
        .map(|(index, text)| ConsoleItem {
            text: text.into(),
            time: if index == 0 { time.as_str().into() } else { "".into() },
            is_command,
            is_error,
        })
        .collect()
}

pub(crate) fn power_item(device: &PowerDevice) -> PowerItem {
    PowerItem { name: device.device.as_str().into(), on: device.is_on(), locked: device.locked_while_printing }
}

pub(crate) fn host_interface_item(interface: &HostInterface) -> HostInterfaceItem {
    HostInterfaceItem {
        name: interface.name.as_str().into(),
        wireless: interface.is_wireless(),
        addresses: interface.addresses.join(", ").into(),
        mac: interface.mac_address.as_str().into(),
    }
}

pub(crate) fn update_item(update: &UpdateInfo) -> UpdateItem {
    let detail = if update.name == "system" {
        format!("{} packages", update.pending)
    } else if update.has_update() {
        format!("{} → {}", update.version, update.remote_version)
    } else {
        update.version.clone()
    };
    UpdateItem {
        name: update.name.as_str().into(),
        version: update.version.as_str().into(),
        detail: detail.into(),
        has_update: update.has_update(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(kind: ConsoleKind, message: &str) -> ConsoleLine {
        ConsoleLine { kind, message: message.to_owned(), time: 0.0 }
    }

    #[test]
    fn a_colour_bar_has_up_to_three_colours() {
        let red = [255, 0, 0];
        assert_eq!(color_bar(&[]).3, 0);
        assert_eq!(color_bar(&[red]).3, 1);
        assert_eq!(color_bar(&[red, [0, 255, 0]]).3, 2);
        assert_eq!(color_bar(&[red; 5]).3, 3);
        assert_eq!(color_bar(&[red, [0, 255, 0]]).1, slint::Color::from_rgb_u8(0, 255, 0));
    }

    #[test]
    fn multi_line_message_becomes_one_row_per_line() {
        let rows = console_items(&line(ConsoleKind::Response, "first\nsecond\r\nthird"));
        let texts: Vec<&str> = rows.iter().map(|row| row.text.as_str()).collect();
        assert_eq!(texts, ["first", "second", "third"]);
        assert!(rows[1].time.is_empty() && rows[2].time.is_empty());
    }

    #[test]
    fn empty_message_keeps_one_row() {
        let rows = console_items(&line(ConsoleKind::Response, ""));
        assert_eq!(rows.len(), 1);
        assert!(rows[0].text.is_empty());
    }

    #[test]
    fn plugin_markup_does_not_reach_the_console() {
        let rows = console_items(&line(
            ConsoleKind::Response,
            r#"// <b><span style="color:#87CEEB">EMPTY</span></b>&nbsp;gate"#,
        ));
        assert_eq!(rows[0].text, "EMPTY gate");
    }

    #[test]
    fn error_flag_applies_to_every_row() {
        let rows = console_items(&line(ConsoleKind::Response, "!! Lost connection\nsee log"));
        assert_eq!(rows[0].text, "Lost connection");
        assert!(rows.iter().all(|row| row.is_error));
        assert!(!rows.iter().any(|row| row.is_command));
    }
}
