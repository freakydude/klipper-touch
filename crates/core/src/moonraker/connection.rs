//! WebSocket connection to Moonraker: matches responses to their requests and forwards notifications.

use super::rpc::{self, Incoming, RpcError};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;

type Pending = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value, RpcError>>>>>;

#[derive(Debug, Error)]
pub enum CallError {
    #[error(transparent)]
    Rpc(#[from] RpcError),
    #[error("connection to Moonraker closed")]
    Closed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Notification {
    pub method: String,
    pub params: Value,
}

/// Handle to an open connection; cheap to clone.
#[derive(Clone)]
pub struct Connection {
    outgoing: mpsc::UnboundedSender<String>,
    pending: Pending,
    next_id: Arc<AtomicU64>,
}

impl Connection {
    /// Opens the connection; notifications arrive via the returned receiver,
    /// which ends as soon as the connection closes.
    pub async fn open(
        url: &str,
    ) -> Result<(Self, mpsc::UnboundedReceiver<Notification>), tokio_tungstenite::tungstenite::Error> {
        let (socket, _) = tokio_tungstenite::connect_async(url).await?;
        let (mut sink, mut stream) = socket.split();
        let (outgoing, mut outgoing_rx) = mpsc::unbounded_channel::<String>();
        let (notifications, notifications_rx) = mpsc::unbounded_channel();
        let pending: Pending = Arc::default();

        tokio::spawn(async move {
            while let Some(text) = outgoing_rx.recv().await {
                if sink.send(Message::text(text)).await.is_err() {
                    break;
                }
            }
            let _ = sink.close().await;
        });

        let reader_pending = Arc::clone(&pending);
        tokio::spawn(async move {
            while let Some(Ok(message)) = stream.next().await {
                match message {
                    Message::Text(text) => dispatch(text.as_str(), &reader_pending, &notifications),
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            lock(&reader_pending).clear();
        });

        let connection = Self { outgoing, pending, next_id: Arc::new(AtomicU64::new(1)) };
        Ok((connection, notifications_rx))
    }

    pub async fn call(&self, method: &str, params: Option<Value>) -> Result<Value, CallError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (reply, response) = oneshot::channel();
        lock(&self.pending).insert(id, reply);
        if self.outgoing.send(rpc::encode_request(id, method, params)).is_err() {
            lock(&self.pending).remove(&id);
            return Err(CallError::Closed);
        }
        response.await.map_err(|_| CallError::Closed)?.map_err(CallError::from)
    }

    pub fn is_closed(&self) -> bool {
        self.outgoing.is_closed()
    }
}

fn dispatch(text: &str, pending: &Pending, notifications: &mpsc::UnboundedSender<Notification>) {
    match rpc::decode(text) {
        Some(Incoming::Response { id, result }) => {
            if let Some(reply) = lock(pending).remove(&id) {
                let _ = reply.send(result);
            }
        }
        Some(Incoming::Notification { method, params }) => {
            let _ = notifications.send(Notification { method, params });
        }
        None => log::debug!("ignoring unreadable message from Moonraker"),
    }
}

fn lock(pending: &Pending) -> std::sync::MutexGuard<'_, HashMap<u64, oneshot::Sender<Result<Value, RpcError>>>> {
    pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}
