use bevy::prelude::*;

use super::{RulesUiTree, support::*};

#[derive(Resource, Deref)]
pub struct RulesPageNumber(pub usize);

pub fn menu(ga: &GameAssets) -> impl Bundle {
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
