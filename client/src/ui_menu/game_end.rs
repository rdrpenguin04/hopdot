use bevy::prelude::*;

use super::{GameEndText, GameEndUiTree, support::*};

pub fn menu(ga: &GameAssets) -> impl Bundle {
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

