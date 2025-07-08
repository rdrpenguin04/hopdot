use std::num::{NonZero, NonZeroU8};

use bincode::{Decode, Encode};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{grid::GridCell, version::Version};

#[derive(Encode, Decode, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct CellState(u8);

impl CellState {
    pub const fn owner(self) -> Option<NonZeroU8> {
        NonZeroU8::new(self.0 >> 5)
    }

    pub const fn count(self) -> NonZeroU8 {
        // SAFETY:
        // Range of the values produced is 1..5
        unsafe { NonZeroU8::new_unchecked((self.0 & 3) + 1) }
    }

    pub const fn from_grid_cell(cell: &GridCell) -> Self {
        if cell.dots > cell.capacity {
            panic!(
                "Sanity Check Failed: Specified cell is cascading, but we're trying to encode it for the protocol?"
            )
        }
        if cell.dots > 4 || cell.dots < 1 {
            panic!(
                "Invalid cell size - empty (hidden?) cell or extended capacity (are we playing 5DHWMVTT?)"
            )
        }
        if cell.owner > 7 {
            panic!("Ok, way too many players in this crowded game");
        }

        let owner = cell.owner as u8;
        let count = (cell.dots as u8) - 1;

        Self((owner << 5) | count)
    }
}

#[derive(Encode, Decode, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Encode, Decode, Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum PlayerKind {
    Player,
    Bot,
}

#[derive(Encode, Decode, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Player {
    pub color: Color,
    pub kind: PlayerKind,
    #[bincode(with_serde)]
    pub id: Uuid,
}

#[derive(Encode, Decode, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Dimension {
    pub width: u8,
    pub height: u8,
}

#[derive(Encode, Decode, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Pos {
    pub x: u8,
    pub y: u8,
}

#[derive(Encode, Decode, Clone, Debug, Hash, PartialEq, Eq)]
pub struct GameInfo {
    pub board_size: Dimension,
    pub players: Vec<Player>,
    pub which_player: Option<NonZeroU8>,
}

#[derive(Encode, Decode, Clone, Debug, Hash, PartialEq, Eq)]
pub struct BoardInfo(pub Vec<CellState>);

#[derive(Encode, Decode, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ProposalType {
    Resign,
    Draw,
}

#[derive(Encode, Decode, Clone, Debug, Hash, PartialEq, Eq)]
pub enum GamePacket {
    SHello(Version),
    CHello(Version),
    SKeepAlive(#[bincode(with_serde)] Timestamp),
    CKeepAlive(#[bincode(with_serde)] Timestamp),
    SGameInfo(GameInfo),
    SBoardInfo(BoardInfo),
    CMoveSelected(Pos),
    SMoveResult(Pos, MoveResult),
    SUpdateTurn(NonZeroU8),
    CProposal(ProposalType),
    SRemoteProposal(ProposalType),
    SProposalAccepted(ProposalType),
}

#[derive(Encode, Decode, Clone, Debug, Hash, PartialEq, Eq)]
pub enum MoveResult {
    Refused,
    CellUpdated,
    CellCascaded,
    GameFinished,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Timestamp(#[serde(with = "chrono::serde::ts_milliseconds")] pub chrono::DateTime<Utc>);
