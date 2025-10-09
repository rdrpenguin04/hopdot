use bevy::prelude::*;

use crate::{
    Config,
    menu::{MainMenuSubState, MenuState},
};

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
                                    |_: On<Pointer<Click>>, mut config: ResMut<Config>, mut width_text: Query<&mut Text, With<WidthText>>| {
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
                                    |_: On<Pointer<Click>>, mut config: ResMut<Config>, mut width_text: Query<&mut Text, With<WidthText>>| {
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
                                    |_: On<Pointer<Click>>, mut config: ResMut<Config>, mut height_text: Query<&mut Text, With<HeightText>>| {
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
                                    |_: On<Pointer<Click>>, mut config: ResMut<Config>, mut height_text: Query<&mut Text, With<HeightText>>| {
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
