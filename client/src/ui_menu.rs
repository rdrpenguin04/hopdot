mod support;

mod credits;
mod game_end;
mod rules;
mod settings;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::{Config, GameAssets, PlayerConfigEntry, SimpleConfig, ui_menu::rules::RulesPageNumber};

#[derive(Component)]
pub struct CreditsUiTree;

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

#[derive(Component)]
struct PlayModeSwitch(usize);

#[derive(Component)]
struct BotLevelSwitch(usize);

fn update_config_from_buttons(
    mut buttons: ParamSet<(
        Query<(&PlayModeSwitch, &mut BackgroundColor, Ref<Interaction>)>,
        Query<(&BotLevelSwitch, &mut BackgroundColor, Ref<Interaction>)>,
    )>,
    mut simple_config: ResMut<SimpleConfig>,
    mut config: ResMut<Config>,
) {
    let mut play_mode = simple_config.0;
    for (id, _, int) in buttons.p0().iter() {
        if int.is_changed() && *int == Interaction::Pressed {
            play_mode = id.0;
        }
    }
    simple_config.0 = play_mode;
    for (id, mut color, _) in buttons.p0().iter_mut() {
        color.0 = if id.0 == play_mode {
            Color::srgba(0.4, 0.4, 0.4, color.0.alpha())
        } else {
            Color::srgba(0.2, 0.2, 0.2, color.0.alpha())
        };
    }
    let mut bot_level = simple_config.1;
    for (id, _, int) in buttons.p1().iter() {
        if int.is_changed() && *int == Interaction::Pressed {
            bot_level = id.0;
        }
    }
    simple_config.1 = bot_level;
    for (id, mut color, _) in buttons.p1().iter_mut() {
        color.0 = if id.0 == bot_level {
            Color::srgba(0.4, 0.4, 0.4, color.0.alpha())
        } else {
            Color::srgba(0.2, 0.2, 0.2, color.0.alpha())
        };
    }
    if simple_config.is_changed() {
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
            if play_mode == 0 {
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
