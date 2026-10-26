//! Background service: keeps the connection to Moonraker, maintains the state and executes commands.
//!
//! The UI talks to the printer exclusively through [`PrinterService`]. State arrives as a
//! [`Snapshot`] over a `watch` channel, one-off results as [`Event`].

use crate::commands::Command;
use crate::files::{self, FileEntry, FileMetadata, LastPrint};
use crate::moonraker::{CallError, Connection, Endpoint, Notification};
use crate::printer::{self, Capabilities, KlippyState, ObjectStore, PendingTargets, PrinterStatus, ServerComponents};
use crate::system::{self, ConsoleKind, ConsoleLine, HostInterface, PowerDevice, UpdateInfo};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, watch};

const RECONNECT_DELAY: Duration = Duration::from_secs(2);
const KLIPPY_POLL_INTERVAL: Duration = Duration::from_secs(2);
const QUERY_TIMEOUT: Duration = Duration::from_secs(15);
const CONSOLE_HISTORY: u32 = 100;
/// Newest jobs read from the job history; files last printed before that show the slicer estimate.
const PRINT_HISTORY_LIMIT: u32 = 300;
const CLIENT_NAME: &str = "klipper-touch";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    Connecting,
    Connected,
    Disconnected,
}

/// Current overall state for display.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    pub endpoint: String,
    pub connection: ConnectionState,
    pub status: PrinterStatus,
    pub capabilities: Capabilities,
}

/// One-off requests for data that is not part of the running subscription.
#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    Files,
    /// Metadata and then the thumbnail of a file.
    FileDetails {
        filename: String,
    },
    /// The last completed print of each file, from the Moonraker job history.
    PrintHistory,
    PowerDevices,
    Updates,
    ConsoleHistory,
    /// Network interfaces of the printer host.
    HostNetwork,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Files(Vec<FileEntry>),
    FileMetadata(Box<FileMetadata>),
    PrintHistory(HashMap<String, LastPrint>),
    Thumbnail { filename: String, data: Vec<u8> },
    PowerDevices(Vec<PowerDevice>),
    Updates(Vec<UpdateInfo>),
    ConsoleHistory(Vec<ConsoleLine>),
    ConsoleLine(ConsoleLine),
    HostNetwork(Vec<HostInterface>),
    CommandFailed { command: String, message: String },
}

enum Request {
    Command(Command),
    Query(Query),
    Connect(Endpoint),
}

/// Handle for the UI; cheap to clone.
#[derive(Clone)]
pub struct PrinterService {
    requests: mpsc::UnboundedSender<Request>,
}

pub struct ServiceChannels {
    pub snapshots: watch::Receiver<Snapshot>,
    pub events: mpsc::UnboundedReceiver<Event>,
}

impl PrinterService {
    /// Starts the service on the given Tokio runtime.
    pub fn start(runtime: &tokio::runtime::Handle, endpoint: Endpoint) -> (Self, ServiceChannels) {
        let (requests, requests_rx) = mpsc::unbounded_channel();
        let (snapshot_tx, snapshots) = watch::channel(Snapshot { endpoint: endpoint.display(), ..Snapshot::default() });
        let (events_tx, events) = mpsc::unbounded_channel();
        runtime.spawn(run(endpoint, requests_rx, snapshot_tx, events_tx));
        (Self { requests }, ServiceChannels { snapshots, events })
    }

    pub fn send(&self, command: Command) {
        let _ = self.requests.send(Request::Command(command));
    }

    pub fn query(&self, query: Query) {
        let _ = self.requests.send(Request::Query(query));
    }

    /// Reconnects, e.g. after the address was changed in the settings.
    pub fn connect_to(&self, endpoint: Endpoint) {
        let _ = self.requests.send(Request::Connect(endpoint));
    }
}

enum SessionEnd {
    ConnectionLost,
    SwitchTo(Endpoint),
    Shutdown,
}

struct Outputs {
    snapshot: watch::Sender<Snapshot>,
    events: mpsc::UnboundedSender<Event>,
}

impl Outputs {
    fn event(&self, event: Event) {
        let _ = self.events.send(event);
    }
}

async fn run(
    mut endpoint: Endpoint,
    mut requests: mpsc::UnboundedReceiver<Request>,
    snapshot: watch::Sender<Snapshot>,
    events: mpsc::UnboundedSender<Event>,
) {
    let outputs = Outputs { snapshot, events };
    let http = reqwest::Client::new();
    loop {
        publish_connection(&outputs, &endpoint, ConnectionState::Connecting);
        let end = match Connection::open(&endpoint.websocket_url()).await {
            Ok((connection, notifications)) => {
                let session = Session::new(connection, http.clone(), endpoint.clone(), &outputs);
                session.run(notifications, &mut requests).await
            }
            Err(error) => {
                log::info!("cannot connect to {}: {error}", endpoint.websocket_url());
                SessionEnd::ConnectionLost
            }
        };
        match end {
            SessionEnd::Shutdown => return,
            SessionEnd::SwitchTo(next) => endpoint = next,
            SessionEnd::ConnectionLost => {
                publish_connection(&outputs, &endpoint, ConnectionState::Disconnected);
                match wait_for_retry(&mut requests, &outputs).await {
                    Retry::Same => {}
                    Retry::SwitchTo(next) => endpoint = next,
                    Retry::Shutdown => return,
                }
            }
        }
    }
}

fn publish_connection(outputs: &Outputs, endpoint: &Endpoint, connection: ConnectionState) {
    outputs.snapshot.send_replace(Snapshot { endpoint: endpoint.display(), connection, ..Snapshot::default() });
}

enum Retry {
    Same,
    SwitchTo(Endpoint),
    Shutdown,
}

/// Waits before the next connection attempt; commands fail immediately during this time.
async fn wait_for_retry(requests: &mut mpsc::UnboundedReceiver<Request>, outputs: &Outputs) -> Retry {
    let delay = tokio::time::sleep(RECONNECT_DELAY);
    tokio::pin!(delay);
    loop {
        tokio::select! {
            () = &mut delay => return Retry::Same,
            request = requests.recv() => match request {
                None => return Retry::Shutdown,
                Some(Request::Connect(endpoint)) => return Retry::SwitchTo(endpoint),
                Some(Request::Command(command)) => outputs.event(Event::CommandFailed {
                    command: command.describe(),
                    message: "not connected to Moonraker".to_owned(),
                }),
                Some(Request::Query(_)) => {}
            }
        }
    }
}

struct Session<'a> {
    connection: Connection,
    http: reqwest::Client,
    endpoint: Endpoint,
    outputs: &'a Outputs,
    store: ObjectStore,
    capabilities: Capabilities,
    components: ServerComponents,
    klippy_ready: bool,
    job_filename: String,
    /// Targets sent but not yet reported back; the snapshot shows them.
    pending_targets: PendingTargets,
}

impl<'a> Session<'a> {
    fn new(connection: Connection, http: reqwest::Client, endpoint: Endpoint, outputs: &'a Outputs) -> Self {
        Self {
            connection,
            http,
            endpoint,
            outputs,
            store: ObjectStore::default(),
            capabilities: Capabilities::default(),
            components: ServerComponents::default(),
            klippy_ready: false,
            job_filename: String::new(),
            pending_targets: PendingTargets::default(),
        }
    }

    async fn run(
        mut self,
        mut notifications: mpsc::UnboundedReceiver<Notification>,
        requests: &mut mpsc::UnboundedReceiver<Request>,
    ) -> SessionEnd {
        if let Err(error) = self.introduce().await {
            log::warn!("Moonraker handshake failed: {error}");
            return SessionEnd::ConnectionLost;
        }
        self.publish();
        let mut klippy_poll = tokio::time::interval(KLIPPY_POLL_INTERVAL);
        loop {
            tokio::select! {
                notification = notifications.recv() => match notification {
                    Some(notification) => self.handle_notification(notification).await,
                    None => return SessionEnd::ConnectionLost,
                },
                request = requests.recv() => match request {
                    None => return SessionEnd::Shutdown,
                    Some(Request::Connect(endpoint)) if endpoint != self.endpoint => return SessionEnd::SwitchTo(endpoint),
                    Some(Request::Connect(_)) => {}
                    Some(Request::Command(command)) => self.execute(command),
                    Some(Request::Query(query)) => self.answer(query),
                },
                _ = klippy_poll.tick(), if !self.klippy_ready => {
                    if let Err(error) = self.refresh_klippy().await {
                        log::debug!("klippy not ready yet: {error}");
                    }
                }
            }
        }
    }

    /// Identifies with Moonraker and performs the first sync of the Klippy state.
    async fn introduce(&mut self) -> Result<(), CallError> {
        let identity = json!({
            "client_name": CLIENT_NAME,
            "version": crate::VERSION,
            "type": "display",
            "url": env!("CARGO_PKG_REPOSITORY"),
        });
        self.connection.call("server.connection.identify", Some(identity)).await?;
        self.refresh_klippy().await
    }

    async fn refresh_klippy(&mut self) -> Result<(), CallError> {
        let info = system::parse_server_info(&self.connection.call("server.info", None).await?);
        self.components = ServerComponents::from_names(info.components.iter().map(String::as_str));
        let state =
            if info.klippy_connected { KlippyState::parse(&info.klippy_state) } else { KlippyState::Disconnected };
        if state == KlippyState::Ready {
            self.subscribe().await?;
        } else {
            self.mark_klippy(state, "").await;
        }
        Ok(())
    }

    /// Determines capabilities and subscribes to the matching objects.
    async fn subscribe(&mut self) -> Result<(), CallError> {
        let listing = self.connection.call("printer.objects.list", None).await?;
        let available: Vec<String> = listing
            .get("objects")
            .and_then(Value::as_array)
            .map(|names| names.iter().filter_map(Value::as_str).map(str::to_owned).collect())
            .unwrap_or_default();
        let mut capabilities = Capabilities::from_objects(available.iter().map(String::as_str));
        capabilities.server = self.components.clone();
        let parameters = printer::subscription(&available, &capabilities);
        let result = self.connection.call("printer.objects.subscribe", Some(parameters)).await?;
        self.capabilities = capabilities;
        self.store.replace(result.get("status").unwrap_or(&Value::Null));
        self.klippy_ready = true;
        self.publish();
        Ok(())
    }

    /// Sets the Klippy state without a subscription, e.g. on shutdown or while Klipper starts.
    async fn mark_klippy(&mut self, state: KlippyState, message: &str) {
        self.klippy_ready = false;
        if state == KlippyState::Disconnected {
            self.store.clear();
        }
        let message = if message.is_empty() { self.klippy_message().await } else { message.to_owned() };
        self.store.merge(&json!({ "webhooks": { "state": state.as_str(), "state_message": message } }));
        self.publish();
    }

    /// Klipper reports the reason for errors and shutdown via `printer.info`.
    async fn klippy_message(&self) -> String {
        match self.connection.call("printer.info", None).await {
            Ok(info) => info.get("state_message").and_then(Value::as_str).unwrap_or_default().trim().to_owned(),
            Err(_) => String::new(),
        }
    }

    async fn handle_notification(&mut self, notification: Notification) {
        match notification.method.as_str() {
            "notify_status_update" => {
                if let Some(changes) = notification.params.get(0) {
                    self.store.merge(changes);
                    self.publish();
                }
            }
            "notify_klippy_ready" => {
                if let Err(error) = self.subscribe().await {
                    log::warn!("subscription after klippy ready failed: {error}");
                }
            }
            "notify_klippy_shutdown" => self.mark_klippy(KlippyState::Shutdown, "").await,
            "notify_klippy_disconnected" => self.mark_klippy(KlippyState::Disconnected, "").await,
            "notify_gcode_response" => {
                if let Some(message) = notification.params.get(0).and_then(Value::as_str) {
                    self.outputs.event(Event::ConsoleLine(console_line(ConsoleKind::Response, message)));
                }
            }
            "notify_power_changed" => self.answer(Query::PowerDevices),
            "notify_update_refreshed" => {
                self.outputs.event(Event::Updates(system::parse_update_status(
                    notification.params.get(0).unwrap_or(&Value::Null),
                )));
            }
            _ => {}
        }
    }

    /// The state with the heater targets that were sent but not yet reported back.
    fn current_status(&mut self) -> PrinterStatus {
        let mut status = PrinterStatus::from_store(&self.store);
        self.pending_targets.apply(&mut status.heaters, Instant::now());
        status
    }

    /// Publishes the state and loads the print file's metadata when the job changes.
    fn publish(&mut self) {
        let status = self.current_status();
        if status.job.filename != self.job_filename {
            self.job_filename.clone_from(&status.job.filename);
            if !self.job_filename.is_empty() {
                self.answer(Query::FileDetails { filename: self.job_filename.clone() });
            }
        }
        self.outputs.snapshot.send_replace(Snapshot {
            endpoint: self.endpoint.display(),
            connection: ConnectionState::Connected,
            status,
            capabilities: self.capabilities.clone(),
        });
    }

    /// Carries out a command; an intent first becomes the commands this printer needs.
    fn execute(&mut self, command: Command) {
        let status = self.current_status();
        let description = command.describe();
        let commands = match command.resolve(&self.capabilities, &status) {
            Ok(commands) => commands,
            Err(message) => {
                self.outputs.event(Event::CommandFailed { command: description, message });
                return;
            }
        };
        let mut targets_changed = false;
        for command in commands {
            targets_changed |= self.dispatch(command, &status);
        }
        if targets_changed {
            self.publish();
        }
    }

    /// Sends one concrete command; true if it set a heater target, which the snapshot shows at once.
    fn dispatch(&mut self, command: Command, status: &PrinterStatus) -> bool {
        if command.interrupts_print() && status.job.state.is_active() {
            self.outputs.event(Event::CommandFailed {
                command: command.describe(),
                message: "not possible while a print is running".to_owned(),
            });
            return false;
        }
        let Some(call) = command.to_call(&status.motion) else { return false };
        if let Command::RunGcode { script } = &command {
            self.outputs.event(Event::ConsoleLine(console_line(ConsoleKind::Command, script)));
        }
        let target_set = match &command {
            Command::SetHeaterTarget { heater, target } => {
                self.pending_targets.request(heater, target.max(0.0), Instant::now());
                true
            }
            _ => false,
        };
        let events = self.outputs.events.clone();
        let failed = move |error: CallError| {
            let message = match error {
                CallError::Rpc(rpc) => rpc.message,
                CallError::Closed => "connection to Moonraker closed".to_owned(),
            };
            let _ = events.send(Event::CommandFailed { command: command.describe(), message });
        };
        // Sent here, in the order of the commands; a task per command could overtake an earlier one.
        match self.connection.start_call(call.method, call.params) {
            Ok(pending) => {
                tokio::spawn(async move {
                    if let Err(error) = pending.response().await {
                        failed(error);
                    }
                });
            }
            Err(error) => failed(error),
        }
        target_set
    }

    fn answer(&self, query: Query) {
        if query == Query::PrintHistory && !self.components.history {
            log::debug!("no job history: Moonraker has no history component");
            return;
        }
        let connection = self.connection.clone();
        let http = self.http.clone();
        let endpoint = self.endpoint.clone();
        let events = self.outputs.events.clone();
        tokio::spawn(async move {
            let fetched = tokio::time::timeout(QUERY_TIMEOUT, fetch(&connection, &http, &endpoint, &query)).await;
            match fetched {
                Ok(Ok(results)) => results.into_iter().for_each(|event| {
                    let _ = events.send(event);
                }),
                Ok(Err(error)) => log::info!("query {query:?} failed: {error}"),
                Err(_) => log::info!("query {query:?} timed out"),
            }
        });
    }
}

async fn fetch(
    connection: &Connection,
    http: &reqwest::Client,
    endpoint: &Endpoint,
    query: &Query,
) -> Result<Vec<Event>, String> {
    let call = |method: &'static str, params: Option<Value>| async move {
        connection.call(method, params).await.map_err(|error| error.to_string())
    };
    let events = match query {
        Query::Files => {
            let listing = call("server.files.list", Some(json!({ "root": "gcodes" }))).await?;
            vec![Event::Files(files::parse_file_list(&listing))]
        }
        Query::FileDetails { filename } => {
            let result = call("server.files.metadata", Some(json!({ "filename": filename }))).await?;
            let metadata = files::parse_metadata(&result).ok_or("unreadable metadata")?;
            let mut events = Vec::new();
            if let Some(path) = metadata.thumbnail_path() {
                match download(http, &endpoint.http_url(&path)).await {
                    Ok(data) => events.push(Event::Thumbnail { filename: filename.clone(), data }),
                    Err(error) => log::info!("thumbnail of {filename} unavailable: {error}"),
                }
            }
            events.insert(0, Event::FileMetadata(Box::new(FileMetadata { filename: filename.clone(), ..metadata })));
            events
        }
        Query::PrintHistory => {
            let history = call("server.history.list", Some(json!({ "limit": PRINT_HISTORY_LIMIT }))).await?;
            vec![Event::PrintHistory(files::parse_last_completed_prints(&history))]
        }
        Query::PowerDevices => {
            vec![Event::PowerDevices(system::parse_power_devices(&call("machine.device_power.devices", None).await?))]
        }
        Query::Updates => {
            vec![Event::Updates(system::parse_update_status(&call("machine.update.status", None).await?))]
        }
        Query::ConsoleHistory => vec![Event::ConsoleHistory(system::parse_gcode_store(
            &call("server.gcode_store", Some(json!({ "count": CONSOLE_HISTORY }))).await?,
        ))],
        Query::HostNetwork => {
            vec![Event::HostNetwork(system::parse_host_network(&call("machine.system_info", None).await?))]
        }
    };
    Ok(events)
}

async fn download(http: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    let response = http.get(url).send().await.map_err(|error| error.to_string())?;
    let response = response.error_for_status().map_err(|error| error.to_string())?;
    Ok(response.bytes().await.map_err(|error| error.to_string())?.to_vec())
}

fn console_line(kind: ConsoleKind, message: &str) -> ConsoleLine {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0.0, |elapsed| elapsed.as_secs_f64());
    ConsoleLine { kind, message: message.to_owned(), time }
}
