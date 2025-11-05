use bevy::prelude::*;

use crate::{
    PlayerConfigEntry,
    menu::{MainMenuSubState, MenuState},
    ui_menu::CustomConfig,
};

use super::{CustomGameSetupUiTree, support::*};

#[derive(Component)]
pub struct PlayerConfigPlusLabel;

#[derive(Component)]
pub struct PlayerConfigMinusLabel;

pub fn menu(ga: &GameAssets) -> impl Bundle {
    #[derive(Component)]
    struct WidthText;
    #[derive(Component)]
    struct HeightText;
    (
        CustomGameSetupUiTree,
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
            h1(ga, "Game setup"),
            (
                Node {
                    margin: UiRect::top(Val::Px(5.0)),
                    display: Display::Block,
                    ..default()
                },
                children![
                    h2(ga, "Players"),
                    player_config(ga, 0),
                    player_config(ga, 1),
                    player_config(ga, 2),
                    player_config(ga, 3),
                    (
                        Node {
                            margin: UiRect::vertical(Val::Px(5.0)),
                            ..default()
                        },
                        children![
                            (
                                PlayerConfigPlusLabel,
                                button_default_bg(ga, "+"),
                                observe(move |_: On<Pointer<Click>>, mut config: ResMut<CustomConfig>| {
                                    for player in config.players.iter_mut() {
                                        if player.is_disabled() {
                                            player.to_human();
                                            break;
                                        }
                                    }
                                })
                            ),
                            (
                                PlayerConfigMinusLabel,
                                button_default_bg(ga, "-"),
                                observe(move |_: On<Pointer<Click>>, mut config: ResMut<CustomConfig>| {
                                    for player in config.players.iter_mut().rev() {
                                        if !player.is_disabled() {
                                            player.to_disabled();
                                            break;
                                        }
                                    }
                                })
                            ),
                        ]
                    )
                ],
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
                                    |_: On<Pointer<Click>>, mut config: ResMut<CustomConfig>, mut width_text: Query<&mut Text, With<WidthText>>| {
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
                                    |_: On<Pointer<Click>>, mut config: ResMut<CustomConfig>, mut width_text: Query<&mut Text, With<WidthText>>| {
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
                                    |_: On<Pointer<Click>>, mut config: ResMut<CustomConfig>, mut height_text: Query<&mut Text, With<HeightText>>| {
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
                                    |_: On<Pointer<Click>>, mut config: ResMut<CustomConfig>, mut height_text: Query<&mut Text, With<HeightText>>| {
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
            back_to_menu::<CustomGameSetupUiTree>(ga, "Back to menu", MenuState::Main(Some(MainMenuSubState::StartGame)))
        ],
    )
}

#[derive(Component)]
pub struct PlayerConfigLabel(usize);

#[derive(Component)]
pub struct PlayerConfigHumanLabel(usize);

#[derive(Component)]
pub struct PlayerConfigBotLabel(usize);

#[derive(Component)]
pub struct PlayerConfigBotLevelLabel {
    player_idx: usize,
    level: usize,
}

fn player_config(ga: &GameAssets, player_idx: usize) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            margin: UiRect::vertical(Val::Px(5.0)),
            ..default()
        },
        PlayerConfigLabel(player_idx),
        children![
            (
                PlayerConfigHumanLabel(player_idx),
                Node {
                    display: Display::Flex,
                    ..default()
                },
                children![(
                    button_default_bg(ga, "Human"),
                    observe(move |_: On<Pointer<Click>>, mut config: ResMut<CustomConfig>| {
                        config.players[player_idx].to_bot();
                    })
                )],
            ),
            (
                PlayerConfigBotLabel(player_idx),
                Node {
                    display: Display::Flex,
                    ..default()
                },
                children![
                    (
                        button_default_bg(ga, "Bot"),
                        observe(move |_: On<Pointer<Click>>, mut config: ResMut<CustomConfig>| {
                            config.players[player_idx].to_human();
                        }),
                    ),
                    (
                        PlayerConfigBotLevelLabel { player_idx, level: 0 },
                        button_default_bg(ga, "Easiest"),
                        observe(move |_: On<Pointer<Click>>, mut config: ResMut<CustomConfig>| {
                            config.players[player_idx].set_level(0);
                        }),
                    ),
                    (
                        PlayerConfigBotLevelLabel { player_idx, level: 1 },
                        button_default_bg(ga, "Easy"),
                        observe(move |_: On<Pointer<Click>>, mut config: ResMut<CustomConfig>| {
                            config.players[player_idx].set_level(1);
                        }),
                    ),
                    (
                        PlayerConfigBotLevelLabel { player_idx, level: 2 },
                        button_default_bg(ga, "Medium"),
                        observe(move |_: On<Pointer<Click>>, mut config: ResMut<CustomConfig>| {
                            config.players[player_idx].set_level(2);
                        }),
                    ),
                    (
                        PlayerConfigBotLevelLabel { player_idx, level: 3 },
                        button_default_bg(ga, "Hard"),
                        observe(move |_: On<Pointer<Click>>, mut config: ResMut<CustomConfig>| {
                            config.players[player_idx].set_level(3);
                        }),
                    ),
                ],
            ),
        ],
    )
}

pub fn render_player_config(
    mut config: ResMut<CustomConfig>,
    mut node: ParamSet<(
        Query<&mut Node, With<PlayerConfigMinusLabel>>,
        Query<&mut Node, With<PlayerConfigPlusLabel>>,
        Query<(&mut Node, &PlayerConfigLabel)>,
        Query<(&mut Node, &PlayerConfigHumanLabel)>,
        Query<(&mut Node, &PlayerConfigBotLabel)>,
    )>,
    mut button_colors: Query<(&mut BackgroundColor, &PlayerConfigBotLevelLabel)>,
) {
    if config.players.is_empty() {
        return;
    } else if config.players.len() < 4 {
        assert!(config.players.len() == 2);
        config.players.push(PlayerConfigEntry::default_for_player(3));
        config.players.push(PlayerConfigEntry::default_for_player(4));
    }
    node.p0().single_mut().unwrap().display = if config.players[2].is_disabled() { Display::None } else { Display::Block };
    node.p1().single_mut().unwrap().display = if config.players[3].is_disabled() { Display::Block } else { Display::None };
    for (mut node, player) in &mut node.p2() {
        node.display = if config.players[player.0].is_disabled() {
            Display::None
        } else {
            Display::Flex
        };
    }
    for (mut node, player) in &mut node.p3() {
        node.display = if config.players[player.0].is_human() { Display::Flex } else { Display::None };
    }
    for (mut node, player) in &mut node.p4() {
        node.display = if config.players[player.0].is_bot() { Display::Flex } else { Display::None };
    }
    for (mut color, PlayerConfigBotLevelLabel { player_idx, level }) in &mut button_colors {
        color.0 = if config.players[*player_idx].level() == *level {
            Color::srgba(0.4, 0.4, 0.4, color.0.alpha())
        } else {
            Color::srgba(0.2, 0.2, 0.2, color.0.alpha())
        };
    }
}
