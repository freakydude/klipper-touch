//! Transfers core state and events into the Slint globals. Runs on the UI thread only.

use crate::status_log::{self, info_display_time, StatusLog};
use crate::{format, thumbnail, view};
use crate::{
    AppWindow, ConsoleItem, FilamentSensorItem, FileItem, HeaterItem, HostInterfaceItem, MacroItem, MacroParamItem,
    MessageItem, MmuGateItem, MmuSensorItem, Network, PowerItem, PrintObjectItem, Printer, SensorItem, SpoolItem,
    UpdateItem,
};
use chrono::Local;
use freakymate_core::estimates;
use freakymate_core::files::{FileCatalog, FileEntry, FileMetadata, LastPrint};
use freakymate_core::printer::{
    is_mmu_klipper_sensor, offered_macros, Capabilities, FilamentHandling, Heater, MacroDetails, MacroForm, MmuStatus,
};
use freakymate_core::spoolman::{self, Lane, Spool, SpoolmanState};
use freakymate_core::system::{ConsoleKind, ConsoleLine, HostInterface, PowerDevice, UpdateInfo};
use freakymate_core::{Command, ConnectionState, Event, PrinterService, Query, Snapshot};
use slint::{ComponentHandle, Image, Model, ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;

const CONSOLE_LIMIT: usize = 300;
/// Status bar and its flyout. Shared with the timer that hides informations, hence the `Rc`.
struct StatusView {
    window: slint::Weak<AppWindow>,
    log: RefCell<StatusLog>,
    messages: Rc<VecModel<MessageItem>>,
}

impl StatusView {
    /// Puts the log on screen: the flyout list and the message the status bar shows.
    fn refresh(&self) {
        let log = self.log.borrow();
        let mut top = 0.0;
        let items: Vec<MessageItem> = log
            .messages()
            .map(|message| {
                let height = status_log::row_height(&message.text);
                let item = MessageItem {
                    text: SharedString::from(message.text.as_str()),
                    time: format::unix_clock(message.time).into(),
                    is_error: message.is_error,
                    top,
                    height,
                };
                top += height + status_log::ROW_GAP;
                item
            })
            .collect();
        replace_rows(&self.messages, items);
        let Some(window) = self.window.upgrade() else { return };
        let printer = window.global::<Printer>();
        printer.set_messages_height(top);
        let badge = log.badge();
        printer.set_status_message(badge.map(|message| SharedString::from(message.headline())).unwrap_or_default());
        printer.set_status_message_error(badge.is_some_and(|message| message.is_error));
        printer.set_status_message_visible(badge.is_some());
    }
}

fn now_seconds() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0.0, |elapsed| elapsed.as_secs_f64())
}

type MacrosLoaded = Box<dyn Fn(&[String])>;

#[derive(Default)]
struct Cache {
    snapshot: Snapshot,
    catalog: FileCatalog,
    thumbnails: HashMap<String, Image>,
    spoolman: SpoolmanState,
    /// Spoolman has answered at least once; before that it is not known to be offline.
    spoolman_known: bool,
    /// The spools of the picker, most recently used first.
    spool_list: Vec<Spool>,
}

pub struct Presenter {
    window: slint::Weak<AppWindow>,
    service: PrinterService,
    cache: RefCell<Cache>,
    /// Last logged source of each derived value, so that a change is logged once.
    origins: RefCell<HashMap<&'static str, &'static str>>,
    heaters: Rc<VecModel<HeaterItem>>,
    sensors: Rc<VecModel<SensorItem>>,
    filament_sensors: Rc<VecModel<FilamentSensorItem>>,
    print_objects: Rc<VecModel<PrintObjectItem>>,
    mmu_gates: Rc<VecModel<MmuGateItem>>,
    mmu_sensors: Rc<VecModel<MmuSensorItem>>,
    macros: Rc<VecModel<SharedString>>,
    macro_items: Rc<VecModel<MacroItem>>,
    all_macros: RefCell<Vec<String>>,
    hidden_macros: RefCell<Vec<String>>,
    macros_loaded: RefCell<Option<MacrosLoaded>>,
    macro_details: RefCell<BTreeMap<String, MacroDetails>>,
    macro_form: Rc<VecModel<MacroParamItem>>,
    macro_form_state: RefCell<MacroForm>,
    files: Rc<VecModel<FileItem>>,
    spools: Rc<VecModel<SpoolItem>>,
    console: Rc<VecModel<ConsoleItem>>,
    power_devices: Rc<VecModel<PowerItem>>,
    updates: Rc<VecModel<UpdateItem>>,
    host_interfaces: Rc<VecModel<HostInterfaceItem>>,
    status: Rc<StatusView>,
    status_timer: Timer,
}

impl Presenter {
    pub fn new(window: &AppWindow, service: PrinterService) -> Rc<Self> {
        let presenter = Rc::new(Self {
            window: window.as_weak(),
            service,
            cache: RefCell::default(),
            origins: RefCell::default(),
            heaters: Rc::default(),
            sensors: Rc::default(),
            filament_sensors: Rc::default(),
            print_objects: Rc::default(),
            mmu_gates: Rc::default(),
            mmu_sensors: Rc::default(),
            macros: Rc::default(),
            macro_items: Rc::default(),
            all_macros: RefCell::default(),
            hidden_macros: RefCell::default(),
            macros_loaded: RefCell::default(),
            macro_details: RefCell::default(),
            macro_form: Rc::default(),
            macro_form_state: RefCell::default(),
            files: Rc::default(),
            spools: Rc::default(),
            console: Rc::default(),
            power_devices: Rc::default(),
            updates: Rc::default(),
            host_interfaces: Rc::default(),
            status: Rc::new(StatusView { window: window.as_weak(), log: RefCell::default(), messages: Rc::default() }),
            status_timer: Timer::default(),
        });
        let printer = window.global::<Printer>();
        printer.set_heaters(ModelRc::from(presenter.heaters.clone()));
        printer.set_sensors(ModelRc::from(presenter.sensors.clone()));
        printer.set_filament_sensors(ModelRc::from(presenter.filament_sensors.clone()));
        printer.set_print_objects(ModelRc::from(presenter.print_objects.clone()));
        printer.set_mmu_gates(ModelRc::from(presenter.mmu_gates.clone()));
        printer.set_mmu_sensors(ModelRc::from(presenter.mmu_sensors.clone()));
        printer.set_macros(ModelRc::from(presenter.macros.clone()));
        printer.set_macro_items(ModelRc::from(presenter.macro_items.clone()));
        printer.set_macro_form(ModelRc::from(presenter.macro_form.clone()));
        printer.set_messages(ModelRc::from(presenter.status.messages.clone()));
        printer.set_files(ModelRc::from(presenter.files.clone()));
        printer.set_spools(ModelRc::from(presenter.spools.clone()));
        printer.set_console(ModelRc::from(presenter.console.clone()));
        printer.set_power_devices(ModelRc::from(presenter.power_devices.clone()));
        printer.set_updates(ModelRc::from(presenter.updates.clone()));
        window.global::<Network>().set_host_interfaces(ModelRc::from(presenter.host_interfaces.clone()));
        presenter
    }

    /// Connects the service's snapshots and events to the UI (asynchronously in the Slint event loop).
    pub fn follow(self: &Rc<Self>, channels: freakymate_core::ServiceChannels) -> Result<(), slint::EventLoopError> {
        let freakymate_core::ServiceChannels { mut snapshots, mut events } = channels;
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

    pub fn apply_snapshot(&self, snapshot: &Snapshot) {
        let Some(window) = self.window.upgrade() else { return };
        let printer = window.global::<Printer>();
        let status = &snapshot.status;
        let connected = snapshot.connection == ConnectionState::Connected;
        let ready = connected && status.klippy_state == freakymate_core::printer::KlippyState::Ready;

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
        let flow = estimates::volumetric_flow(motion.live_extruder_velocity, status.extruder.filament_diameter);
        self.note_origin("volumetric flow", Some(flow.origin));
        printer.set_live_flow(flow.value as f32);
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

        let lanes = self.lanes(snapshot);
        let active_spool = spoolman::active_spool(&lanes).cloned();
        replace_rows(
            &self.heaters,
            status.heaters.iter().map(|heater| heater_item(heater, active_spool.as_ref())).collect(),
        );
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
        let has_mmu = status.mmu.as_ref().is_some_and(|mmu| mmu.enabled && mmu.is_supported());
        replace_rows(
            &self.filament_sensors,
            status
                .filament_sensors
                .iter()
                // With Happy Hare its gate and buffer sensors belong to the MMU tab; this card keeps the common ones.
                .filter(|sensor| !(has_mmu && is_mmu_klipper_sensor(&sensor.object)))
                .map(|sensor| FilamentSensorItem {
                    label: sensor.label.as_str().into(),
                    enabled: sensor.enabled,
                    detected: sensor.detected,
                })
                .collect(),
        );
        replace_rows(
            &self.print_objects,
            status
                .print_objects
                .iter()
                .map(|object| PrintObjectItem { name: object.name.as_str().into(), excluded: object.excluded })
                .collect(),
        );

        let capabilities = &snapshot.capabilities;
        printer.set_can_quad_gantry_level(capabilities.quad_gantry_level);
        printer.set_can_z_tilt(capabilities.z_tilt);
        printer.set_has_probe(capabilities.probe);
        let handling = capabilities.filament_handling();
        printer.set_has_load_macro(matches!(handling, FilamentHandling::Macros { load: true, .. }));
        printer.set_has_unload_macro(matches!(handling, FilamentHandling::Macros { unload: true, .. }));
        printer.set_has_power(capabilities.server.power);
        printer.set_has_update_manager(capabilities.server.update_manager);
        self.set_macro_details(snapshot.status.macro_details.clone());
        self.set_macros(&capabilities.macros, snapshot.connection == ConnectionState::Connected);
        self.set_mmu(status.mmu.as_ref(), &lanes);
        self.set_spool_views(&printer, snapshot, active_spool.as_ref());

        let previous_job =
            std::mem::replace(&mut self.cache.borrow_mut().snapshot, snapshot.clone()).status.job.filename;
        if previous_job != status.job.filename {
            self.show_job_thumbnail();
        }
        self.refresh_job_texts();
    }

    /// Logs which source a derived value comes from when it changes; `None` means no source has a value.
    fn note_origin(&self, value: &'static str, origin: Option<&'static str>) {
        let origin = origin.unwrap_or("none");
        if self.origins.borrow_mut().insert(value, origin) != Some(origin) {
            log::debug!("{value}: {origin}");
        }
    }

    /// Time-related texts; also called every second by the clock timer.
    pub fn refresh_job_texts(&self) {
        let Some(window) = self.window.upgrade() else { return };
        let printer = window.global::<Printer>();
        let now = Local::now();
        let cache = self.cache.borrow();
        let job = &cache.snapshot.status.job;
        let texts =
            view::job_texts(job, cache.catalog.metadata(&job.filename), cache.catalog.last_print(&job.filename), now);
        printer.set_clock(format::clock(now).into());
        printer.set_job_name(texts.name.into());
        printer.set_elapsed(texts.elapsed.into());
        printer.set_remaining(texts.remaining.into());
        printer.set_eta(texts.eta.into());
        printer.set_layers(texts.layers.into());
        printer.set_filament(texts.filament.into());
        if let Some(origin) = texts.expectation_origin {
            self.note_origin("print time expectation", Some(origin));
        }
    }

    pub fn handle_event(&self, event: Event) {
        match event {
            Event::Files(files) => self.show_files(files),
            Event::FileMetadata(metadata) => self.store_metadata(*metadata),
            Event::PrintHistory(last_prints) => self.store_last_prints(last_prints),
            Event::Thumbnail { filename, data } => self.store_thumbnail(&filename, &data),
            Event::PowerDevices(devices) => replace_rows(&self.power_devices, devices.iter().map(power_item).collect()),
            Event::Updates(updates) => replace_rows(&self.updates, updates.iter().map(update_item).collect()),
            Event::ConsoleHistory(lines) => {
                let items: Vec<ConsoleItem> = lines.iter().flat_map(console_items).collect();
                let skip = items.len().saturating_sub(CONSOLE_LIMIT);
                self.console.set_vec(items.into_iter().skip(skip).collect::<Vec<_>>());
            }
            Event::ConsoleLine(line) => {
                for item in console_items(&line) {
                    if self.console.row_count() >= CONSOLE_LIMIT {
                        self.console.remove(0);
                    }
                    self.console.push(item);
                }
                if line.kind == ConsoleKind::Response {
                    self.show_console_response(&line);
                }
            }
            Event::Spoolman(state) => self.store_spoolman(*state),
            Event::SpoolList(spools) => self.store_spool_list(spools),
            Event::SpoolmanChanged => {
                self.service.query(Query::Spoolman);
                self.service.query(Query::SpoolList);
            }
            Event::HostNetwork(interfaces) => {
                replace_rows(&self.host_interfaces, interfaces.iter().map(host_interface_item).collect());
            }
            Event::CommandFailed { command, message } => {
                let shown = format!("{command}: {message}");
                let id = self.status.log.borrow_mut().add_command_failure(&shown, &message, now_seconds());
                self.show_message(id, &shown, true);
            }
        }
    }

    /// Applies the hidden macros from the settings.
    pub fn set_hidden_macros(&self, hidden: Vec<String>) {
        *self.hidden_macros.borrow_mut() = hidden;
        self.refresh_macro_views();
    }

    /// Called with the printer's macros after a connection, so the settings can forget vanished ones.
    pub fn on_macros_loaded(&self, handler: impl Fn(&[String]) + 'static) {
        *self.macros_loaded.borrow_mut() = Some(Box::new(handler));
    }

    /// Stores the printer's macros. An empty list while disconnected must not count as "all macros vanished".
    /// Happy Hare on the Filament page. Only version 4.1 or newer is shown; an older one gets a notice instead.
    fn set_mmu(&self, mmu: Option<&MmuStatus>, lanes: &[Lane]) {
        let Some(window) = self.window.upgrade() else { return };
        let printer = window.global::<Printer>();
        let enabled = mmu.filter(|mmu| mmu.enabled);
        let supported = enabled.filter(|mmu| mmu.is_supported());
        printer.set_has_mmu(supported.is_some());
        printer.set_mmu_unsupported(enabled.is_some() && supported.is_none());
        let Some(mmu) = supported else {
            replace_rows(&self.mmu_gates, Vec::new());
            replace_rows(&self.mmu_sensors, Vec::new());
            return;
        };

        let gates = mmu
            .gates
            .iter()
            .map(|gate| {
                let spool =
                    lanes.iter().find(|lane| lane.gate == Some(gate.index)).and_then(|lane| lane.spool.as_ref());
                // A multi-colour spool shows all its colours; otherwise the gate has the one colour Happy Hare reports.
                let gate_color: Vec<[u8; 3]> = gate.color.into_iter().collect();
                let spool_colors = spool.map(|spool| spool.colors.as_slice()).filter(|colors| colors.len() > 1);
                let bar = color_bar(spool_colors.unwrap_or(&gate_color));
                MmuGateItem {
                    spool_text: spool.map(view::spool_remaining).unwrap_or_default().into(),
                    spool_low: spool.is_some_and(Spool::is_low),
                    spool_id: gate.spool_id.map_or(-1, |id| id as i32),
                    index: gate.index as i32,
                    availability: view::mmu_availability(gate.availability).into(),
                    name: gate.name.as_str().into(),
                    material: gate.material.as_str().into(),
                    color: bar.0,
                    color_2: bar.1,
                    color_3: bar.2,
                    color_count: bar.3,
                    has_color: gate.color.is_some(),
                    tool: gate.tool.map_or(-1, |tool| tool as i32),
                    selected: usize::try_from(mmu.gate) == Ok(gate.index),
                }
            })
            .collect();
        replace_rows(&self.mmu_gates, gates);
        let has_buffer = mmu.sync_feedback.is_some();
        let sensors = view::mmu_chip_sensors(&mmu.sensors, has_buffer)
            .into_iter()
            .map(|sensor| MmuSensorItem {
                label: view::mmu_sensor_label(&sensor.name).into(),
                state: view::mmu_sensor_state(sensor.triggered).into(),
            })
            .collect();
        replace_rows(&self.mmu_sensors, sensors);

        printer.set_mmu_filament(view::mmu_filament(mmu.filament).into());
        printer.set_mmu_action(mmu.action.as_str().into());
        printer.set_mmu_gate(mmu.gate.max(-1));
        printer.set_mmu_tool(mmu.tool.max(-1));
        printer.set_mmu_paused(mmu.paused);
        printer.set_mmu_locked(mmu.locked);
        printer.set_mmu_reason(mmu.reason_for_pause.as_str().into());
        printer.set_mmu_homed(mmu.homed);
        printer.set_mmu_has_buffer(has_buffer);
        let feedback = mmu.sync_feedback.unwrap_or_default();
        printer.set_mmu_buffer_state(view::mmu_buffer_state(feedback.state).into());
        printer.set_mmu_buffer_bias(feedback.bias as f32);
        let flowguard = mmu.flowguard.clone().unwrap_or_default();
        printer.set_mmu_flowguard_state(view::mmu_flowguard_state(flowguard.state()).into());
        let detail: Vec<&str> = [flowguard.trigger.as_str(), flowguard.reason.as_str()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect();
        printer.set_mmu_flowguard_detail(detail.join(": ").into());
        self.note_origin("MMU flow rate", mmu.flow_rate.map(|rate| rate.origin));
        printer.set_mmu_flow_rate(mmu.flow_rate.map_or(-1.0, |rate| rate.value as f32));
    }

    fn set_macros(&self, macros: &[String], connected: bool) {
        if self.all_macros.borrow().as_slice() == macros {
            return;
        }
        *self.all_macros.borrow_mut() = macros.to_vec();
        self.refresh_macro_views();
        if connected && !macros.is_empty() {
            if let Some(handler) = self.macros_loaded.borrow().as_ref() {
                handler(macros);
            }
        }
    }

    fn refresh_macro_views(&self) {
        let offered =
            offered_macros(&self.all_macros.borrow(), &self.macro_details.borrow(), &self.hidden_macros.borrow());
        let visible: Vec<SharedString> =
            offered.iter().filter(|offer| offer.visible).map(|offer| SharedString::from(offer.name.as_str())).collect();
        replace_rows(&self.macros, visible);
        let items: Vec<MacroItem> = offered
            .iter()
            .map(|offer| MacroItem { name: SharedString::from(offer.name.as_str()), visible: offer.visible })
            .collect();
        replace_rows(&self.macro_items, items);
    }

    fn set_macro_details(&self, details: BTreeMap<String, MacroDetails>) {
        if *self.macro_details.borrow() == details {
            return;
        }
        *self.macro_details.borrow_mut() = details;
        self.refresh_macro_views();
    }

    /// Whether the macro takes parameters, so the form has to come before it runs.
    pub fn macro_has_params(&self, name: &str) -> bool {
        self.macro_details.borrow().get(&name.to_ascii_uppercase()).is_some_and(|details| !details.params.is_empty())
    }

    /// Fills the form with the parameters of a macro; empty values are left to Klipper's defaults.
    pub fn open_macro_form(&self, name: &str) {
        let form = MacroForm::open(name, self.macro_details.borrow().get(&name.to_ascii_uppercase()));
        let rows: Vec<MacroParamItem> = form
            .fields()
            .iter()
            .map(|field| MacroParamItem {
                name: SharedString::from(field.name.as_str()),
                placeholder: SharedString::from(field.placeholder.as_str()),
                value: SharedString::from(field.value.as_str()),
            })
            .collect();
        *self.macro_form_state.borrow_mut() = form;
        self.macro_form.set_vec(rows);
        self.show_macro_form_name();
    }

    /// Sets one parameter of the open form.
    pub fn set_macro_param(&self, index: usize, value: &str) {
        let result = self.macro_form_state.borrow_mut().set_value(index, value);
        match result {
            Ok(()) => {
                if let Some(mut row) = self.macro_form.row_data(index) {
                    row.value = SharedString::from(value);
                    self.macro_form.set_row_data(index, row);
                }
            }
            Err(error) => self.show_notice(&error.to_string(), true),
        }
    }

    /// The command for the open form, e.g. `PARK Z=20`.
    pub fn macro_form_script(&self) -> String {
        self.macro_form_state.borrow().script()
    }

    fn show_macro_form_name(&self) {
        let Some(window) = self.window.upgrade() else { return };
        window.global::<Printer>().set_macro_form_name(SharedString::from(self.macro_form_state.borrow().name()));
    }

    /// Empties the shown log only; the history on the printer stays and returns on the next load.
    pub fn clear_console(&self) {
        self.console.set_vec(Vec::new());
    }

    pub fn show_notice(&self, text: &str, is_error: bool) {
        let id = self.status.log.borrow_mut().add(text, is_error, now_seconds());
        self.show_message(id, text, is_error);
    }

    /// Shows a message in the status bar. Informations leave it again after a while, errors stay until seen.
    fn show_message(&self, id: u64, text: &str, is_error: bool) {
        if !is_error {
            let status = Rc::clone(&self.status);
            self.status_timer.start(TimerMode::SingleShot, info_display_time(text), move || {
                status.log.borrow_mut().hide_badge(id);
                status.refresh();
            });
        }
        self.status.refresh();
    }

    /// A `!!` (error) or `//` (information) response of Klipper. Other responses stay in the console.
    /// Informations are left out while a print runs, when Klipper sends many of them.
    fn show_console_response(&self, line: &ConsoleLine) {
        let is_error = line.is_error();
        if !is_error && !(line.is_information() && !self.is_printing()) {
            return;
        }
        let text = line.text();
        let added = self.status.log.borrow_mut().add_response(&text, is_error, now_seconds());
        if let Some(id) = added {
            self.show_message(id, &text, is_error);
        }
    }

    fn is_printing(&self) -> bool {
        self.cache.borrow().snapshot.status.job.state.is_active()
    }

    pub fn mark_messages_seen(&self) {
        self.status.log.borrow_mut().mark_seen();
        self.status.refresh();
    }

    pub fn clear_messages(&self) {
        self.status.log.borrow_mut().clear();
        self.status.refresh();
    }

    /// The lanes of the printer: a gate each with MMU, else the one active spool.
    fn lanes(&self, snapshot: &Snapshot) -> Vec<Lane> {
        let mmu = snapshot.status.mmu.as_ref().filter(|mmu| mmu.is_supported());
        spoolman::lanes(mmu, &self.cache.borrow().spoolman)
    }

    /// The spool in use on the page of the print and on the temperature page, and Spoolman's state.
    fn set_spool_views(&self, printer: &Printer, snapshot: &Snapshot, active: Option<&Spool>) {
        let configured = snapshot.capabilities.server.spoolman;
        let (offline, pending) = {
            let cache = self.cache.borrow();
            (configured && cache.spoolman_known && !cache.spoolman.connected, cache.spoolman.pending_reports)
        };
        printer.set_has_spoolman(configured);
        printer.set_spoolman_offline(offline);
        printer.set_spoolman_pending(pending as i32);
        printer.set_has_spool(active.is_some());
        printer.set_spool_title(active.map(Spool::title).unwrap_or_default().into());
        printer.set_spool_detail(active.map(view::spool_detail).unwrap_or_default().into());
        printer.set_spool_weight(active.map(view::spool_remaining).unwrap_or_default().into());
        let bar = color_bar(active.map(|spool| spool.colors.as_slice()).unwrap_or_default());
        printer.set_spool_color(bar.0);
        printer.set_spool_color_2(bar.1);
        printer.set_spool_color_3(bar.2);
        printer.set_spool_color_count(bar.3);
        printer.set_spool_low(active.is_some_and(Spool::is_low));
        let preset = active.and_then(view::spool_preset_text);
        printer.set_has_spool_preset(preset.is_some());
        printer.set_spool_preset_text(preset.unwrap_or_default().into());
    }

    fn store_spoolman(&self, state: SpoolmanState) {
        {
            let mut cache = self.cache.borrow_mut();
            cache.catalog.store_spools(state.spools.values().cloned());
            cache.spoolman = state;
            cache.spoolman_known = true;
        }
        let snapshot = self.cache.borrow().snapshot.clone();
        self.apply_snapshot(&snapshot);
        self.show_spool_rows();
        self.show_file_rows();
    }

    fn store_spool_list(&self, spools: Vec<Spool>) {
        {
            let mut cache = self.cache.borrow_mut();
            cache.catalog.store_spools(spools.iter().cloned());
            cache.spool_list = spools;
        }
        self.show_spool_rows();
        self.show_file_rows();
    }

    fn show_spool_rows(&self) {
        let snapshot = self.cache.borrow().snapshot.clone();
        let lanes = self.lanes(&snapshot);
        let rows = spool_rows(&self.cache.borrow().spool_list, &lanes);
        replace_rows(&self.spools, rows);
        if let Some(window) = self.window.upgrade() {
            window.global::<Printer>().set_spools_loading(false);
        }
    }

    /// Asks for the spools of the picker, and for the state, which the picker shows the places from.
    pub fn load_spools(&self) {
        if self.spools.row_count() == 0 {
            if let Some(window) = self.window.upgrade() {
                window.global::<Printer>().set_spools_loading(true);
            }
        }
        self.service.query(Query::Spoolman);
        self.service.query(Query::SpoolList);
    }

    /// Heats to what the spool in use recommends; the bed keeps its target if Spoolman has no temperature for it.
    pub fn apply_spool_preset(&self) {
        let snapshot = self.snapshot();
        let lanes = self.lanes(&snapshot);
        let Some(spool) = spoolman::active_spool(&lanes) else { return };
        let Some(extruder) = spool.extruder_temp else { return };
        let bed =
            spool.bed_temp.or_else(|| snapshot.status.heater("heater_bed").map(|heater| heater.target)).unwrap_or(0.0);
        self.service.send(Command::ApplyPreset { extruder, bed });
    }

    /// What differs between the file and the spools, as lines for the confirmation to start it; empty if nothing.
    pub fn print_hints_text(&self, path: &str) -> String {
        let snapshot = self.snapshot();
        let lanes = self.lanes(&snapshot);
        let cache = self.cache.borrow();
        let Some(metadata) = cache.catalog.metadata(path) else { return String::new() };
        let tool_to_gate = snapshot.status.mmu.as_ref().map(|mmu| mmu.tool_to_gate.clone()).unwrap_or_default();
        let show_tool = lanes.iter().any(|lane| lane.gate.is_some());
        let lines: Vec<String> = spoolman::print_hints(metadata, &lanes, &tool_to_gate)
            .iter()
            .map(|hint| format!("• {}", view::print_hint_text(hint, show_tool)))
            .collect();
        if lines.is_empty() {
            String::new()
        } else {
            format!("\n\n{}", lines.join("\n"))
        }
    }

    pub fn request_files(&self) {
        if let Some(window) = self.window.upgrade() {
            window.global::<Printer>().set_files_loading(true);
        }
        self.service.query(Query::Files);
        self.service.query(Query::PrintHistory);
        self.service.query(Query::SpoolList);
    }

    fn show_files(&self, files: Vec<FileEntry>) {
        self.cache.borrow_mut().catalog.set_entries(files);
        self.show_file_rows();
        if let Some(window) = self.window.upgrade() {
            window.global::<Printer>().set_files_loading(false);
        }
        let missing = self.cache.borrow().catalog.needing_details();
        for filename in missing {
            self.service.query(Query::FileDetails { filename });
        }
    }

    fn show_file_rows(&self) {
        let rows: Vec<FileItem> = {
            let cache = self.cache.borrow();
            cache.catalog.entries().iter().map(|entry| file_row(&cache, entry)).collect()
        };
        self.files.set_vec(rows);
    }

    fn store_last_prints(&self, last_prints: HashMap<String, LastPrint>) {
        self.cache.borrow_mut().catalog.store_last_prints(last_prints);
        self.show_file_rows();
        self.refresh_job_texts();
    }

    fn store_metadata(&self, metadata: FileMetadata) {
        let filename = metadata.filename.clone();
        let is_job = {
            let mut cache = self.cache.borrow_mut();
            cache.catalog.store_metadata(metadata);
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
        let Some(index) = cache.catalog.position(filename) else { return };
        let row = file_row(&cache, &cache.catalog.entries()[index]);
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

/// The picker rows: spools in a lane first, then the most recently used, each with its place.
fn spool_rows(spools: &[Spool], lanes: &[Lane]) -> Vec<SpoolItem> {
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

fn color_of([red, green, blue]: [u8; 3]) -> slint::Color {
    slint::Color::from_rgb_u8(red, green, blue)
}

/// The colours of a bar: up to three, and how many there are.
fn color_bar(colors: &[[u8; 3]]) -> (slint::Color, slint::Color, slint::Color, i32) {
    let color = |index: usize| colors.get(index).copied().map_or_else(slint::Color::default, color_of);
    (color(0), color(1), color(2), colors.len().min(3) as i32)
}

/// The row of a heater; the hint says that the target is off from what the spool in use recommends.
fn heater_item(heater: &Heater, spool: Option<&Spool>) -> HeaterItem {
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

fn file_row(cache: &Cache, file: &FileEntry) -> FileItem {
    let info = cache.catalog.info(file);
    let facts = view::file_facts(&info);
    let thumbnail = cache.thumbnails.get(&file.path);
    FileItem {
        path: file.path.as_str().into(),
        name: format::job_name(&file.path).into(),
        modified: facts.modified.into(),
        material: facts.material.into(),
        nozzle: facts.nozzle.into(),
        time: facts.time.into(),
        extruder_temperature: facts.extruder_temperature.into(),
        bed_temperature: facts.bed_temperature.into(),
        chamber_temperature: facts.chamber_temperature.into(),
        thumbnail: thumbnail.cloned().unwrap_or_default(),
        has_thumbnail: thumbnail.is_some(),
        show_thumbnail: thumbnail.is_some() || info.thumbnail_expected(),
    }
}

/// One row per message line: the console list has fixed row heights, so a line break inside a row would overlap its neighbours.
/// Only the first row of a message shows the time.
fn console_items(line: &ConsoleLine) -> Vec<ConsoleItem> {
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

fn power_item(device: &PowerDevice) -> PowerItem {
    PowerItem { name: device.device.as_str().into(), on: device.is_on(), locked: device.locked_while_printing }
}

fn host_interface_item(interface: &HostInterface) -> HostInterfaceItem {
    HostInterfaceItem {
        name: interface.name.as_str().into(),
        wireless: interface.is_wireless(),
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
