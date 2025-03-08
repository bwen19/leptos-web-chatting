use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{FromRequestParts, State};
use axum::{async_trait, http::request::Parts, response::IntoResponse};
use axum_extra::extract::cookie::CookieJar;

use super::client::Client;
use crate::state::AppState;
use common::{CookieManager, Error, Event, Session, User};

// ==================== // WsGuard // ==================== //

pub struct WsGuard(pub User);

#[async_trait]
impl FromRequestParts<AppState> for WsGuard {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookie_jar = CookieJar::from_request_parts(parts, state).await?;
        let (user_id, session) = CookieManager::extract_auth(cookie_jar)?;

        let user = Session::verify(user_id, session, false, &state.store).await?;
        Ok(WsGuard(user))
    }
}

// ==================== // ws_handler // ==================== //

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    WsGuard(user): WsGuard,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, state, user))
}

async fn websocket(socket: WebSocket, state: AppState, user: User) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = broadcast::channel(128);

    let client = Client::new(user.id, state);
    if client.register(tx).await.is_err() {
        return;
    }

    // this task will receive message from broadcast channel and send to client
    let mut send_task = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(20));
        loop {
            tokio::select! {
                data = rx.recv() => {
                    if let Ok(msg) = data {
                        if sender.send(msg.into()).await.is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                _ = interval.tick() => {
                    if sender.send(Message::Ping(Vec::new())).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // this task will receive client message and process
    let mut recv_task = {
        let client = client.clone();

        tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                match msg {
                    Message::Binary(text) => {
                        if let Ok(event) = serde_json::from_slice::<Event>(&text) {
                            if client.process(event).await.is_err() {
                                break;
                            }
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        })
    };

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    // Disconnecting the channels
    client.unregister();
    log::debug!("socket disconnected {}", client);
}
