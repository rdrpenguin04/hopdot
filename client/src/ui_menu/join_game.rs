use bevy::prelude::*;

use crate::{
    GameCode, GameCodeText,
    menu::{MainMenuSubState, MenuState},
    net::{NetManagerMessage, NetServerboundSender, ServerUrl},
    ui_menu::InfoText,
};

use super::{JoinGameUiTree, support::*};

fn letter_button(ga: &GameAssets, letter: char) -> impl Bundle {
    (
        button_default_bg(ga, letter),
        observe(
            move |_: On<Pointer<Click>>, mut game_code: ResMut<GameCode>, mut info_texts: Query<&mut Node, With<InfoText>>| {
                if let Some(x) = &mut game_code.0
                    && x.len() < 4
                {
                    x.push(letter);
                }
                for mut node in &mut info_texts {
                    node.display = Display::None;
                }
            },
        ),
    )
}

pub fn menu(ga: &GameAssets) -> impl Bundle {
    (
        JoinGameUiTree,
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
            h1(ga, "Join game"),
            (
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    (p(ga, "- - - -"), GameCodeText,),
                    (
                        Node {
                            margin: UiRect::top(Val::Px(20.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        children![
                            letter_button(ga, 'A'),
                            letter_button(ga, 'E'),
                            letter_button(ga, 'P'),
                            letter_button(ga, 'O'),
                            letter_button(ga, 'Z'),
                            letter_button(ga, 'X'),
                            letter_button(ga, 'L'),
                            letter_button(ga, 'U'),
                        ],
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
                            letter_button(ga, 'G'),
                            letter_button(ga, 'K'),
                            letter_button(ga, 'I'),
                            letter_button(ga, 'S'),
                            letter_button(ga, 'T'),
                            letter_button(ga, 'V'),
                            letter_button(ga, 'Y'),
                            letter_button(ga, 'N'),
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
                        children![(
                            button_default_bg(ga, "Backspace"),
                            observe(
                                |_: On<Pointer<Click>>, mut game_code: ResMut<GameCode>, mut info_texts: Query<&mut Node, With<InfoText>>| {
                                    if let Some(game_code) = &mut game_code.0 {
                                        game_code.pop();
                                    }
                                    for mut node in &mut info_texts {
                                        node.display = Display::None;
                                    }
                                }
                            ),
                        )],
                    ),
                ],
            ),
            (
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![(
                    button_default_bg(ga, "Join game"),
                    observe(
                        |_: On<Pointer<Click>>,
                         tx: Res<NetServerboundSender>,
                         game_code: Res<GameCode>,
                         server: Res<ServerUrl>,
                         mut info_texts: Query<(&mut Node, &mut Text), With<InfoText>>| {
                            tx.force_send(NetManagerMessage::JoinLobby {
                                code: game_code.0.as_ref().unwrap().clone(),
                                server: server.clone(),
                            })
                            .unwrap();
                            for (mut node, mut text) in &mut info_texts {
                                node.display = Display::Flex;
                                text.0 = "Joining...".into();
                            }
                        }
                    )
                )],
            ),
            (
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    display: Display::None,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                },
                InfoText,
                p(ga, ""),
            ),
            back_to_menu::<JoinGameUiTree>(ga, "Back to online menu", MenuState::Main(Some(MainMenuSubState::Online))),
        ],
    )
}
