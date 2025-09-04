use bevy::{prelude::*, window::PrimaryWindow};
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
    app.insert_resource(RulesPageNumber(1))
        .add_systems(Update, (update_config_from_buttons, update_ui_scale));
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

fn h1(ga: &GameAssets, text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font: ga.bold_font.clone_weak(),
            font_size: 60.0,
            ..default()
        },
    )
}

fn h2(ga: &GameAssets, text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font: ga.bold_font.clone_weak(),
            font_size: 40.0,
            ..default()
        },
    )
}

fn p(ga: &GameAssets, text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font: ga.mono_font.clone_weak(),
            font_size: 20.0,
            ..default()
        },
    )
}

fn button_with_bg(ga: &GameAssets, text: impl Into<String>, color: Color) -> impl Bundle {
    (
        Node {
            margin: UiRect::horizontal(Val::Px(5.0)),
            padding: UiRect::axes(Val::Px(3.0), Val::Px(3.0)),
            ..default()
        },
        Button,
        AnimateBackgroundColor,
        BackgroundColor(color),
        Outline::new(Val::Px(3.0), Val::Px(0.0), Color::WHITE),
        BorderRadius::all(Val::Px(5.0)),
        children![(
            Text::new(text),
            TextFont {
                font: ga.mono_font.clone_weak(),
                font_size: 15.0,
                ..default()
            },
        )],
    )
}

fn back_to_main_menu<T: Component>(ga: &GameAssets) -> impl Bundle {
    (
        Node {
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        },
        Button,
        p(ga, "Back to main menu"),
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

fn left_button(ga: &GameAssets) -> impl Bundle {
    button_with_bg(ga, "<", Color::srgb(0.2, 0.2, 0.2))
}

fn right_button(ga: &GameAssets) -> impl Bundle {
    button_with_bg(ga, ">", Color::srgb(0.2, 0.2, 0.2))
}

pub fn game_end(ga: &GameAssets) -> impl Bundle {
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
                h1(ga, "Player 1 wins!"),
                GameEndText,
            ),
            back_to_main_menu::<GameEndUiTree>(ga)
        ],
    )
}

pub fn credits(ga: &GameAssets) -> impl Bundle {
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
            h1(ga, "Credits"),
            (
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
                p(
                    ga,
                    "Coding and most assets by Ray Redondo\nOriginal concept from KJumpingCube\n\nThis game is open source! Check it out at https://github.com/rdrpenguin04/hopdot"
                ),
            ),
            back_to_main_menu::<CreditsUiTree>(ga),
            (
                Node {
                    margin: UiRect::top(Val::Px(60.0)),
                    ..default()
                },
                Text::new(
                    "© 2025 Lightning Creations. The Lightning Creations logo is a trademark of Lightning Creations and is used by permission of the LC Admins. For more information, visit https://lcdev.xyz"
                ),
                TextFont {
                    font: ga.mono_font.clone_weak(),
                    font_size: 7.0,
                    ..default()
                },
            )
        ],
    )
}

pub fn rules(ga: &GameAssets) -> impl Bundle {
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
            h1(ga, "Rules"),
            (
                Node {
                    min_width: Val::Px(0.0),
                    max_width: Val::Percent(50.0),
                    ..default()
                },
                p(ga, RULES_PAGES[0]),
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
                    p(ga, "Page: "),
                    (
                        left_button(ga),
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
                    (p(ga, "1"), RulesPageNumberText,),
                    (
                        right_button(ga),
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
            back_to_main_menu::<RulesUiTree>(ga),
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

pub fn settings(ga: &GameAssets) -> impl Bundle {
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
            h1(ga, "Settings"),
            (
                Node {
                    margin: UiRect::top(Val::Px(5.0)),
                    display: Display::Block,
                    ..default()
                },
                children![
                    h2(ga, "Player Config"),
                    (
                        Node {
                            margin: UiRect::top(Val::Px(20.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        children![
                            p(ga, "mode: "),
                            (button_with_bg(ga, "Player vs. Player", Color::srgb(0.2, 0.2, 0.2)), PlayModeSwitch(0)),
                            (button_with_bg(ga, "Player vs. Bot", Color::srgb(0.4, 0.4, 0.4)), PlayModeSwitch(1)),
                            (button_with_bg(ga, "Bot vs. Bot", Color::srgb(0.2, 0.2, 0.2)), PlayModeSwitch(2)),
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
                            p(ga, "bot level: "),
                            (button_with_bg(ga, "Easiest", Color::srgb(0.4, 0.4, 0.4)), BotLevelSwitch(0)),
                            (button_with_bg(ga, "Easy", Color::srgb(0.2, 0.2, 0.2)), BotLevelSwitch(1)),
                            (button_with_bg(ga, "Medium", Color::srgb(0.2, 0.2, 0.2)), BotLevelSwitch(2)),
                            (button_with_bg(ga, "Hard", Color::srgb(0.2, 0.2, 0.2)), BotLevelSwitch(3)),
                            (
                                Node {
                                    width: Val::Px(100.0),
                                    ..default()
                                },
                                Text::new("last level coming soon"),
                                TextFont {
                                    font: ga.mono_font.clone_weak(),
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
                    h2(ga, "Grid Size"),
                    (
                        Node {
                            margin: UiRect::top(Val::Px(10.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        children![
                            p(ga, "width: "),
                            (
                                left_button(ga),
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
                            (p(ga, " 6"), WidthText),
                            (
                                right_button(ga),
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
                            p(ga, "height: "),
                            (
                                left_button(ga),
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
                            (p(ga, " 6"), HeightText),
                            (
                                right_button(ga),
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
                    h2(ga, "Flash intensity"),
                    (
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        children![
                            (
                                left_button(ga),
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
                            (p(ga, "0.3"), FlashIntensityText),
                            (
                                right_button(ga),
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
            back_to_main_menu::<SettingsUiTree>(ga)
        ],
    )
}
