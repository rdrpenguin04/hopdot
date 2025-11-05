pub mod support;

mod credits;
mod custom_game_setup;
mod game_end;
mod game_hud;
mod host_game;
mod join_game;
mod rules;
mod settings;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    Config, GameAssets, PlayerConfigEntry,
    menu::MenuRadios,
    net::NetMessage,
    ui_menu::{custom_game_setup::render_player_config, rules::RulesPageNumber},
};

#[derive(Component)]
pub struct CreditsUiTree;

#[derive(Component)]
pub struct CustomGameSetupUiTree;

#[derive(Component)]
pub struct GameEndUiTree;

#[derive(Component)]
pub struct GameEndText;

#[derive(Component)]
pub struct HostGameUiTree;

#[derive(Component)]
pub struct JoinGameUiTree;

#[derive(Component)]
pub struct RulesUiTree;

#[derive(Component)]
pub struct SettingsUiTree;

#[derive(Component)]
pub struct GameHudUiTree;

pub fn plugin(app: &mut App) {
    app.insert_resource(RulesPageNumber(1))
        .insert_resource(CustomConfig(Config {
            players: vec![],
            grid_size: (6, 6),
        }))
        .add_systems(
            Update,
            (
                update_config_from_buttons,
                update_ui_scale,
                render_player_config,
                update_net_menus,
                game_hud::run_menu,
            ),
        )
        .add_systems(Startup, |mut commands: Commands, ga: Res<GameAssets>| {
            commands.spawn(custom_game_setup::menu(&ga));
            commands.spawn(game_end::menu(&ga));
            commands.spawn(rules::menu(&ga));
            commands.spawn(settings::menu(&ga));
            commands.spawn(credits::menu(&ga));
            commands.spawn(host_game::menu(&ga));
            commands.spawn(join_game::menu(&ga));
            commands.spawn(game_hud::menu(&ga));
        });
}

fn update_ui_scale(mut ui_scale: ResMut<UiScale>, windows: Query<&Window, With<PrimaryWindow>>) {
    // TODO: Cheap hack. Do actual size estimation later.
    let Ok(window) = windows.single() else {
        warn!("either we have no window, or we have multiple. either one is bad.");
        return;
    };
    ui_scale.0 = if window.width() > 1000.0 {
        1.0
    } else if window.width() > 600.0 {
        0.75
    } else {
        0.6
    };
}

#[derive(Deref, DerefMut, Resource)]
pub struct CustomConfig(pub Config);

#[derive(Component)]
pub struct InfoText;

// play modes are:
// * PvP: 1
// * PvB: 0 (default)
// * BvB: 2
// * Custom: 3
fn update_config_from_buttons(mut radios: ResMut<MenuRadios>, mut config: ResMut<Config>, custom_config: Res<CustomConfig>) {
    let Some(play_mode) = radios.radios.get("game-type").and_then(|x| x.value_opt()) else {
        return;
    };
    let Some(bot_level_radio) = radios.radios.get_mut("game-difficulty") else {
        return;
    };
    if play_mode == 1 || play_mode == 3 {
        bot_level_radio.disable();
    } else {
        bot_level_radio.enable();
    }
    let bot_level = bot_level_radio.value();
    if play_mode != 3 {
        // TODO: update these definitions to use modified player defaults rather than copy-pasted settings
        config.players = vec![
            if play_mode == 2 {
                PlayerConfigEntry::Bot {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    level: bot_level,
                    _name: String::new(),
                    online: false,
                }
            } else {
                PlayerConfigEntry::Human {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    name: "Player 1".into(),
                    _level: 0,
                    online: false,
                }
            },
            if play_mode == 1 {
                PlayerConfigEntry::Human {
                    color: Color::srgb(0.0, 0.0, 1.0),
                    name: "Player 2".into(),
                    _level: 0,
                    online: false,
                }
            } else {
                PlayerConfigEntry::Bot {
                    color: Color::srgb(0.0, 0.0, 1.0),
                    level: bot_level,
                    _name: String::new(),
                    online: false,
                }
            },
        ];
        config.grid_size = (6, 6);
    } else {
        config.players.clear();
        custom_config
            .players
            .iter()
            .filter(|x| !matches!(x, PlayerConfigEntry::Disabled { .. }))
            .cloned()
            .collect_into(&mut config.players);
        config.grid_size = custom_config.grid_size;
    }
    // config.players = vec![
    //     PlayerConfigEntry::Human {
    //         color: Color::srgb(0.0, 1.0, 0.0),
    //         name: "Player 1".into(),
    //     },
    //     PlayerConfigEntry::Bot {
    //         color: Color::srgb(0.0, 0.0, 1.0),
    //         level: 1,
    //     },
    //     PlayerConfigEntry::Bot {
    //         color: Color::srgb(1.0, 0.0, 0.0),
    //         level: 2,
    //     },
    //     PlayerConfigEntry::Bot {
    //         color: Color::srgb(0.0, 1.0, 1.0),
    //         level: 3,
    //     },
    // ];
}

pub fn update_net_menus(mut net_events: MessageReader<NetMessage>, mut info_texts: Query<(&mut Node, &mut Text), With<InfoText>>) {
    for event in net_events.read() {
        match event {
            NetMessage::RoomCreated { code } => {
                for (mut node, mut text) in &mut info_texts {
                    node.display = Display::Flex;
                    text.0 = format!("Created! Code is {code}");
                }
            }
            NetMessage::RoomNotFound => {
                for (mut node, mut text) in &mut info_texts {
                    node.display = Display::Flex;
                    text.0 = "Error: room not found".into();
                }
            }
        }
    }
}
