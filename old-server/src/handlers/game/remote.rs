use actix_ws::{MessageStream, Session};
use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::{EventReader, EventWriter},
    system::{Commands, Query},
};
use common::proto::{BoardInfo, CellState, Dimension, GameInfo, GamePacket, bincode_config};
use futures::channel::mpsc::{self, Receiver, Sender, UnboundedReceiver, UnboundedSender};
use futures::prelude::*;

use crate::handlers::game::{ConnectionStatus, GameBlob, GameRef, StatusEvent, TurnAction};

#[derive(Component)]
pub struct ActorWsHandler(UnboundedSender<GamePacket>, Receiver<GamePacket>);

pub struct ActorWsSession {
    msg_stream: MessageStream,
    session: Session,
    psend: UnboundedReceiver<GamePacket>,
    precv: Sender<GamePacket>,
}

impl ActorWsSession {
    pub async fn run(mut self) {
        loop {
            todo!()
        }
    }
}

impl ActorWsHandler {
    pub fn create_handler_twin(
        msg_stream: MessageStream,
        session: Session,
    ) -> (ActorWsHandler, ActorWsSession) {
        let (psend_sink, psend_source) = mpsc::unbounded();
        let (precv_sink, prcev_source) = mpsc::channel(128);

        (
            ActorWsHandler(psend_sink, prcev_source),
            ActorWsSession {
                msg_stream,
                session,
                psend: psend_source,
                precv: precv_sink,
            },
        )
    }
}

pub fn update_client_packets(
    actors: Query<(Entity, &GameRef, &mut ActorWsHandler)>,
    game: Query<&GameBlob>,
    mut act: EventWriter<TurnAction>,
    mut conn_status: EventWriter<ConnectionStatus>,
) {
    for (e, gref, mut ws) in actors {
        let Ok(game) = game.get(gref.0) else { continue };

        while let Ok(v) = ws.1.try_next() {
            match v {
                None => {
                    conn_status.write(ConnectionStatus::Leave {
                        game: gref.0,
                        conn: e,
                    });
                }
                Some(GamePacket::CMoveSelected(pos)) => {
                    if gref.1.is_some_and(|v| v == game.turn_player()) {
                        act.write(TurnAction::Move(e, pos));
                    } else {
                        let _ = ws.0.unbounded_send(GamePacket::SMoveResult(
                            pos,
                            common::proto::MoveResult::Refused,
                        ));
                    }
                }
                Some(GamePacket::CProposal(prop)) => {
                    if gref.1.is_some() {
                        act.write(TurnAction::Proposal(e, prop));
                    }
                }
                Some(GamePacket::CCancelProposal(prop)) => {
                    if gref.1.is_some() {
                        act.write(TurnAction::CancelProposal(e, prop));
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn send_server_packets(
    actors: Query<(&GameRef, &ActorWsHandler)>,
    game: Query<&GameBlob>,
    mut game_event: EventReader<StatusEvent>,
) {
    for event in game_event.read() {
        let gent = event.game_entity();
        let game = game.get(gent).unwrap();

        for (gref, actor) in &actors {
            if gref.0 != gent {
                continue;
            }
            match event {
                StatusEvent::BeginGame(_) => {
                    let game_info = GameInfo {
                        board_size: Dimension {
                            width: game.grid().width(),
                            height: game.grid().height(),
                        },
                        players: game
                            .players
                            .iter()
                            .map(|(player, _)| player.clone())
                            .collect(),
                        which_player: gref.1,
                    };
                    let board_info = BoardInfo(
                        game.grid()
                            .grid_inner()
                            .iter()
                            .map(|v| CellState::from_grid_cell(v))
                            .collect(),
                    );

                    actor.0.unbounded_send(GamePacket::SGameInfo(game_info));
                    actor.0.unbounded_send(GamePacket::SBoardInfo(board_info));
                }
                StatusEvent::FinishGame(_, status) => {
                    let _ = actor.0.unbounded_send(GamePacket::SGameStatus(*status));
                }
                StatusEvent::BeginTurn(_, turn_player, _) => {
                    let _ = actor
                        .0
                        .unbounded_send(GamePacket::SUpdateTurn(*turn_player));
                }
                StatusEvent::Move(_, pos, move_result) => {
                    let _ = actor
                        .0
                        .unbounded_send(GamePacket::SMoveResult(*pos, *move_result));
                }
            }
        }
    }
}
