//! End-to-end test of the service against the fake Moonraker in the same process.

use fake_moonraker::machine::MachineProfile;
use fake_moonraker::server::{serve, ServerConfig};
use klipper_touch_core::printer::{KlippyState, PrintState};
use klipper_touch_core::{Command, ConnectionState, Endpoint, Event, PrinterService, Query, ServiceChannels, Snapshot};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;

const WAIT_LIMIT: Duration = Duration::from_secs(10);

async fn start_fake() -> Endpoint {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let config = ServerConfig {
        time_scale: 1000.0,
        status_interval: Duration::from_millis(20),
        profile: MachineProfile::voron(),
    };
    tokio::spawn(serve(listener, config));
    Endpoint::parse(&address.to_string()).unwrap()
}

async fn wait_for(channels: &mut ServiceChannels, condition: impl Fn(&Snapshot) -> bool) -> Snapshot {
    timeout(WAIT_LIMIT, async {
        loop {
            if condition(&channels.snapshots.borrow_and_update()) {
                return channels.snapshots.borrow().clone();
            }
            channels.snapshots.changed().await.expect("service stopped");
        }
    })
    .await
    .expect("condition not reached in time")
}

async fn next_event(channels: &mut ServiceChannels, matches: impl Fn(&Event) -> bool) -> Event {
    timeout(WAIT_LIMIT, async {
        loop {
            let event = channels.events.recv().await.expect("service stopped");
            if matches(&event) {
                return event;
            }
        }
    })
    .await
    .expect("event not received in time")
}

fn ready(snapshot: &Snapshot) -> bool {
    snapshot.connection == ConnectionState::Connected && snapshot.status.klippy_state == KlippyState::Ready
}

#[tokio::test(flavor = "multi_thread")]
async fn connects_detects_capabilities_and_follows_state() {
    let endpoint = start_fake().await;
    let (service, mut channels) = PrinterService::start(&tokio::runtime::Handle::current(), endpoint);

    let snapshot = wait_for(&mut channels, ready).await;
    assert!(snapshot.capabilities.quad_gantry_level);
    assert!(snapshot.capabilities.has_macro("LOAD_FILAMENT"));
    assert!(!snapshot.capabilities.has_macro("_PURGE_HELPER"));
    assert!(snapshot.capabilities.server.power);
    assert!(snapshot.status.heater("heater_generic chamber").is_some());

    service.send(Command::SetHeaterTarget { heater: "extruder".into(), target: 200.0 });
    service.send(Command::Home { x: true, y: true, z: true });
    let snapshot = wait_for(&mut channels, |snapshot| {
        snapshot.status.heater("extruder").is_some_and(|heater| heater.target == 200.0)
            && snapshot.status.motion.homed.all()
    })
    .await;
    assert_eq!(snapshot.status.motion.position[0], 0.0);

    service.send(Command::Jog { axis: klipper_touch_core::printer::Axis::X, distance: 25.0 });
    wait_for(&mut channels, |snapshot| snapshot.status.motion.position[0] == 25.0).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn runs_print_job_and_loads_file_details() {
    let endpoint = start_fake().await;
    let (service, mut channels) = PrinterService::start(&tokio::runtime::Handle::current(), endpoint);
    wait_for(&mut channels, ready).await;

    service.query(Query::Files);
    let Event::Files(files) = next_event(&mut channels, |event| matches!(event, Event::Files(_))).await else {
        unreachable!()
    };
    assert!(files.iter().any(|file| file.path == "benchy.gcode"));

    service.send(Command::StartPrint { filename: "benchy.gcode".into() });
    wait_for(&mut channels, |snapshot| snapshot.status.job.state == PrintState::Printing).await;
    let metadata = next_event(
        &mut channels,
        |event| matches!(event, Event::FileMetadata(meta) if meta.filename == "benchy.gcode"),
    )
    .await;
    let thumbnail = next_event(&mut channels, |event| matches!(event, Event::Thumbnail { .. })).await;
    assert!(matches!(metadata, Event::FileMetadata(meta) if meta.estimated_time == Some(7200.0)));
    assert!(matches!(thumbnail, Event::Thumbnail { data, .. } if data.starts_with(b"<svg")));

    service.send(Command::CancelPrint);
    wait_for(&mut channels, |snapshot| snapshot.status.job.state == PrintState::Cancelled).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn reports_errors_and_recovers_from_emergency_stop() {
    let endpoint = start_fake().await;
    let (service, mut channels) = PrinterService::start(&tokio::runtime::Handle::current(), endpoint);
    wait_for(&mut channels, ready).await;

    service.send(Command::RunGcode { script: "G1 X10".into() });
    let failure = next_event(&mut channels, |event| matches!(event, Event::CommandFailed { .. })).await;
    assert!(matches!(failure, Event::CommandFailed { message, .. } if message == "Must home axis first"));

    service.send(Command::EmergencyStop);
    let snapshot = wait_for(&mut channels, |snapshot| snapshot.status.klippy_state == KlippyState::Shutdown).await;
    assert!(snapshot.status.klippy_message.contains("M112"));

    service.send(Command::FirmwareRestart);
    wait_for(&mut channels, ready).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn queries_host_functions() {
    let endpoint = start_fake().await;
    let (service, mut channels) = PrinterService::start(&tokio::runtime::Handle::current(), endpoint);
    wait_for(&mut channels, ready).await;

    service.query(Query::PowerDevices);
    let Event::PowerDevices(devices) = next_event(&mut channels, |event| matches!(event, Event::PowerDevices(_))).await
    else {
        unreachable!()
    };
    assert_eq!(devices.len(), 2);

    service.query(Query::Updates);
    let Event::Updates(updates) = next_event(&mut channels, |event| matches!(event, Event::Updates(_))).await else {
        unreachable!()
    };
    assert!(updates.iter().any(|update| update.name == "klipper" && update.has_update()));

    service.send(Command::RunGcode { script: "M104 S10".into() });
    wait_for(&mut channels, |snapshot| snapshot.status.heater("extruder").is_some_and(|heater| heater.target == 10.0))
        .await;
    service.query(Query::ConsoleHistory);
    let Event::ConsoleHistory(lines) =
        next_event(&mut channels, |event| matches!(event, Event::ConsoleHistory(_))).await
    else {
        unreachable!()
    };
    assert!(lines.iter().any(|line| line.message == "M104 S10"));
}

#[tokio::test(flavor = "multi_thread")]
async fn reports_disconnected_when_server_is_missing() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = Endpoint::parse(&listener.local_addr().unwrap().to_string()).unwrap();
    drop(listener);
    let (_service, mut channels) = PrinterService::start(&tokio::runtime::Handle::current(), endpoint);

    wait_for(&mut channels, |snapshot| snapshot.connection == ConnectionState::Disconnected).await;
}
