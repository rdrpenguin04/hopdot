use bevy::prelude::*;
use bevy_defer::{AsyncCommandsExtension as _, AsyncWorld, fetch};

#[cfg(not(target_family = "wasm"))]
use std::pin::Pin;
use std::time::Duration;

use async_channel::{Receiver, Sender};
use async_wsocket::{ConnectionMode, Message, Url, futures_util::SinkExt};

#[cfg(target_family = "wasm")]
use bevy::tasks::IoTaskPool;
#[cfg(not(target_family = "wasm"))]
use bevy_tokio_tasks::TokioTasksRuntime;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::{
    CellColor, Config, Dot, DotCell, GameAssets, GameOperation, GridTray, MainState, NeedNewBoard, PlayerConfigEntry, VisualGrid,
    anim::TargetUiOpacity,
    menu::MenuRadios,
    spawn_dot,
    ui_menu::{HostGameUiTree, JoinGameUiTree, support::fade_out_ui},
};

const SERVER_LIST: [(&str, &str); 3] = [
    ("Local server", "ws://localhost:8080"),
    ("LC HQ", "wss://hopdot.lcdev.xyz"),
    ("New LC HQ", "ws://hopdot.lc"),
];

#[derive(Clone, Debug, Default, Resource)]
pub struct ServerUrl {
    name: String,
    url: String,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<ServerUrl>()
        .add_systems(PreStartup, setup_channel)
        .add_systems(Startup, start_net_manager)
        .add_systems(Update, process_net_inbound)
        .add_systems(Last, maybe_shutdown)
        .add_message::<NetMessage>();
}

fn maybe_shutdown(mut app_exit_reader: MessageReader<AppExit>, s_s: Res<NetServerboundSender>) {
    for _ in app_exit_reader.read() {
        s_s.force_send(NetManagerMessage::Shutdown).unwrap();
    }
}

#[derive(Message)]
pub enum NetMessage {
    RoomCreated { code: String },
    RoomNotFound,
}

pub enum NetMessageClientbound {
    PingFailed(usize),
    PingSucceeded(usize),
    RoomCreated {
        code: String,
    },
    RoomReady {
        code: String,
        settings: GameSettings,
        server: ServerUrl,
    },
    RoomNotFound,

    GameStart {
        me: u8,
    },
    Move {
        player: u8,
        x: u8,
        y: u8,
    },

    #[cfg(not(target_family = "wasm"))]
    Spawn(Pin<Box<dyn Future<Output = ()> + Send + Sync>>),
}

#[derive(Deref, Resource)]
pub struct NetClientboundSender(Sender<NetMessageClientbound>);

#[derive(Deref, Resource)]
pub struct NetClientboundReceiver(Receiver<NetMessageClientbound>);

#[derive(Debug)]
pub enum NetManagerMessage {
    Ping(usize),
    HostGame { settings: GameSettings, server: ServerUrl },
    JoinLobby { code: String, server: ServerUrl },
    CancelLobby,
    JoinGame { code: String, server: ServerUrl },
    Move { x: u8, y: u8 },
    Shutdown,
}

#[derive(Deref, Resource)]
pub struct NetServerboundSender(Sender<NetManagerMessage>);

#[derive(Deref, Resource)]
pub struct NetServerboundReceiver(Receiver<NetManagerMessage>);

fn process_net_inbound(
    (r_c, s_s): (Res<NetClientboundReceiver>, Res<NetServerboundSender>),
    mut server_url: ResMut<ServerUrl>,
    #[cfg(not(target_family = "wasm"))] runtime: Res<TokioTasksRuntime>,
    (mut config, mut radios): (ResMut<Config>, ResMut<MenuRadios>),
    mut message_writer: MessageWriter<NetMessage>,
    mut commands: Commands,
    mut ui_opacity: ResMut<TargetUiOpacity>,
    (host_ui_tree, join_ui_tree): (Query<Entity, With<HostGameUiTree>>, Query<Entity, With<JoinGameUiTree>>),
    grid: Res<VisualGrid>,
    mut cells: Query<(&DotCell, &mut CellColor, &Transform)>,
    game_assets: Res<GameAssets>,
    grid_tray: Query<Entity, With<GridTray>>,
    (mut next_game_state, mut need_new_board): (ResMut<NextState<GameOperation>>, ResMut<NextState<NeedNewBoard>>),
    mut local_me: Local<u8>,
) {
    while let Ok(message) = r_c.try_recv() {
        match message {
            NetMessageClientbound::PingFailed(x) => {
                if x + 1 < SERVER_LIST.len() {
                    s_s.force_send(NetManagerMessage::Ping(x + 1)).unwrap();
                }
            }
            NetMessageClientbound::PingSucceeded(x) => {
                (server_url.name, server_url.url) = (SERVER_LIST[x].0.into(), SERVER_LIST[x].1.into());
            }
            NetMessageClientbound::RoomCreated { code } => {
                message_writer.write(NetMessage::RoomCreated { code });
            }
            NetMessageClientbound::RoomReady { code, settings, server } => {
                info!("RoomReady {{ ... }}");
                let Some(play_mode) = radios.radios.get_mut("game-type") else {
                    return;
                };
                play_mode.disable();
                config.grid_size = (settings.width.into(), settings.height.into());
                config.players = vec![
                    PlayerConfigEntry::default_for_player(1).as_human().as_online(),
                    PlayerConfigEntry::default_for_player(2).as_human().as_online(),
                ];
                need_new_board.set(NeedNewBoard(true));
                commands.spawn_task(|| async move {
                    fetch!(NetServerboundSender).with(|s_s| s_s.force_send(NetManagerMessage::JoinGame { code, server }).unwrap());
                    AsyncWorld.sleep(1.5).await;
                    fetch!(NextState<MainState>).with(|x| x.set(MainState::Game));
                    Ok(())
                });
                fade_out_ui(&mut commands, &mut ui_opacity, &host_ui_tree);
                fade_out_ui(&mut commands, &mut ui_opacity, &join_ui_tree);
                info!("{:?}", config.players);
            }
            NetMessageClientbound::RoomNotFound => {
                message_writer.write(NetMessage::RoomNotFound);
            }

            NetMessageClientbound::GameStart { me } => {
                info!("GameStart {{ me: {me} }}");
                *local_me = me;
                config.players[me as usize - 1].set_online(false);
                info!("{:?}", config.players);
            }
            NetMessageClientbound::Move { player, x, y } => {
                if player == *local_me {
                    continue; // Skip
                }
                let entity = grid[y as usize][x as usize];
                let (
                    _,
                    mut color,
                    Transform {
                        translation: Vec3 { x, z, .. },
                        ..
                    },
                ) = cells.get_mut(entity).unwrap();
                commands
                    .entity(entity)
                    .with_related::<Dot>((spawn_dot(*x, *z, &game_assets), ChildOf(grid_tray.single().unwrap())));
                color.player = player as usize;
                next_game_state.set(GameOperation::Animating);
            }

            #[cfg(not(target_family = "wasm"))]
            NetMessageClientbound::Spawn(x) => {
                runtime.spawn_background_task(|_| x);
            }
        }
    }
}

fn setup_channel(mut commands: Commands) {
    let (s_c, r_c) = async_channel::unbounded();
    let (s_s, r_s) = async_channel::unbounded();
    commands.insert_resource(NetClientboundSender(s_c));
    commands.insert_resource(NetClientboundReceiver(r_c));
    commands.insert_resource(NetServerboundSender(s_s));
    commands.insert_resource(NetServerboundReceiver(r_s));
}

fn start_net_manager(
    s_c: Res<NetClientboundSender>,
    s_s: Res<NetServerboundSender>,
    r_s: Res<NetServerboundReceiver>,
    #[cfg(not(target_family = "wasm"))] runtime: Res<TokioTasksRuntime>,
) {
    let future = net_manager_main(r_s.0.clone(), s_c.0.clone());

    #[cfg(not(target_family = "wasm"))]
    runtime.spawn_background_task(|_| future);
    #[cfg(target_family = "wasm")]
    IoTaskPool::get().spawn(future).detach();

    s_s.force_send(NetManagerMessage::Ping(0)).unwrap();
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GameSettings {
    pub capacity: u8,
    pub width: u8,
    pub height: u8,
}

#[derive(Serialize)]
#[serde(tag = "ty", rename_all = "snake_case")]
pub enum LobbyServerbound {
    New(GameSettings),
    Join { code: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "ty", rename_all = "snake_case")]
pub enum LobbyClientbound {
    Created { code: String },
    Ready { code: String, settings: GameSettings },
    RoomNotFound { code: String },
}

#[derive(Serialize)]
#[serde(tag = "ty", rename_all = "snake_case")]
pub enum GameServerbound {
    Move { x: u8, y: u8 },
    Resign,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum LeaveReason {
    Disconnected,
    NoLegalMoves,
    Resigned,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "ty", rename_all = "snake_case")]
pub enum GameClientbound {
    OutOfTurn,
    InvalidMove {
        grid: Vec<u8>, // Vec<CellState>
    },
    Move {
        player: u8,
        x: u8,
        y: u8,
    },
    GameStart {
        me: u8,
    },
    PlayerEliminated {
        player: u8,
        reason: LeaveReason,
    },
    WaitingFor {
        players: u8,
    },
    Turn {
        player: u8,
    },
    GameWin {
        player: u8,
    },
}

async fn net_manager_main(rx: Receiver<NetManagerMessage>, tx: Sender<NetMessageClientbound>) {
    let mut lobby_connection = None;
    let mut game_connection = None;
    loop {
        let message = rx.recv().await.unwrap();
        info!("{message:?}");
        match message {
            NetManagerMessage::Ping(x) => {
                let (_, url) = SERVER_LIST[x];
                let connection = async_wsocket::connect(
                    &Url::parse(&(String::from(url) + "/ws/lobby")).unwrap(),
                    &ConnectionMode::Direct,
                    Duration::from_secs(2),
                )
                .await;
                match connection {
                    Ok(mut ws) => {
                        ws.close().await.unwrap();
                        tx.send(NetMessageClientbound::PingSucceeded(x)).await.unwrap();
                    }
                    Err(e) => {
                        error!("{e:?}");
                        tx.send(NetMessageClientbound::PingFailed(x)).await.unwrap();
                    }
                }
            }
            NetManagerMessage::HostGame { settings, server } => {
                let ws = async_wsocket::connect(
                    &Url::parse(&(server.url.clone() + "/ws/lobby")).unwrap(),
                    &ConnectionMode::Direct,
                    Duration::from_secs(2),
                )
                .await
                .unwrap();
                let (mut ws_tx, mut ws_rx) = ws.split();
                ws_tx
                    .send(Message::Binary(bson::serialize_to_vec(&LobbyServerbound::New(settings)).unwrap()))
                    .await
                    .unwrap();
                lobby_connection = Some(ws_tx);

                let reading_future = {
                    let tx = tx.clone(); // so we can move this into the future
                    async move {
                        let mut the_code = String::new();
                        while let Some(x) = ws_rx.next().await {
                            match x {
                                Err(e) => {
                                    error!("{e:?}");
                                }
                                Ok(message) => {
                                    match message {
                                        Message::Binary(x) => {
                                            let message = bson::deserialize_from_slice(&x).unwrap();
                                            info!("{message:?}");
                                            match message {
                                                LobbyClientbound::Created { code } => {
                                                    the_code = code.clone();
                                                    tx.send(NetMessageClientbound::RoomCreated { code }).await.unwrap();
                                                }
                                                LobbyClientbound::Ready { settings, .. } => {
                                                    tx.send(NetMessageClientbound::RoomReady {
                                                        code: the_code.clone(),
                                                        settings,
                                                        server: server.clone(),
                                                    })
                                                    .await
                                                    .unwrap();
                                                }
                                                LobbyClientbound::RoomNotFound { .. } => {
                                                    // shouldn't happen
                                                }
                                            }
                                        }
                                        #[cfg(not(target_family = "wasm"))]
                                        Message::Close(_) => {}
                                        x => info!("unhandled message: {x:?}"),
                                    }
                                }
                            }
                        }
                    }
                };
                #[cfg(not(target_family = "wasm"))]
                tx.send(NetMessageClientbound::Spawn(Box::pin(reading_future))).await.unwrap();
                #[cfg(target_family = "wasm")]
                IoTaskPool::get().spawn(reading_future).detach();
            }
            NetManagerMessage::JoinLobby { code, server } => {
                let ws = async_wsocket::connect(
                    &Url::parse(&(server.url.clone() + "/ws/lobby")).unwrap(),
                    &ConnectionMode::Direct,
                    Duration::from_secs(2),
                )
                .await
                .unwrap();
                let (mut ws_tx, mut ws_rx) = ws.split();
                ws_tx
                    .send(Message::Binary(bson::serialize_to_vec(&LobbyServerbound::Join { code: code.clone() }).unwrap()))
                    .await
                    .unwrap();
                lobby_connection = Some(ws_tx);

                let reading_future = {
                    let tx = tx.clone(); // so we can move this into the future
                    async move {
                        while let Some(x) = ws_rx.next().await {
                            match x {
                                Err(e) => {
                                    error!("{e:?}");
                                }
                                Ok(message) => {
                                    match message {
                                        Message::Binary(x) => {
                                            let message = bson::deserialize_from_slice(&x).unwrap();
                                            info!("{message:?}");
                                            match message {
                                                LobbyClientbound::Created { .. } => {
                                                    // shouldn't happen
                                                }
                                                LobbyClientbound::Ready { settings, .. } => {
                                                    tx.send(NetMessageClientbound::RoomReady {
                                                        code: code.clone(),
                                                        settings,
                                                        server: server.clone(),
                                                    })
                                                    .await
                                                    .unwrap();
                                                }
                                                LobbyClientbound::RoomNotFound { .. } => {
                                                    tx.send(NetMessageClientbound::RoomNotFound).await.unwrap();
                                                }
                                            }
                                        }
                                        #[cfg(not(target_family = "wasm"))]
                                        Message::Close(_) => {}
                                        x => info!("unhandled message: {x:?}"),
                                    }
                                }
                            }
                        }
                    }
                };
                #[cfg(not(target_family = "wasm"))]
                tx.send(NetMessageClientbound::Spawn(Box::pin(reading_future))).await.unwrap();
                #[cfg(target_family = "wasm")]
                IoTaskPool::get().spawn(reading_future).detach();
            }
            NetManagerMessage::CancelLobby => {
                if let Some(mut ws) = lobby_connection.take() {
                    ws.close().await.unwrap();
                }
            }
            NetManagerMessage::JoinGame { code, server } => {
                if let Some(mut ws) = lobby_connection.take() {
                    ws.close().await.unwrap();
                }
                let ws = async_wsocket::connect(
                    &Url::parse(&(format!("{}/ws/game?id={code}", server.url))).unwrap(),
                    &ConnectionMode::Direct,
                    Duration::from_secs(2),
                )
                .await
                .unwrap();
                let (ws_tx, mut ws_rx) = ws.split();
                game_connection = Some(ws_tx);

                let reading_future = {
                    let tx = tx.clone(); // so we can move this into the future
                    async move {
                        while let Some(x) = ws_rx.next().await {
                            match x {
                                Err(e) => {
                                    error!("{e:?}");
                                }
                                Ok(message) => match message {
                                    Message::Binary(x) => {
                                        let message = match bson::deserialize_from_slice(&x) {
                                            Ok(x) => x,
                                            Err(e) => {
                                                error!("error handling unrecognized server message: {e:?} (message: {x:?})");
                                                continue;
                                            }
                                        };
                                        info!("{message:?}");
                                        match message {
                                            GameClientbound::WaitingFor { .. } | GameClientbound::Turn { .. } => {
                                                // May use this later, but don't need to yet
                                            }
                                            GameClientbound::GameStart { me } => {
                                                tx.send(NetMessageClientbound::GameStart { me }).await.unwrap();
                                            }
                                            GameClientbound::Move { player, x, y } => {
                                                tx.send(NetMessageClientbound::Move { player, x, y }).await.unwrap();
                                            }
                                            x => {
                                                error!("unhandled server message: {x:?}");
                                            }
                                        }
                                    }
                                    #[cfg(not(target_family = "wasm"))]
                                    Message::Close(_) => {}
                                    x => info!("unhandled websocket message: {x:?}"),
                                },
                            }
                        }
                    }
                };
                #[cfg(not(target_family = "wasm"))]
                tx.send(NetMessageClientbound::Spawn(Box::pin(reading_future))).await.unwrap();
                #[cfg(target_family = "wasm")]
                IoTaskPool::get().spawn(reading_future).detach();
            }
            NetManagerMessage::Move { x, y } => {
                if let Some(ws) = &mut game_connection {
                    ws.send(Message::Binary(bson::serialize_to_vec(&GameServerbound::Move { x, y }).unwrap()))
                        .await
                        .unwrap();
                }
            }
            NetManagerMessage::Shutdown => {
                if let Some(mut x) = lobby_connection.take() {
                    let _ = x.close().await;
                }
                if let Some(mut x) = game_connection.take() {
                    let _ = x.send(Message::Binary(bson::serialize_to_vec(&GameServerbound::Resign).unwrap())).await;
                    let _ = x.close().await;
                }
            }
        }
    }
}
