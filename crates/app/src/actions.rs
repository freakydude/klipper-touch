//! Connects the UI's `Actions` callbacks to printer commands.

use crate::presenter::Presenter;
use crate::settings_controller::SettingsController;
use crate::{Actions, AppWindow, TextTools};
use freakymate_core::printer::Axis;
use freakymate_core::{Command, PrinterService, Query};
use slint::ComponentHandle;
use std::rc::Rc;

pub fn bind(
    window: &AppWindow,
    service: &PrinterService,
    presenter: &Rc<Presenter>,
    settings: &Rc<SettingsController>,
) {
    let actions = window.global::<Actions>();
    let send = |command: fn() -> Command| {
        let service = service.clone();
        move || service.send(command())
    };

    let home_service = service.clone();
    actions.on_home(move |x, y, z| home_service.send(Command::Home { x, y, z }));
    let jog_service = service.clone();
    actions.on_jog(move |axis, distance| {
        if let Some(axis) = parse_axis(&axis) {
            jog_service.send(Command::Jog { axis, distance: f64::from(distance) });
        }
    });
    actions.on_disable_motors(send(|| Command::DisableMotors));
    actions.on_level(send(|| Command::Level));

    let heater_service = service.clone();
    actions.on_set_heater(move |heater, target| {
        heater_service.send(Command::SetHeaterTarget { heater: heater.into(), target: f64::from(target) });
    });
    let step_service = service.clone();
    actions.on_step_heater(move |heater, step| {
        step_service.send(Command::StepHeater { heater: heater.into(), step: f64::from(step) });
    });
    let (preset_service, preset_settings) = (service.clone(), Rc::clone(settings));
    actions.on_apply_preset(move |index| {
        let Some(preset) = usize::try_from(index).ok().and_then(|index| preset_settings.preset(index)) else { return };
        preset_service.send(Command::ApplyPreset { extruder: preset.extruder, bed: preset.bed });
    });
    actions.on_cool_down(send(|| Command::CoolDown));
    actions.on_set_part_fan(with(service, |service, percent: f32| {
        service.send(Command::SetPartFanSpeed { fraction: f64::from(percent) / 100.0 });
    }));

    actions.on_adjust_z_offset(with(service, |service, delta: f32| {
        service.send(Command::AdjustZOffset { delta: round_offset(delta) })
    }));
    actions.on_reset_z_offset(send(|| Command::ResetZOffset));
    actions.on_apply_z_offset(send(|| Command::ApplyZOffset));
    actions.on_save_config(send(|| Command::SaveConfig));
    actions.on_set_speed_factor(with(service, |service, percent: f32| {
        service.send(Command::SetSpeedFactor { percent: f64::from(percent) })
    }));
    actions.on_set_extrude_factor(with(service, |service, percent: f32| {
        service.send(Command::SetExtrudeFactor { percent: f64::from(percent) });
    }));
    actions.on_set_pressure_advance(with(service, |service, value: f32| {
        service.send(Command::SetPressureAdvance { value: round_offset(value) });
    }));

    let extrude_service = service.clone();
    actions.on_extrude(move |length, speed| {
        extrude_service.send(Command::Extrude { length: f64::from(length), speed: f64::from(speed) });
    });
    actions.on_load_filament(send(|| Command::LoadFilament));
    actions.on_unload_filament(send(|| Command::UnloadFilament));
    actions.on_run_macro(with(service, |service, name: slint::SharedString| {
        service.send(Command::RunMacro { name: name.into() })
    }));
    actions.on_exclude_object(with(service, |service, name: slint::SharedString| {
        service.send(Command::ExcludeObject { name: name.into() })
    }));
    actions.on_mmu_select_gate(with(service, |service, gate: i32| {
        if let Ok(gate) = usize::try_from(gate) {
            service.send(Command::MmuSelectGate { gate });
        }
    }));
    actions.on_mmu_load(send(|| Command::MmuLoad));
    actions.on_mmu_unload(send(|| Command::MmuUnload));
    actions.on_mmu_home(send(|| Command::MmuHome));
    actions.on_mmu_eject(send(|| Command::MmuEject { gate: None }));
    actions.on_mmu_recover(send(|| Command::MmuRecover));
    actions.on_mmu_unlock(send(|| Command::MmuUnlock));
    actions.on_run_gcode(with(service, |service, script: slint::SharedString| {
        service.send(Command::RunGcode { script: script.into() })
    }));

    let spools_presenter = Rc::clone(presenter);
    actions.on_load_spools(move || spools_presenter.load_spools());
    actions.on_set_active_spool(with(service, |service, id: i32| {
        service.send(Command::SetActiveSpool { spool_id: u32::try_from(id).ok() });
    }));
    let assign_service = service.clone();
    actions.on_assign_gate_spool(move |gate, id| {
        if let Ok(gate) = usize::try_from(gate) {
            assign_service.send(Command::AssignGateSpool { gate, spool_id: u32::try_from(id).ok() });
        }
    });
    let (measure_service, measure_presenter) = (service.clone(), Rc::clone(presenter));
    actions.on_measure_spool(move |id, text| match (u32::try_from(id), parse_weight(&text)) {
        (Ok(spool_id), Some(weight)) => measure_service.send(Command::MeasureSpool { spool_id, weight }),
        _ => measure_presenter.show_notice("Enter the weight of the spool in grams", true),
    });
    let preset_presenter = Rc::clone(presenter);
    actions.on_apply_spool_preset(move || preset_presenter.apply_spool_preset());
    let hints_presenter = Rc::clone(presenter);
    actions.on_print_hints(move |path| hints_presenter.print_hints_text(&path).into());

    let files_presenter = Rc::clone(presenter);
    actions.on_refresh_files(move || files_presenter.request_files());
    actions.on_start_print(with(service, |service, filename: slint::SharedString| {
        service.send(Command::StartPrint { filename: filename.into() });
    }));
    actions.on_pause_print(send(|| Command::PausePrint));
    actions.on_resume_print(send(|| Command::ResumePrint));
    actions.on_cancel_print(send(|| Command::CancelPrint));

    actions.on_emergency_stop(send(|| Command::EmergencyStop));
    actions.on_restart_klipper(send(|| Command::RestartKlipper));
    actions.on_firmware_restart(send(|| Command::FirmwareRestart));
    let system_service = service.clone();
    actions.on_refresh_system(move || {
        system_service.query(Query::PowerDevices);
        system_service.query(Query::Updates);
    });
    let power_service = service.clone();
    actions.on_set_power(move |device, on| power_service.send(Command::SetPowerDevice { device: device.into(), on }));
    actions.on_restart_service(with(service, |service, name: slint::SharedString| {
        service.send(Command::RestartService { service: name.into() })
    }));
    actions.on_reboot_host(send(|| Command::RebootHost));
    actions.on_shutdown_host(send(|| Command::ShutdownHost));
    let updates_service = service.clone();
    actions.on_refresh_updates(move || {
        updates_service.send(Command::RefreshUpdates);
        updates_service.query(Query::Updates);
    });
    actions.on_upgrade(with(service, |service, name: slint::SharedString| {
        service.send(Command::Upgrade { name: name.into() })
    }));
    let console_service = service.clone();
    actions.on_load_console(move || console_service.query(Query::ConsoleHistory));
    let console_presenter = presenter.clone();
    actions.on_clear_console(move || console_presenter.clear_console());
    let macro_settings = settings.clone();
    actions.on_set_macro_visible(move |name, visible| macro_settings.set_macro_visible(name.as_str(), visible));
    let has_params_presenter = presenter.clone();
    actions.on_macro_has_params(move |name| has_params_presenter.macro_has_params(&name));
    let open_presenter = presenter.clone();
    actions.on_open_macro_form(move |name| open_presenter.open_macro_form(&name));
    let param_presenter = presenter.clone();
    actions.on_set_macro_param(move |index, value| {
        if let Ok(index) = usize::try_from(index) {
            param_presenter.set_macro_param(index, &value);
        }
    });
    let script_presenter = presenter.clone();
    actions.on_macro_form_script(move || script_presenter.macro_form_script().into());
    let seen_presenter = presenter.clone();
    actions.on_messages_seen(move || seen_presenter.mark_messages_seen());
    let clear_presenter = presenter.clone();
    actions.on_messages_clear(move || clear_presenter.clear_messages());

    window.global::<TextTools>().on_remove_last(|text| {
        let mut text = text.to_string();
        text.pop();
        text.into()
    });
    window.global::<TextTools>().on_mask(|text| mask(&text).into());
}

/// A weight as typed, in grams; a comma as decimal separator is fine. Zero and negative are no weights.
fn parse_weight(text: &str) -> Option<f64> {
    text.trim().replace(',', ".").parse::<f64>().ok().filter(|weight| weight.is_finite() && *weight > 0.0)
}

fn mask(text: &str) -> String {
    "•".repeat(text.chars().count())
}

/// Callback with arguments that receives a cloned service.
fn with<A, F>(service: &PrinterService, action: F) -> impl Fn(A) + 'static
where
    F: Fn(&PrinterService, A) + 'static,
{
    let service = service.clone();
    move |arguments| action(&service, arguments)
}

fn parse_axis(axis: &str) -> Option<Axis> {
    match axis {
        "x" | "X" => Some(Axis::X),
        "y" | "Y" => Some(Axis::Y),
        "z" | "Z" => Some(Axis::Z),
        _ => None,
    }
}

/// Slint computes in f32; small steps like 0.005 are smoothed to five decimal places.
fn round_offset(value: f32) -> f64 {
    (f64::from(value) * 100_000.0).round() / 100_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_names_and_rounding() {
        assert_eq!(parse_axis("y"), Some(Axis::Y));
        assert_eq!(parse_axis("e"), None);
        assert_eq!(round_offset(0.005_f32), 0.005);
        assert_eq!(round_offset(-0.025_f32), -0.025);
    }

    #[test]
    fn weights_are_read_with_a_point_or_a_comma_and_must_be_positive() {
        assert_eq!(parse_weight("793.5"), Some(793.5));
        assert_eq!(parse_weight(" 793,5 "), Some(793.5));
        assert_eq!(parse_weight("0"), None);
        assert_eq!(parse_weight("-3"), None);
        assert_eq!(parse_weight("abc"), None);
        assert_eq!(parse_weight(""), None);
    }

    #[test]
    fn masks_every_character() {
        assert_eq!(mask("päss"), "••••");
        assert_eq!(mask(""), "");
    }
}
