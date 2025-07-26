use std::{
    cell::RefCell,
    future::pending,
    sync::{Arc, atomic::AtomicUsize},
    thread::JoinHandle,
};

use futures::{StreamExt, channel::mpsc, lock::Mutex};
use uuid::Uuid;

use crate::handlers::game::{Actor, TurnAction};

const TURN_FINISHED: usize = 0;
const TURN_ACTIVE: usize = 1 << 16;
const TURN_INACTIVE: usize = 2 << 16;

pub struct RemoteActor {
    event_queue: Mutex<mpsc::Receiver<std::io::Result<TurnAction>>>,
    thread: JoinHandle<()>,
    turn_status: Arc<AtomicUsize>,
    user_id: Uuid,
    player_number: u8,
}

impl Actor for RemoteActor {
    fn id(&self) -> uuid::Uuid {
        self.user_id
    }

    fn send_result(&self, move_number: usize, res: common::proto::MoveResult) {
        self.turn_status
            .store(res as usize, std::sync::atomic::Ordering::Relaxed);
    }

    fn await_move(
        &self,
        move_number: usize,
        player_number: u8,
    ) -> futures::future::BoxFuture<'_, std::io::Result<TurnAction>> {
        let flag = ((self.player_number == player_number) as usize + 1) << 16;
        Box::pin(async move {
            self.turn_status
                .store(flag, std::sync::atomic::Ordering::Relaxed);
            let mut event_queue = self.event_queue.lock().await;
            event_queue.next().await.unwrap()
        })
    }
}
