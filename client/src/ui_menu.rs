use bevy::prelude::*;
use bevy_defer::{AsyncAccess as _, AsyncCommandsExtension as _, AsyncWorld, fetch};

use crate::{
    Config, FlashIntensity, GameAssets, MainState, PlayerConfigEntry, SimpleConfig,
    anim::{AnimateBackgroundColor, TargetUiOpacity},
    observe::observe,
};

#[derive(Component)]
pub struct CreditsUiTree;

#[derive(Component)]
pub struct GameEndUiTree;

#[derive(Component)]
pub struct GameEndText;

#[derive(Component)]
pub struct RulesUiTree;

#[derive(Resource, Deref)]
struct RulesPageNumber(usize);

pub fn plugin(app: &mut App) {
    app.insert_resource(RulesPageNumber(1)).add_systems(Update, update_config_from_buttons);
}

fn back_to_main_menu<T: Component>(game_assets: &GameAssets) -> impl Bundle {
    (
        Node {
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        },
        Button,
        Text::new("Back to main menu"),
        TextFont {
            font: game_assets.mono_font.clone_weak(),
            font_size: 20.0,
            ..default()
        },
        Outline::new(Val::Px(5.0), Val::Px(5.0), Color::WHITE),
        BorderRadius::all(Val::Px(5.0)),
        observe(
            |_: Trigger<Pointer<Click>>,
             mut commands: Commands,
             mut next_state: ResMut<NextState<MainState>>,
             mut ui_opacity: ResMut<TargetUiOpacity>,
             ui_tree: Query<Entity, With<T>>| {
                next_state.set(MainState::Menu);
                ui_opacity.0 = 0.0;
                let ui_tree = ui_tree.single().unwrap();
                commands.spawn_task(move || async move {
                    AsyncWorld.sleep(1.0).await;
                    fetch!(ui_tree, Visibility).get_mut(|x| *x = Visibility::Hidden)?;
                    Ok(())
                });
            },
        ),
    )
}

fn left_button(game_assets: &GameAssets) -> impl Bundle {
    (
        Node {
            margin: UiRect::horizontal(Val::Px(10.0)),
            ..default()
        },
        Button,
        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        Text::new("<"),
        TextFont {
            font: game_assets.mono_font.clone_weak(),
            font_size: 15.0,
            ..default()
        },
        Outline::new(Val::Px(2.0), Val::Px(5.0), Color::WHITE),
        BorderRadius::all(Val::Px(5.0)),
        AnimateBackgroundColor,
    )
}

fn right_button(game_assets: &GameAssets) -> impl Bundle {
    (
        Node {
            margin: UiRect::horizontal(Val::Px(10.0)),
            ..default()
        },
        Button,
        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        Text::new(">"),
        TextFont {
            font: game_assets.mono_font.clone_weak(),
            font_size: 15.0,
            ..default()
        },
        Outline::new(Val::Px(2.0), Val::Px(5.0), Color::WHITE),
        BorderRadius::all(Val::Px(5.0)),
        AnimateBackgroundColor,
    )
}

pub fn game_end(game_assets: &GameAssets) -> impl Bundle {
    (
        GameEndUiTree,
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        Visibility::Hidden,
        children![
            (
                Node {
                    margin: UiRect::top(Val::Percent(10.0)),
                    ..default()
                },
                Text::new("Player 1 wins!"),
                TextFont {
                    font: game_assets.bold_font.clone_weak(),
                    font_size: 60.0,
                    ..default()
                },
                GameEndText,
            ),
            back_to_main_menu::<GameEndUiTree>(game_assets)
        ],
    )
}

pub fn credits(game_assets: &GameAssets) -> impl Bundle {
    (
        CreditsUiTree,
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        Visibility::Hidden,
        children![
            (
                Text::new("Credits"),
                TextFont {
                    font: game_assets.bold_font.clone_weak(),
                    font_size: 60.0,
                    ..default()
                },
            ),
            (
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
                Text::new(
                    "Coding and most assets by Ray Redondo\nOriginal concept from KJumpingCube\n\nThis game is open source! Check it out at https://github.com/rdrpenguin04/hopdot"
                ),
                TextFont {
                    font: game_assets.mono_font.clone_weak(),
                    font_size: 20.0,
                    ..default()
                },
            ),
            back_to_main_menu::<CreditsUiTree>(game_assets),
            (
                Node {
                    margin: UiRect::top(Val::Px(60.0)),
                    ..default()
                },
                Text::new(
                    "© 2025 Lightning Creations. The Lightning Creations logo is a trademark of Lightning Creations and is used by permission of the LC Admins. For more information, visit https://lcdev.xyz"
                ),
                TextFont {
                    font: game_assets.mono_font.clone_weak(),
                    font_size: 7.0,
                    ..default()
                },
            )
        ],
    )
}

pub fn rules(game_assets: &GameAssets) -> impl Bundle {
    #[derive(Component)]
    struct RulesText;

    #[derive(Component)]
    struct RulesPageNumberText;

    ////////////////////////////////////////////////////////////////////////////////////////////
    const RULES_PAGES: [&str; 3] = [
        "The object of Hopdot is to claim the entire board. You can claim a square in one of two \
        ways: directly taking an unowned square on your turn, or cascading from a neighboring \
        square.",
        "Each square has a maximum carrying capacity equal to the number of neighbors it has. In \
        other words:\n  * the corner squares can hold two dots,\n  * the edge squares can hold \
        three dots,\n  * and the center squares can hold four dots.",
        "A useful strategy tip to know: the corners are the strategically best squares to take \
        first, as they have few neighbors and can be defended easily. The edges come next.\n\n\
        The other important thing to avoid is racing. If you have a square near an opponent's \
        square, and their square has more dots than yours, don't try to build yours; you'll just \
        give them a more-built cell to work with.",
    ];

    (
        RulesUiTree,
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        Visibility::Hidden,
        children![
            (
                Text::new("Rules"),
                TextFont {
                    font: game_assets.bold_font.clone_weak(),
                    font_size: 60.0,
                    ..default()
                },
            ),
            (
                Node {
                    min_width: Val::Px(0.0),
                    max_width: Val::Percent(50.0),
                    ..default()
                },
                Text::new(RULES_PAGES[0]),
                TextFont {
                    font: game_assets.mono_font.clone_weak(),
                    font_size: 20.0,
                    ..default()
                },
                RulesText,
            ),
            (
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    (
                        Text::new("Page: "),
                        TextFont {
                            font: game_assets.mono_font.clone_weak(),
                            font_size: 20.0,
                            ..default()
                        },
                    ),
                    (
                        left_button(game_assets),
                        observe(
                            |_: Trigger<Pointer<Click>>,
                             mut rules_page_number: ResMut<RulesPageNumber>,
                             mut page_num_text: Query<&mut Text, With<RulesPageNumberText>>,
                             mut rules_text: Query<&mut Text, (With<RulesText>, Without<RulesPageNumberText>)>| {
                                rules_page_number.0 -= 1;
                                if rules_page_number.0 < 1 {
                                    rules_page_number.0 = 1;
                                }
                                page_num_text.single_mut().unwrap().0 = format!("{}", rules_page_number.0);
                                rules_text.single_mut().unwrap().0 = RULES_PAGES[rules_page_number.0 - 1].into();
                            },
                        )
                    ),
                    (
                        Text::new("1"),
                        TextFont {
                            font: game_assets.mono_font.clone_weak(),
                            font_size: 20.0,
                            ..default()
                        },
                        RulesPageNumberText,
                    ),
                    (
                        right_button(game_assets),
                        observe(
                            |_: Trigger<Pointer<Click>>,
                             mut rules_page_number: ResMut<RulesPageNumber>,
                             mut page_num_text: Query<&mut Text, With<RulesPageNumberText>>,
                             mut rules_text: Query<&mut Text, (With<RulesText>, Without<RulesPageNumberText>)>| {
                                rules_page_number.0 += 1;
                                if rules_page_number.0 > RULES_PAGES.len() {
                                    rules_page_number.0 = RULES_PAGES.len();
                                }
                                page_num_text.single_mut().unwrap().0 = format!("{}", rules_page_number.0);
                                rules_text.single_mut().unwrap().0 = RULES_PAGES[rules_page_number.0 - 1].into();
                            },
                        )
                    )
                ]
            ),
            back_to_main_menu::<RulesUiTree>(game_assets),
        ],
    )
}

#[derive(Component)]
pub struct SettingsUiTree;

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
}

pub fn settings(game_assets: &GameAssets) -> impl Bundle {
    #[derive(Component)]
    struct WidthText;
    #[derive(Component)]
    struct HeightText;
    #[derive(Component)]
    struct FlashIntensityText;
    (
        SettingsUiTree,
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        Visibility::Hidden,
        children![
            (
                Text::new("Settings"),
                TextFont {
                    font: game_assets.bold_font.clone_weak(),
                    font_size: 60.0,
                    ..default()
                },
            ),
            (
                Node {
                    margin: UiRect::top(Val::Px(5.0)),
                    display: Display::Block,
                    ..default()
                },
                children![
                    (
                        Text::new("Player Config"),
                        TextFont {
                            font: game_assets.bold_font.clone_weak(),
                            font_size: 40.0,
                            ..default()
                        },
                    ),
                    (
                        Node {
                            margin: UiRect::top(Val::Px(20.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        children![
                            (
                                Text::new("mode: "),
                                TextFont {
                                    font: game_assets.mono_font.clone_weak(),
                                    font_size: 20.0,
                                    ..default()
                                },
                            ),
                            (
                                Node {
                                    margin: UiRect::horizontal(Val::Px(10.0)),
                                    ..default()
                                },
                                Button,
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                Text::new("Player vs. Player"),
                                TextFont {
                                    font: game_assets.mono_font.clone_weak(),
                                    font_size: 15.0,
                                    ..default()
                                },
                                Outline::new(Val::Px(2.0), Val::Px(5.0), Color::WHITE),
                                BorderRadius::all(Val::Px(5.0)),
                                AnimateBackgroundColor,
                                PlayModeSwitch(0),
                            ),
                            (
                                Node {
                                    margin: UiRect::horizontal(Val::Px(10.0)),
                                    ..default()
                                },
                                Button,
                                BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                                Text::new("Player vs. Bot"),
                                TextFont {
                                    font: game_assets.mono_font.clone_weak(),
                                    font_size: 15.0,
                                    ..default()
                                },
                                Outline::new(Val::Px(2.0), Val::Px(5.0), Color::WHITE),
                                BorderRadius::all(Val::Px(5.0)),
                                AnimateBackgroundColor,
                                PlayModeSwitch(1),
                            ),
                            (
                                Node {
                                    margin: UiRect::horizontal(Val::Px(10.0)),
                                    ..default()
                                },
                                Button,
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                Text::new("Bot vs. Bot"),
                                TextFont {
                                    font: game_assets.mono_font.clone_weak(),
                                    font_size: 15.0,
                                    ..default()
                                },
                                Outline::new(Val::Px(2.0), Val::Px(5.0), Color::WHITE),
                                BorderRadius::all(Val::Px(5.0)),
                                AnimateBackgroundColor,
                                PlayModeSwitch(2),
                            )
                        ]
                    ),
                    (
                        Node {
                            margin: UiRect::top(Val::Px(20.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        children![
                            (
                                Text::new("bot level: "),
                                TextFont {
                                    font: game_assets.mono_font.clone_weak(),
                                    font_size: 20.0,
                                    ..default()
                                },
                            ),
                            (
                                Node {
                                    margin: UiRect::horizontal(Val::Px(10.0)),
                                    ..default()
                                },
                                Button,
                                BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                                Text::new("Easiest"),
                                TextFont {
                                    font: game_assets.mono_font.clone_weak(),
                                    font_size: 15.0,
                                    ..default()
                                },
                                Outline::new(Val::Px(2.0), Val::Px(5.0), Color::WHITE),
                                BorderRadius::all(Val::Px(5.0)),
                                AnimateBackgroundColor,
                                BotLevelSwitch(0),
                            ),
                            (
                                Node {
                                    margin: UiRect::horizontal(Val::Px(10.0)),
                                    ..default()
                                },
                                Button,
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                Text::new("Easy"),
                                TextFont {
                                    font: game_assets.mono_font.clone_weak(),
                                    font_size: 15.0,
                                    ..default()
                                },
                                Outline::new(Val::Px(2.0), Val::Px(5.0), Color::WHITE),
                                BorderRadius::all(Val::Px(5.0)),
                                AnimateBackgroundColor,
                                BotLevelSwitch(1),
                            ),
                            (
                                Node {
                                    margin: UiRect::horizontal(Val::Px(10.0)),
                                    ..default()
                                },
                                Button,
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                Text::new("Medium"),
                                TextFont {
                                    font: game_assets.mono_font.clone_weak(),
                                    font_size: 15.0,
                                    ..default()
                                },
                                Outline::new(Val::Px(2.0), Val::Px(5.0), Color::WHITE),
                                BorderRadius::all(Val::Px(5.0)),
                                AnimateBackgroundColor,
                                BotLevelSwitch(2),
                            ),
                            (
                                Node {
                                    width: Val::Px(100.0),
                                    ..default()
                                },
                                Text::new("more levels coming soon"),
                                TextFont {
                                    font: game_assets.mono_font.clone_weak(),
                                    font_size: 8.0,
                                    ..default()
                                },
                            )
                        ]
                    )
                ]
            ),
            (
                Node {
                    margin: UiRect::top(Val::Px(5.0)),
                    display: Display::Block,
                    ..default()
                },
                children![
                    (
                        Text::new("Grid Size"),
                        TextFont {
                            font: game_assets.bold_font.clone_weak(),
                            font_size: 40.0,
                            ..default()
                        },
                    ),
                    (
                        Node {
                            margin: UiRect::top(Val::Px(10.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        children![
                            (
                                Text::new("width: "),
                                TextFont {
                                    font: game_assets.mono_font.clone_weak(),
                                    font_size: 20.0,
                                    ..default()
                                },
                            ),
                            (
                                left_button(game_assets),
                                observe(
                                    |_: Trigger<Pointer<Click>>, mut config: ResMut<Config>, mut width_text: Query<&mut Text, With<WidthText>>| {
                                        config.grid_size.0 -= 1;
                                        if config.grid_size.0 < 1 {
                                            config.grid_size.0 = 1;
                                        }
                                        width_text.single_mut().unwrap().0 = format!("{:>2}", config.grid_size.0);
                                    },
                                )
                            ),
                            (
                                Text::new(" 6"),
                                TextFont {
                                    font: game_assets.mono_font.clone_weak(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                WidthText,
                            ),
                            (
                                right_button(game_assets),
                                observe(
                                    |_: Trigger<Pointer<Click>>, mut config: ResMut<Config>, mut width_text: Query<&mut Text, With<WidthText>>| {
                                        config.grid_size.0 += 1;
                                        if config.grid_size.0 > 20 {
                                            config.grid_size.0 = 20;
                                        }
                                        width_text.single_mut().unwrap().0 = format!("{:>2}", config.grid_size.0);
                                    },
                                )
                            )
                        ]
                    ),
                    (
                        Node {
                            margin: UiRect::top(Val::Px(20.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        children![
                            (
                                Text::new("height: "),
                                TextFont {
                                    font: game_assets.mono_font.clone_weak(),
                                    font_size: 20.0,
                                    ..default()
                                },
                            ),
                            (
                                left_button(game_assets),
                                observe(
                                    |_: Trigger<Pointer<Click>>, mut config: ResMut<Config>, mut height_text: Query<&mut Text, With<HeightText>>| {
                                        config.grid_size.1 -= 1;
                                        if config.grid_size.1 < 1 {
                                            config.grid_size.1 = 1;
                                        }
                                        height_text.single_mut().unwrap().0 = format!("{:>2}", config.grid_size.1);
                                    },
                                )
                            ),
                            (
                                Text::new(" 6"),
                                TextFont {
                                    font: game_assets.mono_font.clone_weak(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                HeightText,
                            ),
                            (
                                right_button(game_assets),
                                observe(
                                    |_: Trigger<Pointer<Click>>, mut config: ResMut<Config>, mut height_text: Query<&mut Text, With<HeightText>>| {
                                        config.grid_size.1 += 1;
                                        if config.grid_size.1 > 20 {
                                            config.grid_size.1 = 20;
                                        }
                                        height_text.single_mut().unwrap().0 = format!("{:>2}", config.grid_size.1);
                                    },
                                )
                            )
                        ]
                    )
                ]
            ),
            (
                Node {
                    margin: UiRect::top(Val::Px(5.0)),
                    display: Display::Block,
                    ..default()
                },
                children![
                    (
                        Text::new("Flash intensity"),
                        TextFont {
                            font: game_assets.bold_font.clone_weak(),
                            font_size: 40.0,
                            ..default()
                        },
                    ),
                    (
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        children![
                            (
                                left_button(game_assets),
                                observe(
                                    |_: Trigger<Pointer<Click>>,
                                     mut flash_intensity: ResMut<FlashIntensity>,
                                     mut flash_intensity_text: Query<&mut Text, With<FlashIntensityText>>| {
                                        flash_intensity.0 -= 0.1;
                                        if flash_intensity.0 < 0.0 {
                                            flash_intensity.0 = 0.0;
                                        }
                                        flash_intensity_text.single_mut().unwrap().0 = format!("{:#1.1}", flash_intensity.0);
                                    },
                                )
                            ),
                            (
                                Text::new("0.3"),
                                TextFont {
                                    font: game_assets.mono_font.clone_weak(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                FlashIntensityText,
                            ),
                            (
                                right_button(game_assets),
                                observe(
                                    |_: Trigger<Pointer<Click>>,
                                     mut flash_intensity: ResMut<FlashIntensity>,
                                     mut flash_intensity_text: Query<&mut Text, With<FlashIntensityText>>| {
                                        flash_intensity.0 += 0.1;
                                        if flash_intensity.0 > 1.0 {
                                            flash_intensity.0 = 1.0;
                                        }
                                        flash_intensity_text.single_mut().unwrap().0 = format!("{:#1.1}", flash_intensity.0);
                                    },
                                )
                            )
                        ]
                    )
                ]
            ),
            back_to_main_menu::<SettingsUiTree>(game_assets)
        ],
    )
}
