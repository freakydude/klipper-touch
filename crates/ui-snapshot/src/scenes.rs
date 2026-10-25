//! The run, split into scenes that can be chosen on the command line.

use crate::session::Session;
use klipper_touch::{Confirm, Navigation, Page};
use klipper_touch_core::network::{Availability, NetworkCommand, NetworkState};
use klipper_touch_core::printer::{Activity, KlippyState, PrintState};
use klipper_touch_core::{Command, ConnectionState, Query};
use slint::ComponentHandle;
use std::time::Duration;

/// The fake's idle timeout passes after about 30 s at time scale 20.
const IDLE_WAIT_LIMIT: Duration = Duration::from_secs(60);

/// Scenes run in this order; each one leaves the printer ready for the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scene {
    /// Every page in standby, with the file list.
    Pages,
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
    pub const ALL: [Self; 5] = [Self::Pages, Self::Network, Self::Printing, Self::Shutdown, Self::Idle];
    pub const DEFAULT: [Self; 4] = [Self::Pages, Self::Network, Self::Printing, Self::Shutdown];

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
            .ok_or_else(|| format!("unknown scene '{name}' (pages, network, printing, shutdown, idle, all)"))
    }

    fn name(self) -> &'static str {
        match self {
            Self::Pages => "pages",
            Self::Network => "network",
            Self::Printing => "printing",
            Self::Shutdown => "shutdown",
            Self::Idle => "idle",
        }
    }

    fn run(self, session: &mut Session) -> Result<(), String> {
        match self {
            Self::Pages => pages(session),
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
        (Page::Extrude, "extrude"),
        (Page::Macros, "macros"),
        (Page::Console, "console"),
        (Page::System, "system"),
        (Page::Settings, "settings"),
    ] {
        session.show_page(page, name)?;
    }
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
    session.show_page(Page::Temperature, "temperature-printing")?;
    session.show_page(Page::System, "system-printing")?;

    session.window.global::<Navigation>().set_page(Page::Print);
    ask(session, "Cancel print", "The current print will be stopped and cannot be resumed.", "cancel-print", "");
    session.capture("confirm-cancel")?;
    session.window.global::<Confirm>().invoke_execute();
    session.wait_until("print cancelled", |snapshot| snapshot.status.job.state == PrintState::Cancelled)?;
    session.capture("print-cancelled")
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
