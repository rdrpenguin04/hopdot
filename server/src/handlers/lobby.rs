use std::{collections::HashMap, sync::RwLock};

use actix_web::{
    HttpRequest, Responder,
    web::{self, Data},
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_ws::{AggregatedMessage, CloseCode};
use futures_util::StreamExt as _;
use log::error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::session::extract_session;

#[derive(Deserialize, Serialize)]
enum RoomVisibility {
    Public {
        min_elo: usize,
        max_elo: usize,
    },
    Private,
}

#[derive(Deserialize)]
enum LobbyCommand {
    Create {
        width: usize,
        height: usize,
        player_count: usize,
        room_visibility: RoomVisibility,
    },
    Join(Uuid),
}

#[derive(Serialize)]
struct RoomInfo {
    vis: RoomVisibility,
    width: usize,
    height: usize,
    player_max: usize,
    players: Vec<Uuid>,
}

#[derive(Serialize)]
enum LobbyUpdate {
    CurrentLobby(HashMap<Uuid, RoomInfo>),
    NewRoom {
        id: Uuid,
        #[serde(flatten)]
        room: RoomInfo,
    },
    RoomClosed {
        id: Uuid,
    },
}

#[derive(Default)]
pub struct LobbyData {
    rooms: HashMap<Uuid, RoomInfo>,
}

pub async fn ws(
    auth: BearerAuth,
    req: HttpRequest,
    body: web::Payload,
    lobby_data: Data<RwLock<LobbyData>>,
) -> actix_web::Result<impl Responder> {
    let session = extract_session(auth.token())?;

    let (response, mut ws_session, msg_stream) = actix_ws::handle(&req, body)?;
    let mut msg_stream = msg_stream.aggregate_continuations();

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                AggregatedMessage::Ping(bytes) => {
                    if ws_session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                AggregatedMessage::Text(text) => {
                    let command = match serde_json::from_str(&text) {
                        Ok(x) => x,
                        Err(e) => {
                            error!("WebSocket error: {e}");
                            let _ = ws_session.close(Some(CloseCode::Invalid.into())).await;
                            return;
                        }
                    };
                    match command {
                        LobbyCommand::Create { .. } => todo!(),
                        LobbyCommand::Join(_) => todo!(),
                    }
                }
                x => {
                    dbg!(x);
                }
            }
        }

        let _ = ws_session.close(None).await;
    });

    Ok(response)
}
