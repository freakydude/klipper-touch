//! Connects the UI's `Actions` callbacks to printer commands.

use crate::presenter::Presenter;
use crate::settings_controller::SettingsController;
use crate::{Actions, AppWindow};
use freakymate_core::sources::klipper::tuning::TuningKey;
use freakymate_core::sources::klipper::{Axis, TestZ};
use freakymate_core::{Command, PrinterService, Query};
use slint::platform::WindowEvent;
use slint::{ComponentHandle, SharedString};
use std::rc::Rc;
use std::time::Duration;

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
    let fan_service = service.clone();
    actions.on_set_fan(move |fan, percent| {
        fan_service.send(Command::SetFanSpeed { fan: fan.into(), speed: f64::from(percent) / 100.0 });
    });
    let light_service = service.clone();
    actions.on_set_light(move |led, percent| {
        light_service.send(Command::SetLight { led: led.into(), level: f64::from(percent) / 100.0 });
    });
    actions.on_toggle_light(send(|| Command::ToggleLight));
    let effect_service = service.clone();
    actions.on_set_led_effect(move |effect, on| {
        effect_service.send(Command::SetLedEffect { effect: effect.into(), on });
    });
    let tuning_service = service.clone();
    actions.on_step_tuning(move |id, direction| {
        if let Some(key) = TuningKey::parse(&id) {
            tuning_service.send(Command::StepTuning { key, direction: f64::from(direction) });
        }
    });
    actions.on_reset_tuning(with(service, |service, id: SharedString| {
        if let Some(key) = TuningKey::parse(&id) {
            service.send(Command::ResetTuning { key });
        }
    }));
    let history_presenter = presenter.clone();
    actions.on_open_history(move |object| history_presenter.open_history(&object));
    let close_presenter = presenter.clone();
    actions.on_close_history(move || close_presenter.close_history());

    actions.on_adjust_z_offset(with(service, |service, delta: f32| {
        service.send(Command::AdjustZOffset { delta: round_offset(delta) })
    }));
    actions.on_reset_z_offset(send(|| Command::ResetZOffset));
    actions.on_apply_z_offset(send(|| Command::ApplyZOffset));
    actions.on_save_config(send(|| Command::SaveConfig));
    actions.on_calibrate_bed_mesh(send(|| Command::CalibrateBedMesh));
    actions.on_load_bed_mesh(with(service, |service, profile: SharedString| {
        service.send(Command::LoadBedMesh { profile: profile.into() })
    }));
    actions.on_clear_bed_mesh(send(|| Command::ClearBedMesh));
    actions.on_calculate_screws_tilt(send(|| Command::CalculateScrewsTilt));
    actions.on_adjust_bed_screws(send(|| Command::AdjustBedScrews));
    actions.on_calibrate_z(send(|| Command::CalibrateZ));
    actions.on_test_z(with(service, |service, distance: f32| {
        service.send(Command::TestZ { step: TestZ::By(round_offset(distance)) })
    }));
    actions.on_bisect_z(with(service, |service, direction: i32| {
        let step = if direction > 0 { TestZ::BisectUp } else { TestZ::BisectDown };
        service.send(Command::TestZ { step })
    }));
    actions.on_accept_calibration(send(|| Command::AcceptCalibration));
    actions.on_screw_adjusted(send(|| Command::ScrewAdjusted));
    actions.on_abort_calibration(send(|| Command::AbortCalibration));
    actions.on_set_speed_factor(with(service, |service, percent: f32| {
        service.send(Command::SetSpeedFactor { percent: f64::from(percent) })
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
    bind_layer_pause(&actions, service, presenter);

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
    let map_presenter = presenter.clone();
    actions.on_print_object_at(move |x_share, y_share, reach_share| {
        map_presenter
            .print_object_at(f64::from(x_share), f64::from(y_share), f64::from(reach_share))
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1)
    });
    let seen_presenter = presenter.clone();
    actions.on_messages_seen(move || seen_presenter.mark_messages_seen());
    let clear_presenter = presenter.clone();
    actions.on_messages_clear(move || clear_presenter.clear_messages());

    let key_window = window.as_weak();
    actions.on_press_key(move |text| press_key(&key_window, text));
}

/// Planning a pause at a layer. The macros report nothing back, so a short notice confirms what was sent; it says
/// nothing about the plan, which may change in another client.
fn bind_layer_pause(actions: &Actions, service: &PrinterService, presenter: &Rc<Presenter>) {
    let send_with_notice = |command: fn(i32) -> Option<(Command, String)>| {
        let (service, presenter) = (service.clone(), Rc::clone(presenter));
        move |layer: i32| {
            if let Some((command, notice)) = command(layer) {
                service.send(command);
                presenter.show_notice(&notice, false);
            }
        }
    };
    let next_layer = send_with_notice(|_| Some((Command::PauseAtNextLayer, "Pause at the next layer sent".into())));
    actions.on_pause_at_next_layer(move || next_layer(0));
    actions.on_pause_at_layer(send_with_notice(|layer| {
        let layer = u32::try_from(layer).ok().filter(|layer| *layer > 0)?;
        Some((Command::PauseAtLayer { layer }, format!("Pause at layer {layer} sent")))
    }));
    let clear = send_with_notice(|_| Some((Command::ClearLayerPause, "Planned pauses cleared".into())));
    actions.on_clear_layer_pause(move || clear(0));
}

/// Types a key of the on-screen keyboard into the focused field, as a physical keyboard would. Deferred, because the
/// tap that caused it is still being handled.
fn press_key(window: &slint::Weak<AppWindow>, text: SharedString) {
    let window = window.clone();
    slint::Timer::single_shot(Duration::ZERO, move || {
        let Some(window) = window.upgrade() else { return };
        window.window().dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
        window.window().dispatch_event(WindowEvent::KeyReleased { text });
    });
}

/// A weight as typed, in grams; a comma as decimal separator is fine. Zero and negative are no weights.
fn parse_weight(text: &str) -> Option<f64> {
    text.trim().replace(',', ".").parse::<f64>().ok().filter(|weight| weight.is_finite() && *weight > 0.0)
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
}
