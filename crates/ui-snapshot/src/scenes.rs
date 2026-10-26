//! The run, split into scenes that can be chosen on the command line.

use crate::session::Session;
use freakymate::{Confirm, Navigation, Page, Printer, SpoolPicker};
use freakymate_core::network::{Availability, NetworkCommand, NetworkState};
use freakymate_core::printer::{Activity, KlippyState, PrintState};
use freakymate_core::tuning::TuningKey;
use freakymate_core::{Command, ConnectionState, Query};
use slint::{ComponentHandle, Model};
use std::time::Duration;

/// The fake's idle timeout passes after about 30 s at time scale 20.
const IDLE_WAIT_LIMIT: Duration = Duration::from_secs(60);

/// Scenes run in this order; each one leaves the printer ready for the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scene {
    /// Every page in standby, with the file list.
    Pages,
    /// Spoolman: the spool in use, the temperature hint, the picker and the hints before a print.
    Spools,
    /// Settings > Network: list, connecting, confirmation, read-only fallback.
    Network,
    /// Taps that last longer than a status update still count while the temperatures change.
    Taps,
    /// Editing keys in the text entry of the console, physical and on-screen.
    Typing,
    /// Print running, system page locked, cancel.
    Printing,
    /// Emergency stop and firmware restart.
    Shutdown,
    /// Idle timeout; slow, therefore not part of the default selection.
    Idle,
}

impl Scene {
    pub const ALL: [Self; 8] = [
        Self::Pages,
        Self::Spools,
        Self::Network,
        Self::Taps,
        Self::Typing,
        Self::Printing,
        Self::Shutdown,
        Self::Idle,
    ];
    pub const DEFAULT: [Self; 7] =
        [Self::Pages, Self::Spools, Self::Network, Self::Taps, Self::Typing, Self::Printing, Self::Shutdown];

    /// Parses `pages,network` or `all`.
    pub fn parse_list(text: &str) -> Result<Vec<Self>, String> {
        if text == "all" {
            return Ok(Self::ALL.to_vec());
        }
        text.split(',').map(|name| Self::parse(name.trim())).collect()
    }

    fn parse(name: &str) -> Result<Self, String> {
        Self::ALL.into_iter().find(|scene| scene.name() == name).ok_or_else(|| {
            format!("unknown scene '{name}' (pages, spools, network, taps, typing, printing, shutdown, idle, all)")
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::Pages => "pages",
            Self::Spools => "spools",
            Self::Network => "network",
            Self::Taps => "taps",
            Self::Typing => "typing",
            Self::Printing => "printing",
            Self::Shutdown => "shutdown",
            Self::Idle => "idle",
        }
    }

    fn run(self, session: &mut Session) -> Result<(), String> {
        match self {
            Self::Pages => pages(session),
            Self::Spools => spools(session),
            Self::Network => network(session),
            Self::Taps => taps(session),
            Self::Typing => typing(session),
            Self::Printing => printing(session),
            Self::Shutdown => shutdown(session),
            Self::Idle => idle(session),
        }
    }
}

/// Connects, homes and loads host data, then runs the chosen scenes in their fixed order.
pub fn run(session: &mut Session, chosen: &[Scene]) -> Result<(), String> {
    session.capture("connecting")?;
    session.wait_until("printer ready", |snapshot| {
        snapshot.connection == ConnectionState::Connected && snapshot.status.klippy_state == KlippyState::Ready
    })?;
    session.service.send(Command::Home { x: true, y: true, z: true });
    session.service.query(Query::PowerDevices);
    session.service.query(Query::Updates);
    session.service.query(Query::ConsoleHistory);
    session.service.send(Command::RunGcode { script: "STATUS".into() });
    session.service.send(Command::RunGcode { script: "HELP".into() });
    session.wait_until("homing", |snapshot| snapshot.status.motion.homed.all())?;

    for scene in Scene::ALL.into_iter().filter(|scene| chosen.contains(scene)) {
        scene.run(session)?;
    }
    Ok(())
}

fn pages(session: &mut Session) -> Result<(), String> {
    session.show_page(Page::Print, "print-standby")?;
    session.presenter.request_files();
    session.window.global::<Navigation>().set_files_open(true);
    session.pump(Duration::from_secs(2));
    session.capture("print-files")?;
    session.window.global::<Navigation>().set_files_open(false);
    for (page, name) in [
        (Page::Move, "move"),
        (Page::Temperature, "temperature"),
        (Page::Tune, "tune"),
        (Page::Filament, "filament"),
        (Page::Console, "console"),
        (Page::System, "system"),
        (Page::Updates, "updates"),
        (Page::Settings, "display"),
    ] {
        session.show_page(page, name)?;
        if page == Page::Temperature {
            heat_tabs(session)?;
        }
        if page == Page::Tune && session.presenter.snapshot().capabilities.tuning.get(TuningKey::MaxAccel).is_some() {
            session.window.global::<Navigation>().set_tune_tab(1);
            session.show_page(Page::Tune, "tune-expert")?;
            session.window.global::<Navigation>().set_tune_tab(0);
        }
        if page == Page::Filament && session.presenter.snapshot().status.mmu.is_some() {
            session.window.global::<Navigation>().set_filament_tab(1);
            session.show_page(Page::Filament, "filament-mmu")?;
            session.window.global::<Navigation>().set_filament_tab(0);
        }
    }
    messages(session)
}

/// The filament path during a load: the fake holds the filament in the bowden, then lets it go back to the gate.
fn filament_path_midway(session: &mut Session) -> Result<(), String> {
    let at = |position: i32| {
        move |snapshot: &freakymate_core::Snapshot| {
            snapshot.status.mmu.as_ref().is_some_and(|mmu| mmu.filament_pos == position)
        }
    };
    session.service.send(Command::RunGcode { script: "FAKE_MMU_FILAMENT POS=3 DIRECTION=1".into() });
    session.wait_until("filament in the bowden", at(3))?;
    session.window.global::<Navigation>().set_filament_tab(1);
    session.show_page(Page::Filament, "filament-mmu-loading")?;
    session.window.global::<Navigation>().set_filament_tab(0);
    session.service.send(Command::RunGcode { script: "FAKE_MMU_FILAMENT POS=0 DIRECTION=0".into() });
    session.wait_until("filament back in the gate", at(0))
}

/// The message flyout with an error from a command the printer refuses.
fn messages(session: &mut Session) -> Result<(), String> {
    session.service.send(Command::RunGcode { script: "SET_FAN_SPEED FAN=missing SPEED=1".into() });
    session.pump(Duration::from_secs(1));
    session.window.global::<Navigation>().set_messages_open(true);
    session.capture("messages")?;
    session.window.global::<Navigation>().set_messages_open(false);
    // The scenes after it should not show this made-up error in the status bar.
    session.presenter.clear_messages();
    session.pump(Duration::from_millis(200));
    Ok(())
}

/// The Fans and Sensors tabs of the Thermal page and the light panel, where the printer has them.
fn heat_tabs(session: &mut Session) -> Result<(), String> {
    let status = session.presenter.snapshot().status;
    let navigation = session.window.global::<Navigation>();
    if !status.fans.is_empty() || !status.leds.is_empty() {
        navigation.set_heat_tab(1);
        session.show_page(Page::Temperature, "temperature-fans")?;
    }
    if !status.sensors.is_empty() {
        session.window.global::<Navigation>().set_heat_tab(2);
        session.show_page(Page::Temperature, "temperature-sensors")?;
    }
    session.window.global::<Navigation>().set_heat_tab(0);
    if !status.leds.is_empty() {
        session.window.global::<Navigation>().set_light_open(true);
        session.capture("light-panel")?;
        session.window.global::<Navigation>().set_light_open(false);
    }
    Ok(())
}

/// The temperature history of the nozzle, once it has heated: Moonraker's stored minutes plus the live value.
fn nozzle_history(session: &mut Session) -> Result<(), String> {
    session.presenter.open_history("extruder");
    session.pump(Duration::from_secs(1));
    session.capture("history-nozzle")?;
    session.presenter.close_history();
    Ok(())
}

/// Without Spoolman in Moonraker there is nothing to show and the scene is skipped.
fn spools(session: &mut Session) -> Result<(), String> {
    session.pump(Duration::from_secs(2));
    let snapshot = session.presenter.snapshot();
    if !snapshot.capabilities.server.spoolman {
        return Ok(());
    }
    let has_mmu = snapshot.status.mmu.is_some();
    if has_mmu {
        session.service.send(Command::MmuHome);
        session.wait_until("MMU homed", |snapshot| snapshot.status.mmu.as_ref().is_some_and(|mmu| mmu.homed))?;
        session.service.send(Command::MmuSelectGate { gate: 1 });
        session
            .wait_until("gate 1 selected", |snapshot| snapshot.status.mmu.as_ref().is_some_and(|mmu| mmu.gate == 1))?;
        filament_path_midway(session)?;
    } else {
        session.service.send(Command::SetActiveSpool { spool_id: Some(6) });
    }
    session.pump(Duration::from_secs(2));
    session.service.send(Command::SetHeaterTarget { heater: "extruder".into(), target: 210.0 });
    session.pump(Duration::from_secs(1));
    session.show_page(Page::Print, "print-spool")?;
    session.show_page(Page::Temperature, "temperature-spool")?;

    session.window.global::<Navigation>().set_page(Page::Print);
    // Opens like a tap on the strip or gate 1: the spool in use is selected.
    let current = session.window.global::<Printer>().get_spool_id();
    session.window.global::<SpoolPicker>().invoke_show(if has_mmu { 1 } else { -1 }, current);
    session.pump(Duration::from_secs(2));
    session.capture("spool-picker")?;
    session.window.global::<SpoolPicker>().set_open(false);

    session.presenter.request_files();
    session.pump(Duration::from_secs(2));
    let hints = session.presenter.print_hints_text("long_job.gcode");
    session.window.global::<Confirm>().invoke_ask(
        "Start print".into(),
        format!("long_job{hints}").into(),
        "start-print".into(),
        "long_job.gcode".into(),
        false,
    );
    session.pump(Duration::from_millis(300));
    session.capture("confirm-start-hints")?;
    session.window.global::<Confirm>().set_open(false);
    session.service.send(Command::SetHeaterTarget { heater: "extruder".into(), target: 0.0 });
    session.pump(Duration::from_millis(500));
    Ok(())
}

fn network(session: &mut Session) -> Result<(), String> {
    session.window.global::<Navigation>().set_page(Page::Network);
    session.network_service.send(NetworkCommand::Refresh);
    session.pump(Duration::from_millis(500));
    session.capture("network")?;

    session.network_service.send(NetworkCommand::Connect { ssid: "Cafe Guest".into(), password: None, hidden: false });
    session.pump(Duration::from_millis(200));
    session.capture("network-connecting")?;
    session.pump(Duration::from_secs(2));

    ask(session, "Remove network", "The saved password of \"Garage\" is deleted.", "forget-network", "Garage");
    session.capture("network-forget")?;
    session.window.global::<Confirm>().set_open(false);

    session.network.apply_state(NetworkState {
        availability: Availability::Unavailable("NetworkManager not reachable".into()),
        ..NetworkState::default()
    });
    session.service.query(Query::HostNetwork);
    session.pump(Duration::from_secs(1));
    session.capture("network-unavailable")?;
    session.network_service.send(NetworkCommand::Refresh);
    Ok(())
}

/// The "Cool down" button of the nozzle row on the Heat page, below the tabs (800x480, no fitting).
const COOL_DOWN_BUTTON: (f32, f32) = (628.0, 148.0);
const TAP_ROUNDS: usize = 10;
/// A finger rests this long: several status updates arrive meanwhile, as the temperatures change all the time.
const TAP_DURATION: Duration = Duration::from_millis(500);

/// A tap that spans status updates must still count: the heater rows may not be rebuilt under the finger.
fn taps(session: &mut Session) -> Result<(), String> {
    session.window.global::<Navigation>().set_page(Page::Temperature);
    let heating = |target| Command::SetHeaterTarget { heater: "extruder".into(), target };
    let target = |snapshot: &freakymate_core::Snapshot| snapshot.status.heater("extruder").map(|heater| heater.target);
    let mut missed = 0;
    for _ in 0..TAP_ROUNDS {
        session.service.send(heating(200.0));
        session.wait_until("nozzle target", |snapshot| target(snapshot) == Some(200.0))?;
        session.pump(Duration::from_millis(300));
        let (x, y) = COOL_DOWN_BUTTON;
        session.press(x, y);
        session.pump(TAP_DURATION);
        session.release(x, y);
        session.pump(Duration::from_millis(100));
        if target(&session.presenter.snapshot()) != Some(0.0) {
            missed += 1;
        }
    }
    if missed > 0 {
        return Err(format!("{missed} of {TAP_ROUNDS} taps on Cool down were lost while the temperatures changed"));
    }
    Ok(())
}

/// The command field of the console (800x480, no fitting).
const COMMAND_FIELD: (f32, f32) = (480.0, 444.0);

/// Keys of the on-screen keyboard (800x480, no fitting).
const ON_SCREEN_Q: (f32, f32) = (126.0, 183.0);
const ON_SCREEN_S: (f32, f32) = (196.0, 226.0);
const ON_SCREEN_BACKSPACE: (f32, f32) = (629.0, 314.0);
const ON_SCREEN_ENTER: (f32, f32) = (732.0, 314.0);

/// The text entry edits like any text field: Backspace, Delete, Home, End, Shift and Ctrl are the keyboard's own
/// business. The result is read from the command that the entry sends.
fn typing(session: &mut Session) -> Result<(), String> {
    use slint::platform::Key;
    session.window.global::<Navigation>().set_page(Page::Console);
    session.pump(Duration::from_millis(300));
    let (x, y) = COMMAND_FIELD;
    session.press(x, y);
    session.release(x, y);
    session.pump(Duration::from_millis(300));
    session.capture("text-entry")?;

    // Physical keyboard: "XSTATUZ" -> Backspace -> "S" -> Home -> Delete (removes the X) -> End -> Enter.
    for character in "XSTATUZ".chars() {
        session.key(character.to_string());
    }
    session.key(Key::Backspace);
    session.key("S");
    session.key(Key::Home);
    session.key(Key::Delete);
    session.key(Key::End);
    session.capture("text-entry-typed")?;
    session.key(Key::Return);
    expect_console_command(session, "STATUS")?;

    // On-screen keys: the field must keep the focus while they are tapped. Q, backspace, S, enter.
    session.tap(x, y);
    session.pump(Duration::from_millis(300));
    for (key_x, key_y) in [ON_SCREEN_Q, ON_SCREEN_BACKSPACE, ON_SCREEN_S, ON_SCREEN_ENTER] {
        session.tap(key_x, key_y);
    }
    expect_console_command(session, "S")?;

    // Escape closes without sending.
    session.tap(x, y);
    session.pump(Duration::from_millis(300));
    session.key("H");
    session.key(Key::Escape);
    session.pump(Duration::from_millis(300));
    if session.window.global::<Printer>().get_console().iter().any(|line| line.is_command && line.text == "H") {
        return Err("Escape sent the command instead of closing the entry".to_owned());
    }
    Ok(())
}

/// The command appears as the newest line of the console.
fn expect_console_command(session: &mut Session, command: &str) -> Result<(), String> {
    session.pump(Duration::from_millis(500));
    let lines = session.window.global::<Printer>().get_console();
    let last = lines.iter().last();
    if last.as_ref().is_some_and(|line| line.is_command && line.text == command) {
        return Ok(());
    }
    Err(format!(
        "expected the console to end with the command {command}, it ends with {:?}",
        last.map(|line| line.text)
    ))
}

fn printing(session: &mut Session) -> Result<(), String> {
    session.service.send(Command::StartPrint { filename: "benchy.gcode".into() });
    session.wait_until("print running", |snapshot| {
        snapshot.status.job.state == PrintState::Printing && snapshot.status.job.progress > 0.02
    })?;
    session.pump(Duration::from_secs(1));
    session.show_page(Page::Print, "print-printing")?;
    nozzle_history(session)?;
    tangle_boost(session)?;
    session.show_page(Page::Temperature, "temperature-printing")?;
    session.show_page(Page::System, "system-printing")?;

    session.window.global::<Navigation>().set_page(Page::Print);
    session.service.send(Command::PausePrint);
    session.wait_until("print paused", |snapshot| snapshot.status.job.state == PrintState::Paused)?;
    session.pump(Duration::from_secs(1));
    session.capture("print-paused")?;
    exclude_object(session)?;
    session.service.send(Command::ResumePrint);
    session.wait_until("print resumed", |snapshot| snapshot.status.job.state == PrintState::Printing)?;
    ask(session, "Cancel print", "The current print will be stopped and cannot be resumed.", "cancel-print", "");
    session.capture("confirm-cancel")?;
    session.window.global::<Confirm>().invoke_execute();
    session.wait_until("print cancelled", |snapshot| snapshot.status.job.state == PrintState::Cancelled)?;
    session.capture("print-cancelled")
}

/// Raises the tension in the fake's buffer until tangle prevention boosts the gear current, and takes it back after the
/// picture. Skipped without a buffer that tangle prevention watches (Happy Hare with a proportional sensor).
fn tangle_boost(session: &mut Session) -> Result<(), String> {
    let has_buffer = session.presenter.snapshot().status.mmu.as_ref().is_some_and(|mmu| mmu.flowguard.is_some());
    if !has_buffer {
        return Ok(());
    }
    let boosting = |snapshot: &freakymate_core::Snapshot| {
        snapshot.status.mmu.as_ref().and_then(|mmu| mmu.flowguard.as_ref()).is_some_and(|flowguard| flowguard.boosting)
    };
    session.service.send(Command::RunGcode { script: "FAKE_MMU_TANGLE BOOST=1".into() });
    session.wait_until("tangle prevention boosting", boosting)?;
    session.show_page(Page::Print, "print-tangle-boost")?;
    session.service.send(Command::RunGcode { script: "FAKE_MMU_TANGLE BOOST=0".into() });
    session.wait_until("boost released", |snapshot| !boosting(snapshot))
}

/// Excludes one object of the print and checks that the Objects dialog data follows. Skipped without the module.
fn exclude_object(session: &mut Session) -> Result<(), String> {
    if session.presenter.snapshot().status.print_objects.is_empty() {
        return Ok(());
    }
    session.service.send(Command::ExcludeObject { name: "BENCHY_1".into() });
    session.wait_until("object excluded", |snapshot| {
        snapshot.status.print_objects.iter().any(|object| object.name == "BENCHY_1" && object.excluded)
    })?;
    session.pump(Duration::from_secs(1));
    let items = session.window.global::<Printer>().get_print_objects();
    let excluded: Vec<String> = (0..items.row_count())
        .filter_map(|row| items.row_data(row))
        .filter(|item| item.excluded)
        .map(|item| item.name.to_string())
        .collect();
    if excluded != ["BENCHY_1"] {
        return Err(format!("Printer.print-objects shows excluded {excluded:?}, expected [\"BENCHY_1\"]"));
    }
    session.capture("print-objects-excluded")
}

fn shutdown(session: &mut Session) -> Result<(), String> {
    session.service.send(Command::EmergencyStop);
    session.wait_until("shutdown", |snapshot| snapshot.status.klippy_state == KlippyState::Shutdown)?;
    session.capture("klippy-shutdown")?;
    session.service.send(Command::FirmwareRestart);
    session.wait_until("ready again", |snapshot| snapshot.status.klippy_state == KlippyState::Ready)
}

fn idle(session: &mut Session) -> Result<(), String> {
    session.wait_until_for("idle timeout", IDLE_WAIT_LIMIT, |snapshot| snapshot.status.activity == Activity::Idle)?;
    session.show_page(Page::Print, "print-idle")
}

/// Opens the confirmation dialog as a page would.
fn ask(session: &Session, title: &str, message: &str, action: &str, argument: &str) {
    session.window.global::<Confirm>().invoke_ask(title.into(), message.into(), action.into(), argument.into(), true);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scene_lists() {
        assert_eq!(Scene::parse_list("network, pages"), Ok(vec![Scene::Network, Scene::Pages]));
        assert_eq!(Scene::parse_list("all"), Ok(Scene::ALL.to_vec()));
        assert!(Scene::parse_list("pages,unknown").is_err());
    }
}
