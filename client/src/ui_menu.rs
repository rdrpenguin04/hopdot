mod support;

mod credits;
mod custom_game_setup;
mod game_end;
mod rules;
mod settings;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::{Config, GameAssets, PlayerConfigEntry, menu::MenuRadios, ui_menu::rules::RulesPageNumber};

#[derive(Component)]
pub struct CreditsUiTree;

#[derive(Component)]
pub struct CustomGameSetupUiTree;

#[derive(Component)]
pub struct GameEndUiTree;

#[derive(Component)]
pub struct GameEndText;

#[derive(Component)]
pub struct RulesUiTree;

#[derive(Component)]
pub struct SettingsUiTree;

pub fn plugin(app: &mut App) {
    app.insert_resource(RulesPageNumber(1))
        .add_systems(Update, (update_config_from_buttons, update_ui_scale))
        .add_systems(Startup, |mut commands: Commands, ga: Res<GameAssets>| {
            commands.spawn(custom_game_setup::menu(&ga));
            commands.spawn(game_end::menu(&ga));
            commands.spawn(rules::menu(&ga));
            commands.spawn(settings::menu(&ga));
            commands.spawn(credits::menu(&ga));
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

// play modes are:
// * PvP: 1
// * PvB: 0 (default)
// * BvB: 2
// * Custom: 3
fn update_config_from_buttons(mut radios: ResMut<MenuRadios>, mut config: ResMut<Config>) {
    let Some(play_mode) = radios.radios.get("game-type").map(|x| x.value()) else {
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
        config.players = vec![
            if play_mode == 2 {
                PlayerConfigEntry::Bot {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    level: bot_level,
                }
            } else {
                PlayerConfigEntry::Human {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    name: "Player 1".into(),
                }
            },
            if play_mode == 1 {
                PlayerConfigEntry::Human {
                    color: Color::srgb(0.0, 0.0, 1.0),
                    name: "Player 2".into(),
                }
            } else {
                PlayerConfigEntry::Bot {
                    color: Color::srgb(0.0, 0.0, 1.0),
                    level: bot_level,
                }
            },
        ];
    } else {
        // TODO: read from expanded custom config
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
