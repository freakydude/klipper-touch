//! Transfers core state and events into the Slint globals. Runs on the UI thread only.

use crate::{format, thumbnail, view};
use crate::{
    AppWindow, ConsoleItem, FilamentSensorItem, FileItem, HeaterItem, HostInterfaceItem, Network, PowerItem, Printer,
    SensorItem, UpdateItem,
};
use chrono::Local;
use klipper_touch_core::files::{FileEntry, FileMetadata};
use klipper_touch_core::printer::{Capabilities, Heater};
use klipper_touch_core::system::{ConsoleKind, ConsoleLine, HostInterface, PowerDevice, UpdateInfo};
use klipper_touch_core::{Command, ConnectionState, Event, PrinterService, Query, Snapshot};
use slint::{ComponentHandle, Image, Model, ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

const CONSOLE_LIMIT: usize = 300;
/// This many files get metadata and thumbnail right away; the rest stays plain.
const FILE_DETAILS_LIMIT: usize = 40;
const NOTICE_DURATION: Duration = Duration::from_secs(4);

#[derive(Default)]
struct Cache {
    snapshot: Snapshot,
    files: Vec<FileEntry>,
    metadata: HashMap<String, FileMetadata>,
    thumbnails: HashMap<String, Image>,
}

pub struct Presenter {
    window: slint::Weak<AppWindow>,
    service: PrinterService,
    cache: RefCell<Cache>,
    pending_targets: RefCell<view::PendingTargets>,
    heaters: Rc<VecModel<HeaterItem>>,
    sensors: Rc<VecModel<SensorItem>>,
    filament_sensors: Rc<VecModel<FilamentSensorItem>>,
    macros: Rc<VecModel<SharedString>>,
    files: Rc<VecModel<FileItem>>,
    console: Rc<VecModel<ConsoleItem>>,
    power_devices: Rc<VecModel<PowerItem>>,
    updates: Rc<VecModel<UpdateItem>>,
    host_interfaces: Rc<VecModel<HostInterfaceItem>>,
    notice_timer: Timer,
}

impl Presenter {
    pub fn new(window: &AppWindow, service: PrinterService) -> Rc<Self> {
        let presenter = Rc::new(Self {
            window: window.as_weak(),
            service,
            cache: RefCell::default(),
            pending_targets: RefCell::default(),
            heaters: Rc::default(),
            sensors: Rc::default(),
            filament_sensors: Rc::default(),
            macros: Rc::default(),
            files: Rc::default(),
            console: Rc::default(),
            power_devices: Rc::default(),
            updates: Rc::default(),
            host_interfaces: Rc::default(),
            notice_timer: Timer::default(),
        });
        let printer = window.global::<Printer>();
        printer.set_heaters(ModelRc::from(presenter.heaters.clone()));
        printer.set_sensors(ModelRc::from(presenter.sensors.clone()));
        printer.set_filament_sensors(ModelRc::from(presenter.filament_sensors.clone()));
        printer.set_macros(ModelRc::from(presenter.macros.clone()));
        printer.set_files(ModelRc::from(presenter.files.clone()));
        printer.set_console(ModelRc::from(presenter.console.clone()));
        printer.set_power_devices(ModelRc::from(presenter.power_devices.clone()));
        printer.set_updates(ModelRc::from(presenter.updates.clone()));
        window.global::<Network>().set_host_interfaces(ModelRc::from(presenter.host_interfaces.clone()));
        presenter
    }

    /// Connects the service's snapshots and events to the UI (asynchronously in the Slint event loop).
    pub fn follow(self: &Rc<Self>, channels: klipper_touch_core::ServiceChannels) -> Result<(), slint::EventLoopError> {
        let klipper_touch_core::ServiceChannels { mut snapshots, mut events } = channels;
        let presenter = Rc::clone(self);
        slint::spawn_local(async move {
            loop {
                let snapshot = snapshots.borrow_and_update().clone();
                presenter.apply_snapshot(&snapshot);
                if snapshots.changed().await.is_err() {
                    break;
                }
            }
        })?;
        let presenter = Rc::clone(self);
        slint::spawn_local(async move {
            while let Some(event) = events.recv().await {
                presenter.handle_event(event);
            }
        })?;
        Ok(())
    }

    pub fn snapshot(&self) -> Snapshot {
        self.cache.borrow().snapshot.clone()
    }

    pub fn capabilities(&self) -> Capabilities {
        self.cache.borrow().snapshot.capabilities.clone()
    }

    /// Heater as the UI shows it, including targets Klipper has not reported back yet.
    pub fn heater(&self, object: &str) -> Option<Heater> {
        let mut heaters = self.cache.borrow().snapshot.status.heaters.clone();
        self.pending_targets.borrow_mut().apply(&mut heaters, Instant::now());
        heaters.into_iter().find(|heater| heater.object == object)
    }

    /// Sends a heater target and shows it right away instead of waiting for the next status update.
    pub fn set_heater_target(&self, object: &str, target: f64) {
        self.pending_targets.borrow_mut().request(object, target, Instant::now());
        self.service.send(Command::SetHeaterTarget { heater: object.to_owned(), target });
        let snapshot = self.snapshot();
        self.apply_snapshot(&snapshot);
    }

    pub fn apply_snapshot(&self, snapshot: &Snapshot) {
        let Some(window) = self.window.upgrade() else { return };
        let printer = window.global::<Printer>();
        let mut shown = snapshot.status.clone();
        self.pending_targets.borrow_mut().apply(&mut shown.heaters, Instant::now());
        let status = &shown;
        let connected = snapshot.connection == ConnectionState::Connected;
        let ready = connected && status.klippy_state == klipper_touch_core::printer::KlippyState::Ready;

        printer.set_connection(connection_name(snapshot.connection).into());
        printer.set_klippy_state(status.klippy_state.as_str().into());
        printer.set_klippy_message(status.klippy_message.as_str().into());
        printer.set_endpoint(snapshot.endpoint.as_str().into());
        printer.set_ready(ready);

        printer.set_print_state(status.job.state.as_str().into());
        printer.set_activity(status.activity.as_str().into());
        printer.set_filename(status.job.filename.as_str().into());
        printer.set_progress(status.job.progress as f32);
        printer.set_job_message(status.job.message.as_str().into());

        let motion = &status.motion;
        printer.set_position_x(motion.live_position[0] as f32);
        printer.set_position_y(motion.live_position[1] as f32);
        printer.set_position_z(motion.live_position[2] as f32);
        printer.set_homed_x(motion.homed.x);
        printer.set_homed_y(motion.homed.y);
        printer.set_homed_z(motion.homed.z);
        printer.set_z_offset(motion.homing_origin[2] as f32);
        printer.set_speed_factor((motion.speed_factor * 100.0) as f32);
        printer.set_extrude_factor((motion.extrude_factor * 100.0) as f32);
        printer.set_live_velocity(motion.live_velocity as f32);
        printer.set_live_flow(view::volumetric_flow(motion.live_extruder_velocity) as f32);
        printer.set_pressure_advance(status.extruder.pressure_advance as f32);
        printer.set_can_extrude(status.extruder.can_extrude);

        let nozzle = status.heater("extruder");
        printer.set_nozzle_temperature(nozzle.map_or(0.0, |heater| heater.temperature) as f32);
        printer.set_nozzle_target(nozzle.map_or(0.0, |heater| heater.target) as f32);
        let bed = status.heater("heater_bed");
        printer.set_has_bed(bed.is_some());
        printer.set_bed_temperature(bed.map_or(0.0, |heater| heater.temperature) as f32);
        printer.set_bed_target(bed.map_or(0.0, |heater| heater.target) as f32);
        printer.set_has_part_fan(status.part_fan_speed.is_some());
        printer.set_part_fan((status.part_fan_speed.unwrap_or_default() * 100.0) as f32);
        printer.set_save_config_pending(status.save_config_pending);
        printer.set_leveling_state(match status.leveling_applied {
            Some(true) => "applied".into(),
            Some(false) => "pending".into(),
            None => SharedString::new(),
        });

        replace_rows(&self.heaters, status.heaters.iter().map(heater_item).collect());
        replace_rows(
            &self.sensors,
            status
                .sensors
                .iter()
                .map(|sensor| SensorItem {
                    label: sensor.label.as_str().into(),
                    temperature: sensor.temperature as f32,
                })
                .collect(),
        );
        replace_rows(
            &self.filament_sensors,
            status
                .filament_sensors
                .iter()
                .map(|sensor| FilamentSensorItem {
                    label: sensor.label.as_str().into(),
                    enabled: sensor.enabled,
                    detected: sensor.detected,
                })
                .collect(),
        );

        let capabilities = &snapshot.capabilities;
        printer.set_can_quad_gantry_level(capabilities.quad_gantry_level);
        printer.set_can_z_tilt(capabilities.z_tilt);
        printer.set_has_probe(capabilities.probe);
        printer.set_has_load_macro(capabilities.has_macro("LOAD_FILAMENT"));
        printer.set_has_unload_macro(capabilities.has_macro("UNLOAD_FILAMENT"));
        printer.set_has_power(capabilities.server.power);
        printer.set_has_update_manager(capabilities.server.update_manager);
        replace_rows(&self.macros, capabilities.macros.iter().map(|name| SharedString::from(name.as_str())).collect());

        let previous_job =
            std::mem::replace(&mut self.cache.borrow_mut().snapshot, snapshot.clone()).status.job.filename;
        if previous_job != status.job.filename {
            self.show_job_thumbnail();
        }
        self.refresh_job_texts();
    }

    /// Time-related texts; also called every second by the clock timer.
    pub fn refresh_job_texts(&self) {
        let Some(window) = self.window.upgrade() else { return };
        let printer = window.global::<Printer>();
        let now = Local::now();
        let cache = self.cache.borrow();
        let job = &cache.snapshot.status.job;
        let texts = view::job_texts(job, cache.metadata.get(&job.filename), now);
        printer.set_clock(format::clock(now).into());
        printer.set_job_name(texts.name.into());
        printer.set_elapsed(texts.elapsed.into());
        printer.set_remaining(texts.remaining.into());
        printer.set_eta(texts.eta.into());
        printer.set_layers(texts.layers.into());
        printer.set_filament(texts.filament.into());
    }

    pub fn handle_event(&self, event: Event) {
        match event {
            Event::Files(files) => self.show_files(files),
            Event::FileMetadata(metadata) => self.store_metadata(metadata),
            Event::Thumbnail { filename, data } => self.store_thumbnail(&filename, &data),
            Event::PowerDevices(devices) => replace_rows(&self.power_devices, devices.iter().map(power_item).collect()),
            Event::Updates(updates) => replace_rows(&self.updates, updates.iter().map(update_item).collect()),
            Event::ConsoleHistory(lines) => {
                let items: Vec<ConsoleItem> = lines.iter().map(console_item).collect();
                let skip = items.len().saturating_sub(CONSOLE_LIMIT);
                self.console.set_vec(items.into_iter().skip(skip).collect::<Vec<_>>());
            }
            Event::ConsoleLine(line) => {
                if self.console.row_count() >= CONSOLE_LIMIT {
                    self.console.remove(0);
                }
                self.console.push(console_item(&line));
            }
            Event::HostNetwork(interfaces) => {
                replace_rows(&self.host_interfaces, interfaces.iter().map(host_interface_item).collect());
            }
            Event::CommandFailed { command, message } => self.show_notice(&format!("{command}: {message}"), true),
        }
    }

    pub fn show_notice(&self, text: &str, is_error: bool) {
        let Some(window) = self.window.upgrade() else { return };
        let printer = window.global::<Printer>();
        printer.set_notice(text.into());
        printer.set_notice_is_error(is_error);
        let weak = self.window.clone();
        self.notice_timer.start(TimerMode::SingleShot, NOTICE_DURATION, move || {
            if let Some(window) = weak.upgrade() {
                window.global::<Printer>().set_notice(SharedString::new());
            }
        });
    }

    pub fn request_files(&self) {
        if let Some(window) = self.window.upgrade() {
            window.global::<Printer>().set_files_loading(true);
        }
        self.service.query(Query::Files);
    }

    fn show_files(&self, files: Vec<FileEntry>) {
        let missing: Vec<String> = {
            let cache = self.cache.borrow();
            files
                .iter()
                .take(FILE_DETAILS_LIMIT)
                .filter(|file| !cache.metadata.contains_key(&file.path))
                .map(|file| file.path.clone())
                .collect()
        };
        let rows: Vec<FileItem> = {
            let cache = self.cache.borrow();
            files
                .iter()
                .map(|file| file_item(file, cache.metadata.get(&file.path), cache.thumbnails.get(&file.path)))
                .collect()
        };
        self.cache.borrow_mut().files = files;
        self.files.set_vec(rows);
        if let Some(window) = self.window.upgrade() {
            window.global::<Printer>().set_files_loading(false);
        }
        for filename in missing {
            self.service.query(Query::FileDetails { filename });
        }
    }

    fn store_metadata(&self, metadata: FileMetadata) {
        let filename = metadata.filename.clone();
        let is_job = {
            let mut cache = self.cache.borrow_mut();
            cache.metadata.insert(filename.clone(), metadata);
            cache.snapshot.status.job.filename == filename
        };
        self.update_file_row(&filename);
        if is_job {
            self.refresh_job_texts();
        }
    }

    fn store_thumbnail(&self, filename: &str, data: &[u8]) {
        let Some(image) = thumbnail::decode(data) else {
            log::info!("thumbnail of {filename} cannot be decoded");
            return;
        };
        self.cache.borrow_mut().thumbnails.insert(filename.to_owned(), image);
        self.update_file_row(filename);
        self.show_job_thumbnail();
    }

    fn update_file_row(&self, filename: &str) {
        let cache = self.cache.borrow();
        let Some(index) = cache.files.iter().position(|file| file.path == filename) else { return };
        let row = file_item(&cache.files[index], cache.metadata.get(filename), cache.thumbnails.get(filename));
        drop(cache);
        if index < self.files.row_count() {
            self.files.set_row_data(index, row);
        }
    }

    fn show_job_thumbnail(&self) {
        let Some(window) = self.window.upgrade() else { return };
        let printer = window.global::<Printer>();
        let cache = self.cache.borrow();
        match cache.thumbnails.get(&cache.snapshot.status.job.filename) {
            Some(image) => {
                printer.set_job_thumbnail(image.clone());
                printer.set_has_job_thumbnail(true);
            }
            None => printer.set_has_job_thumbnail(false),
        }
    }
}

fn connection_name(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Connecting => "connecting",
        ConnectionState::Connected => "connected",
        ConnectionState::Disconnected => "disconnected",
    }
}

/// Replaces the rows only on changes so Slint does not rebuild needlessly.
pub(crate) fn replace_rows<T: Clone + PartialEq + 'static>(model: &VecModel<T>, rows: Vec<T>) {
    let unchanged = model.row_count() == rows.len()
        && rows.iter().enumerate().all(|(index, row)| model.row_data(index).as_ref() == Some(row));
    if !unchanged {
        model.set_vec(rows);
    }
}

fn heater_item(heater: &Heater) -> HeaterItem {
    HeaterItem {
        object: heater.object.as_str().into(),
        label: heater.label.as_str().into(),
        temperature: heater.temperature as f32,
        target: heater.target as f32,
        power: heater.power as f32,
        can_raise: heater.can_raise(),
    }
}

fn file_item(file: &FileEntry, metadata: Option<&FileMetadata>, thumbnail: Option<&Image>) -> FileItem {
    FileItem {
        path: file.path.as_str().into(),
        name: format::job_name(&file.path).into(),
        detail: view::file_detail(file.modified, file.size, metadata).into(),
        thumbnail: thumbnail.cloned().unwrap_or_default(),
        has_thumbnail: thumbnail.is_some(),
    }
}

fn console_item(line: &ConsoleLine) -> ConsoleItem {
    ConsoleItem {
        text: line.message.as_str().into(),
        time: format::unix_clock(line.time).into(),
        is_command: line.kind == ConsoleKind::Command,
        is_error: line.message.starts_with("!!"),
    }
}

fn power_item(device: &PowerDevice) -> PowerItem {
    PowerItem { name: device.device.as_str().into(), on: device.is_on(), locked: device.locked_while_printing }
}

fn host_interface_item(interface: &HostInterface) -> HostInterfaceItem {
    HostInterfaceItem {
        name: interface.name.as_str().into(),
        wireless: view::is_wireless_interface(&interface.name),
        addresses: interface.addresses.join(", ").into(),
        mac: interface.mac_address.as_str().into(),
    }
}

fn update_item(update: &UpdateInfo) -> UpdateItem {
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
