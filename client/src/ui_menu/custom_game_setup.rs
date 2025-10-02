use bevy::prelude::*;

use crate::{ui_menu::{BotLevelSwitch, PlayModeSwitch}, Config};

use super::{CustomGameSetupUiTree, support::*};

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
                    h2(ga, "Player Config"),
                    (
                        Node {
                            margin: UiRect::top(Val::Px(20.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
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
        ]
    )
}
