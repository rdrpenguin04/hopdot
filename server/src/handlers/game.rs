use common::proto::{MoveResult, Pos, ProposalType};
use futures::future::BoxFuture;
use uuid::Uuid;

pub mod remote;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum TurnAction {
    Move(Pos),
    Proposal(Option<Pos>, ProposalType),
    AcceptProposal(Option<Pos>, ProposalType),
}

pub trait Actor: Send + Sync {
    fn id(&self) -> Uuid;
    fn await_move(
        &self,
        move_number: usize,
        player_number: u8,
    ) -> BoxFuture<'_, std::io::Result<TurnAction>>;

    fn send_result(&self, move_number: usize, res: MoveResult);
}

impl<'a> core::fmt::Debug for dyn Actor + 'a {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id().fmt(f)
    }
}
