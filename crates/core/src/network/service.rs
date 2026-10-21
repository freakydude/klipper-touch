//! Background task that owns the network backend, applies commands and publishes the state.

use super::simulated::Simulated;
use super::{Availability, NetworkCommand, NetworkError, NetworkNotice, NetworkState};
use std::time::Duration;
use tokio::sync::{mpsc, watch};

/// NetworkManager does not push changes to us; the state is read again this often.
const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

/// Which implementation manages the networks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkBackend {
    /// NetworkManager of this host (Linux only).
    System,
    /// In-memory networks for development and screenshots.
    Simulated,
}

/// Handle for the UI; cheap to clone.
#[derive(Clone)]
pub struct NetworkService {
    commands: mpsc::UnboundedSender<NetworkCommand>,
}

pub struct NetworkChannels {
    pub states: watch::Receiver<NetworkState>,
    pub notices: mpsc::UnboundedReceiver<NetworkNotice>,
}

impl NetworkService {
    pub fn start(runtime: &tokio::runtime::Handle, backend: NetworkBackend) -> (Self, NetworkChannels) {
        let (commands, commands_rx) = mpsc::unbounded_channel();
        let (state_tx, states) = watch::channel(NetworkState::default());
        let (notice_tx, notices) = mpsc::unbounded_channel();
        runtime.spawn(run(backend, commands_rx, Outputs { state: state_tx, notices: notice_tx }));
        (Self { commands }, NetworkChannels { states, notices })
    }

    pub fn send(&self, command: NetworkCommand) {
        let _ = self.commands.send(command);
    }
}

enum Backend {
    #[cfg(target_os = "linux")]
    NetworkManager(super::networkmanager::NetworkManager),
    Simulated(Simulated),
}

impl Backend {
    async fn open(choice: NetworkBackend) -> Result<Self, NetworkError> {
        match choice {
            NetworkBackend::Simulated => Ok(Self::Simulated(Simulated::default())),
            #[cfg(target_os = "linux")]
            NetworkBackend::System => super::networkmanager::NetworkManager::connect().await.map(Self::NetworkManager),
            #[cfg(not(target_os = "linux"))]
            NetworkBackend::System => {
                Err(NetworkError::Unavailable("only supported on Linux with NetworkManager".to_owned()))
            }
        }
    }

    async fn read(&self) -> Result<NetworkState, NetworkError> {
        match self {
            #[cfg(target_os = "linux")]
            Self::NetworkManager(manager) => manager.read().await,
            Self::Simulated(simulated) => Ok(simulated.read()),
        }
    }

    async fn apply(&mut self, command: &NetworkCommand) -> Result<(), NetworkError> {
        match self {
            #[cfg(target_os = "linux")]
            Self::NetworkManager(manager) => match command {
                NetworkCommand::Refresh => Ok(()),
                NetworkCommand::Scan => manager.scan().await,
                NetworkCommand::Connect { ssid, password, hidden } => {
                    manager.connect_wifi(ssid, password.as_deref(), *hidden).await
                }
                NetworkCommand::Forget { ssid } => manager.forget(ssid).await,
                NetworkCommand::SetWifiEnabled(enabled) => manager.set_wifi_enabled(*enabled).await,
                NetworkCommand::SetWired { interface, connected } => manager.set_wired(interface, *connected).await,
            },
            Self::Simulated(simulated) => match command {
                NetworkCommand::Refresh | NetworkCommand::Scan => Ok(()),
                NetworkCommand::Connect { ssid, password, hidden } => {
                    simulated.connect(ssid, password.as_deref(), *hidden).await
                }
                NetworkCommand::Forget { ssid } => simulated.forget(ssid),
                NetworkCommand::SetWifiEnabled(enabled) => {
                    simulated.set_wifi_enabled(*enabled);
                    Ok(())
                }
                NetworkCommand::SetWired { interface, connected } => simulated.set_wired(interface, *connected),
            },
        }
    }
}

struct Outputs {
    state: watch::Sender<NetworkState>,
    notices: mpsc::UnboundedSender<NetworkNotice>,
}

impl Outputs {
    fn notice(&self, message: String, error: bool) {
        let _ = self.notices.send(NetworkNotice { message, error });
    }

    fn unavailable(&self, error: &NetworkError) {
        let reason = match error {
            NetworkError::Unavailable(reason) => reason.clone(),
            other => other.to_string(),
        };
        self.state
            .send_replace(NetworkState { availability: Availability::Unavailable(reason), ..NetworkState::default() });
    }

    async fn refresh(&self, backend: &Backend) {
        match backend.read().await {
            Ok(state) => {
                self.state.send_replace(state);
            }
            Err(error) => {
                log::warn!("reading network state failed: {error}");
                self.unavailable(&error);
            }
        }
    }

    async fn execute(&self, backend: &mut Backend, command: NetworkCommand) {
        if let Some(busy) = busy_text(&command) {
            self.state.send_modify(|state| state.busy = Some(busy));
        }
        match backend.apply(&command).await {
            Ok(()) => {
                if let Some(message) = success_text(&command) {
                    self.notice(message, false);
                }
            }
            Err(error) => {
                log::info!("network command {} failed: {error}", describe(&command));
                self.notice(error.to_string(), true);
            }
        }
        self.refresh(backend).await;
    }
}

async fn run(choice: NetworkBackend, mut commands: mpsc::UnboundedReceiver<NetworkCommand>, outputs: Outputs) {
    let mut backend = match Backend::open(choice).await {
        Ok(backend) => backend,
        Err(error) => {
            log::info!("{error}");
            outputs.unavailable(&error);
            // Keep the channel open so the UI can send commands without effect.
            while commands.recv().await.is_some() {}
            return;
        }
    };
    let mut refresh = tokio::time::interval(REFRESH_INTERVAL);
    loop {
        tokio::select! {
            command = commands.recv() => match command {
                Some(command) => outputs.execute(&mut backend, command).await,
                None => return,
            },
            _ = refresh.tick() => outputs.refresh(&backend).await,
        }
    }
}

fn busy_text(command: &NetworkCommand) -> Option<String> {
    match command {
        NetworkCommand::Connect { ssid, .. } => Some(format!("Connecting to {ssid}")),
        NetworkCommand::Scan => Some("Searching for networks".to_owned()),
        _ => None,
    }
}

fn success_text(command: &NetworkCommand) -> Option<String> {
    match command {
        NetworkCommand::Refresh | NetworkCommand::Scan => None,
        NetworkCommand::Connect { ssid, .. } => Some(format!("Connected to {ssid}")),
        NetworkCommand::Forget { ssid } => Some(format!("Removed {ssid}")),
        NetworkCommand::SetWifiEnabled(true) => Some("Wi-Fi switched on".to_owned()),
        NetworkCommand::SetWifiEnabled(false) => Some("Wi-Fi switched off".to_owned()),
        NetworkCommand::SetWired { interface, connected: true } => Some(format!("{interface} connected")),
        NetworkCommand::SetWired { interface, connected: false } => Some(format!("{interface} disconnected")),
    }
}

/// Description for the log without secrets.
fn describe(command: &NetworkCommand) -> String {
    match command {
        NetworkCommand::Connect { ssid, hidden, .. } => format!("connect to '{ssid}' (hidden: {hidden})"),
        other => format!("{other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn simulated_service_publishes_state_and_notices() {
        let runtime = tokio::runtime::Handle::current();
        let (service, mut channels) = NetworkService::start(&runtime, NetworkBackend::Simulated);
        channels.states.wait_for(|state| state.availability == Availability::Managed).await.unwrap();

        service.send(NetworkCommand::Forget { ssid: "Garage".into() });
        let notice = channels.notices.recv().await.unwrap();
        assert_eq!(notice, NetworkNotice { message: "Removed Garage".into(), error: false });
        channels.states.wait_for(|state| !state.known.contains(&"Garage".to_owned())).await.unwrap();

        service.send(NetworkCommand::Forget { ssid: "Unknown".into() });
        assert!(channels.notices.recv().await.unwrap().error);
    }

    #[test]
    fn log_description_hides_the_password() {
        let command =
            NetworkCommand::Connect { ssid: "Home".into(), password: Some("secret-pw".into()), hidden: false };
        assert!(!describe(&command).contains("secret-pw"));
    }

    #[cfg(not(target_os = "linux"))]
    #[tokio::test]
    async fn system_backend_is_unavailable_outside_linux() {
        let (_service, mut channels) =
            NetworkService::start(&tokio::runtime::Handle::current(), NetworkBackend::System);
        let state =
            channels.states.wait_for(|state| state.availability != Availability::Checking).await.unwrap().clone();
        assert!(matches!(state.availability, Availability::Unavailable(_)));
    }
}
