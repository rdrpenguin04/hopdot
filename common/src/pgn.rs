use bincode::{Decode, Encode};

#[derive(Encode, Decode, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SingleMove {
    pub x: u8,
    pub y: u8,
    pub did_cascade: bool,
    pub status_type: Option<MoveStatusType>,
}

#[derive(Encode, Decode, Clone, Debug, Hash, PartialEq, Eq)]
pub enum MoveStatusType {}
