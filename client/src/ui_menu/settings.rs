use bevy::prelude::*;

use crate::FlashIntensity;

use super::{SettingsUiTree, support::*};

pub fn menu(ga: &GameAssets) -> impl Bundle {
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
                    margin: UiRect::vertical(Val::Px(15.0)),
                    display: Display::Block,
                    align_items: AlignItems::Center,
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
            p(ga, "Looking for the game setup options? They're now in the new Start Game menu!"),
            back_to_main_menu::<SettingsUiTree>(ga)
        ],
    )
}
