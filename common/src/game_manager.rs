use std::collections::HashMap;
use std::num::NonZeroU8;
use std::sync::Exclusive;
use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

use crate::ai::Ai;
use crate::grid::Grid;
use crate::proto::*;

// TODO: rewrite Debug impl to use a cleaner Grid repr
#[derive(Debug)]
#[allow(dead_code)]
pub struct GameManager {
    board: Grid,
    player_list: Vec<Player>,
    spectators: Vec<Uuid>,
    bots: HashMap<Uuid, Box<dyn Ai>>,
    current_player: usize,
    current_turn_number: usize,
    listeners: Vec<Box<dyn EventListener>>,
    tx: Sender<Event>,
    rx: Exclusive<Receiver<Event>>,
}

impl GameManager {
    pub fn new(width: u8, height: u8, players: Vec<Player>) -> Self {
        let num_players = players
            .len()
            .try_into()
            .expect("We don't (even theoretically) support more than 255 players");
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            board: Grid::new(width, height, num_players),
            player_list: players,
            spectators: vec![],
            bots: HashMap::new(),
            current_player: 0,
            current_turn_number: 0,
            listeners: vec![],
            tx,
            rx: Exclusive::new(rx),
        }
    }

    pub fn register_listener<E: Into<Box<dyn EventListener>>>(&mut self, listener: E) {
        self.listeners.push(listener.into());
    }

    pub fn queue_event(&self, event: Event) {
        self.tx.send(event).unwrap();
    }

    pub fn process_all_events(&mut self) {
        while let Ok(e) = self.rx.get_mut().try_recv() {
            for listener in &self.listeners {
                listener.handle_event(e, self);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Event {
    MoveSelected(Pos),
    MoveResult(Pos, MoveResult),
    UpdateTurn(NonZeroU8),
    CancelProposal(ProposalType),
    Proposal(NonZeroU8, ProposalType),
    ProposalAccepted(ProposalType),
    ProposalRefused(ProposalType),
    PlayerStatus(NonZeroU8, PlayerStatus),
}

pub trait EventListener: Sync {
    fn handle_event(&self, event: Event, x: &GameManager);
    fn name(&self) -> &str;
}

impl<'a> core::fmt::Debug for dyn EventListener + 'a {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl<'a, E: EventListener + 'a> From<E> for Box<dyn EventListener + 'a> {
    fn from(value: E) -> Self {
        Box::new(value)
    }
}
