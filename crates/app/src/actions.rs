//! Connects the UI's `Actions` callbacks to printer commands.

use crate::presenter::Presenter;
use crate::settings_controller::SettingsController;
use crate::{Actions, AppWindow, TextTools};
use klipper_touch_core::printer::Axis;
use klipper_touch_core::{Command, PrinterService, Query};
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
    let (level_service, level_presenter) = (service.clone(), Rc::clone(presenter));
    actions.on_level(move || {
        let capabilities = level_presenter.capabilities();
        if capabilities.quad_gantry_level {
            level_service.send(Command::QuadGantryLevel);
        } else if capabilities.z_tilt {
            level_service.send(Command::ZTiltAdjust);
        }
    });

    let heater_presenter = Rc::clone(presenter);
    actions.on_set_heater(move |heater, target| heater_presenter.set_heater_target(&heater, f64::from(target)));
    let step_presenter = Rc::clone(presenter);
    actions.on_step_heater(move |object, step| {
        if let Some(heater) = step_presenter.heater(&object) {
            step_presenter.set_heater_target(&object, heater.stepped_target(f64::from(step)));
        }
    });
    let (preset_settings, preset_presenter) = (Rc::clone(settings), Rc::clone(presenter));
    actions.on_apply_preset(move |index| {
        let Some(preset) = usize::try_from(index).ok().and_then(|index| preset_settings.preset(index)) else { return };
        preset_presenter.set_heater_target("extruder", preset.extruder);
        if preset_presenter.capabilities().heater_bed {
            preset_presenter.set_heater_target("heater_bed", preset.bed);
        }
    });
    let cool_presenter = Rc::clone(presenter);
    actions.on_cool_down(move || {
        for heater in cool_presenter.snapshot().status.heaters {
            cool_presenter.set_heater_target(&heater.object, 0.0);
        }
    });
    actions.on_set_part_fan(with(service, |service, percent: f32| {
        service.send(Command::SetPartFanSpeed { fraction: f64::from(percent) / 100.0 });
    }));

    actions.on_adjust_z_offset(with(service, |service, delta: f32| {
        service.send(Command::AdjustZOffset { delta: round_offset(delta) })
    }));
    actions.on_reset_z_offset(send(|| Command::ResetZOffset));
    let (offset_service, offset_presenter) = (service.clone(), Rc::clone(presenter));
    actions.on_apply_z_offset(move || {
        offset_service.send(Command::ApplyZOffset { to_probe: offset_presenter.capabilities().probe });
    });
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
    actions.on_load_filament(send(|| Command::RunMacro { name: "LOAD_FILAMENT".into() }));
    actions.on_unload_filament(send(|| Command::RunMacro { name: "UNLOAD_FILAMENT".into() }));
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
        let script = script.trim().to_owned();
        if !script.is_empty() {
            service.send(Command::RunGcode { script });
        }
    }));

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
    fn masks_every_character() {
        assert_eq!(mask("päss"), "••••");
        assert_eq!(mask(""), "");
    }
}
