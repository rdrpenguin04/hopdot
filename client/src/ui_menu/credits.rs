use bevy::prelude::*;

use super::{CreditsUiTree, support::*};

pub fn menu(ga: &GameAssets) -> impl Bundle {
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
                    font: ga.mono_font.clone(),
                    font_size: 7.0,
                    ..default()
                },
            )
        ],
    )
}
