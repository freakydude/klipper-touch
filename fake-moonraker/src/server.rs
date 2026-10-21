//! HTTP and WebSocket server that behaves like Moonraker.

use crate::backend::Backend;
use crate::machine::MachineProfile;
use crate::session::Session;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;

const SIMULATION_STEP: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy)]
pub struct ServerConfig {
    /// Speeds up heating and print progress; 1.0 means real time.
    pub time_scale: f64,
    pub status_interval: Duration,
    pub profile: MachineProfile,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { time_scale: 1.0, status_interval: Duration::from_millis(250), profile: MachineProfile::default() }
    }
}

#[derive(Clone)]
struct AppState {
    backend: Arc<Backend>,
    status_interval: Duration,
    next_connection_id: Arc<AtomicU64>,
}

pub async fn serve(listener: TcpListener, config: ServerConfig) -> io::Result<()> {
    let backend = Arc::new(Backend::with_profile(config.time_scale, config.profile));
    let simulation = tokio::spawn(simulate(Arc::clone(&backend)));
    let state =
        AppState { backend, status_interval: config.status_interval, next_connection_id: Arc::new(AtomicU64::new(1)) };

    let result = axum::serve(listener, router(state)).await;
    simulation.abort();
    result
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/websocket", get(websocket))
        .route("/server/files/gcodes/{*path}", get(thumbnail))
        .with_state(state)
}

async fn simulate(backend: Arc<Backend>) {
    let mut ticker = tokio::time::interval(SIMULATION_STEP);
    let mut last_tick = Instant::now();
    loop {
        ticker.tick().await;
        let now = Instant::now();
        backend.advance_simulation(now - last_tick);
        last_tick = now;
    }
}

async fn websocket(upgrade: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    let connection_id = state.next_connection_id.fetch_add(1, Ordering::Relaxed);
    upgrade.on_upgrade(move |socket| run_session(socket, state, connection_id))
}

async fn run_session(mut socket: WebSocket, state: AppState, connection_id: u64) {
    let mut session = Session::new(connection_id);
    let mut status_ticker = tokio::time::interval(state.status_interval);
    let mut notifications = state.backend.notifications();

    loop {
        let outgoing = tokio::select! {
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Text(text))) => session.handle_text(&state.backend, text.as_str()),
                Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                Some(Ok(_)) => None,
            },
            _ = status_ticker.tick() => session.status_notification(&state.backend),
            notification = notifications.recv() => match notification {
                Ok(message) => Some(message),
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => None,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            },
        };

        if let Some(message) = outgoing {
            if socket.send(Message::Text(message.into())).await.is_err() {
                break;
            }
        }
    }
}

async fn thumbnail(State(state): State<AppState>, Path(path): Path<String>) -> Response {
    match state.backend.catalog().thumbnail_svg(&path) {
        Some(svg) => ([(header::CONTENT_TYPE, "image/svg+xml")], svg).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
