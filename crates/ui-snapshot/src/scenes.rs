//! The run, split into scenes that can be chosen on the command line.

use crate::session::Session;
use klipper_touch::{Confirm, Navigation, Page, Printer, SpoolPicker};
use klipper_touch_core::network::{Availability, NetworkCommand, NetworkState};
use klipper_touch_core::printer::{Activity, KlippyState, PrintState};
use klipper_touch_core::{Command, ConnectionState, Query};
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
    /// Print running, system page locked, cancel.
    Printing,
    /// Emergency stop and firmware restart.
    Shutdown,
    /// Idle timeout; slow, therefore not part of the default selection.
    Idle,
}

impl Scene {
    pub const ALL: [Self; 6] = [Self::Pages, Self::Spools, Self::Network, Self::Printing, Self::Shutdown, Self::Idle];
    pub const DEFAULT: [Self; 5] = [Self::Pages, Self::Spools, Self::Network, Self::Printing, Self::Shutdown];

    /// Parses `pages,network` or `all`.
    pub fn parse_list(text: &str) -> Result<Vec<Self>, String> {
        if text == "all" {
            return Ok(Self::ALL.to_vec());
        }
        text.split(',').map(|name| Self::parse(name.trim())).collect()
    }

    fn parse(name: &str) -> Result<Self, String> {
        Self::ALL
            .into_iter()
            .find(|scene| scene.name() == name)
            .ok_or_else(|| format!("unknown scene '{name}' (pages, spools, network, printing, shutdown, idle, all)"))
    }

    fn name(self) -> &'static str {
        match self {
            Self::Pages => "pages",
            Self::Spools => "spools",
            Self::Network => "network",
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
        (Page::Settings, "settings"),
    ] {
        session.show_page(page, name)?;
        if page == Page::Filament && session.presenter.snapshot().status.mmu.is_some() {
            session.window.global::<Navigation>().set_filament_tab(1);
            session.show_page(Page::Filament, "filament-mmu")?;
            session.window.global::<Navigation>().set_filament_tab(0);
        }
    }
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
    } else {
        session.service.send(Command::SetActiveSpool { spool_id: Some(6) });
    }
    session.pump(Duration::from_secs(2));
    session.service.send(Command::SetHeaterTarget { heater: "extruder".into(), target: 210.0 });
    session.pump(Duration::from_secs(1));
    session.show_page(Page::Print, "print-spool")?;
    session.show_page(Page::Temperature, "temperature-spool")?;

    session.window.global::<Navigation>().set_page(Page::Print);
    session.window.global::<SpoolPicker>().invoke_show(if has_mmu { 1 } else { -1 });
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
    session.window.global::<Navigation>().set_page(Page::Settings);
    session.window.global::<Navigation>().set_settings_tab(1);
    session.network_service.send(NetworkCommand::Refresh);
    session.pump(Duration::from_millis(500));
    session.capture("settings-network")?;

    session.network_service.send(NetworkCommand::Connect { ssid: "Cafe Guest".into(), password: None, hidden: false });
    session.pump(Duration::from_millis(200));
    session.capture("settings-network-connecting")?;
    session.pump(Duration::from_secs(2));

    ask(session, "Remove network", "The saved password of \"Garage\" is deleted.", "forget-network", "Garage");
    session.capture("settings-network-forget")?;
    session.window.global::<Confirm>().set_open(false);

    session.network.apply_state(NetworkState {
        availability: Availability::Unavailable("NetworkManager not reachable".into()),
        ..NetworkState::default()
    });
    session.service.query(Query::HostNetwork);
    session.pump(Duration::from_secs(1));
    session.capture("settings-network-unavailable")?;
    session.window.global::<Navigation>().set_settings_tab(0);
    session.network_service.send(NetworkCommand::Refresh);
    Ok(())
}

fn printing(session: &mut Session) -> Result<(), String> {
    session.service.send(Command::StartPrint { filename: "benchy.gcode".into() });
    session.wait_until("print running", |snapshot| {
        snapshot.status.job.state == PrintState::Printing && snapshot.status.job.progress > 0.02
    })?;
    session.pump(Duration::from_secs(1));
    session.show_page(Page::Print, "print-printing")?;
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
    let boosting = |snapshot: &klipper_touch_core::Snapshot| {
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
