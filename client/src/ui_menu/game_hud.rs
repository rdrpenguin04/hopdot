use crate::{Config, CurrentTurn, GameOperation, MainState, ai::Ais, anim::TargetMaterialColor, menu::MenuState, ui_menu::GameHudUiTree};

use super::support::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct HudInfoText;

pub fn menu(ga: &GameAssets) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Pickable::IGNORE,
        GameHudUiTree,
        Visibility::Hidden,
        children![
            (
                Node {
                    display: Display::Block,
                    flex_direction: FlexDirection::Row,
                    align_self: AlignSelf::FlexEnd,
                    padding: UiRect::all(px(15.0)),
                    ..default()
                },
                children![(
                    button_default_bg(ga, "Pause"),
                    observe(
                        |_: On<Pointer<Click>>, mut main_state: ResMut<NextState<MainState>>, mut menu_state: ResMut<NextState<MenuState>>| {
                            main_state.set(MainState::Menu);
                            menu_state.set(MenuState::Pause);
                        }
                    ),
                )]
            ),
            (Node { flex_grow: 1.0, ..default() }, Pickable::IGNORE),
            (
                Node {
                    display: Display::Block,
                    flex_direction: FlexDirection::Row,
                    align_self: AlignSelf::FlexEnd,
                    padding: UiRect::all(px(15.0)),
                    ..default()
                },
                HudInfoText,
                TargetMaterialColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                p(ga, ""),
            ),
        ],
    )
}

pub fn run_menu(
    hud_info_text: Query<(&mut Node, &mut Text, &mut TargetMaterialColor), With<HudInfoText>>,
    game_op: Res<State<GameOperation>>,
    current_turn: Res<State<CurrentTurn>>,
    config: Res<Config>,
    ais: Res<Ais>,
) {
    for (mut node, mut text, mut target_color) in hud_info_text {
        match game_op.get() {
            GameOperation::Animating => {
                node.align_self = AlignSelf::FlexEnd;
                target_color.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
            }
            GameOperation::Human => {
                node.align_self = AlignSelf::FlexEnd;
                let Some(player) = config.players.get(current_turn.0 - 1) else {
                    dbg!("invalid player, bailing out");
                    continue;
                };
                target_color.0 = player.color();
                text.0 = format!("{}, your turn", player.name());
            }
            GameOperation::Bot => {
                node.align_self = AlignSelf::FlexEnd;
                let Some(player) = config.players.get(current_turn.0 - 1) else {
                    dbg!("invalid player, bailing out");
                    continue;
                };
                target_color.0 = player.color();
                text.0 = format!("{} is thinking...", ais[player.level()].name());
            }
            GameOperation::OnlinePlayer => {
                node.align_self = AlignSelf::FlexEnd;
                let Some(player) = config.players.get(current_turn.0 - 1) else {
                    dbg!("invalid player, bailing out");
                    continue;
                };
                target_color.0 = player.color();
                text.0 = format!("Waiting for {}...", player.name());
            }
            GameOperation::Connecting => {
                node.align_self = AlignSelf::Center;
                target_color.0 = Color::srgba(1.0, 1.0, 1.0, 1.0);
                text.0 = "Connecting...".into();
            }
        }
    }
}
