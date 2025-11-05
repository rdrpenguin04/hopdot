use bevy::prelude::*;

use crate::{
    menu::{MainMenuSubState, MenuState},
    net::{GameSettings, NetManagerMessage, NetServerboundSender, ServerUrl},
    ui_menu::{HostGameUiTree, InfoText},
};

use super::support::*;

pub fn menu(ga: &GameAssets) -> impl Bundle {
    (
        HostGameUiTree,
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
            h1(ga, "Host game"),
            p(ga, "(customization options coming soon)"),
            (
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![(
                    button_default_bg(ga, "Create game"),
                    observe(
                        |_: On<Pointer<Click>>,
                         tx: Res<NetServerboundSender>,
                         server: Res<ServerUrl>,
                         mut info_texts: Query<(&mut Node, &mut Text), With<InfoText>>| {
                            tx.force_send(NetManagerMessage::HostGame {
                                settings: GameSettings {
                                    capacity: 2,
                                    width: 6,
                                    height: 6,
                                },
                                server: server.clone(),
                            })
                            .unwrap();
                            for (mut node, mut text) in &mut info_texts {
                                node.display = Display::Flex;
                                text.0 = "Creating...".into();
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
            back_to_menu::<HostGameUiTree>(ga, "Back to online menu", MenuState::Main(Some(MainMenuSubState::Online))),
        ],
    )
}
