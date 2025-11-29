use std::num::NonZeroU8;

use bevy::ecs::{component::Component, entity::Entity, event::Event};
use common::{
    grid::Grid,
    proto::{GameStatus, MoveResult, Player, Pos, ProposalType},
};
use futures::future::BoxFuture;
use uuid::Uuid;

pub mod remote;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Event)]
pub enum TurnAction {
    Move(Entity, Pos),
    Proposal(Entity, ProposalType),
    CancelProposal(Entity, ProposalType),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Component)]
pub struct GameRef(Entity, Option<NonZeroU8>);

#[derive(Clone, Debug, Hash, PartialEq, Eq, Event)]
pub enum StatusEvent {
    BeginGame(Entity),
    FinishGame(Entity, GameStatus),
    BeginTurn(Entity, NonZeroU8, u32),
    Move(Entity, Pos, MoveResult),
}

impl StatusEvent {
    pub fn game_entity(&self) -> Entity {
        match self {
            Self::BeginGame(ent) => *ent,
            Self::FinishGame(ent, _) => *ent,
            Self::BeginTurn(ent, _, _) => *ent,
            Self::Move(ent, _, _) => *ent,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Component)]
pub struct ActorId(Uuid);

#[derive(Clone, Debug, Hash, PartialEq, Eq, Event)]
pub enum ConnectionStatus {
    Join { game: Entity, conn: Entity },
    Leave { game: Entity, conn: Entity },
}

#[derive(Clone, Debug, Component)]
pub struct GameBlob {
    players: Vec<(Player, Entity)>,
    board: Grid,
    current_player: NonZeroU8,
}

impl GameBlob {
    pub fn grid(&self) -> &Grid {
        &self.board
    }
    pub fn players(&self) -> &[(Player, Entity)] {
        &self.players
    }

    pub fn turn_player(&self) -> NonZeroU8 {
        self.current_player
    }

    pub fn turn_player_entity(&self) -> Entity {
        self.players[self.current_player.get() as usize - 1].1
    }
}
