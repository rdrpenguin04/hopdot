use bevy::prelude::*;
use bevy_defer::{AsyncAccess as _, AsyncCommandsExtension as _, AsyncWorld, fetch};

use crate::{GameAssets, MainState, anim::TargetUiOpacity, observe::observe};

#[derive(Component)]
pub struct GameEndUiTree;

#[derive(Component)]
pub struct GameEndText;

pub fn game_end_menu(game_assets: &GameAssets) -> impl Bundle {
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
                Text::new("Player 1 wins!"),
                TextFont {
                    font: game_assets.bold_font.clone_weak(),
                    font_size: 60.0,
                    ..default()
                },
                GameEndText,
            ),
            (
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
                Button,
                Text::new("Back to main menu"),
                TextFont {
                    font: game_assets.mono_font.clone_weak(),
                    font_size: 20.0,
                    ..default()
                },
                Outline::new(Val::Px(5.0), Val::Px(5.0), Color::WHITE),
                BorderRadius::all(Val::Px(5.0)),
                observe(
                    |_: Trigger<Pointer<Click>>,
                     mut commands: Commands,
                     mut next_state: ResMut<NextState<MainState>>,
                     mut ui_opacity: ResMut<TargetUiOpacity>,
                     ui_tree: Query<Entity, With<GameEndUiTree>>| {
                        next_state.set(MainState::Menu);
                        ui_opacity.0 = 0.0;
                        let ui_tree = ui_tree.single().unwrap();
                        commands.spawn_task(move || async move {
                            AsyncWorld.sleep(1.0).await;
                            fetch!(ui_tree, Visibility).get_mut(|x| *x = Visibility::Hidden)?;
                            Ok(())
                        });
                    },
                )
            )
        ],
    )
    // .with_children(|commands| {
    //     commands.spawn((
    //         Node {
    //             margin: UiRect::top(Val::Percent(10.0)),
    //             ..default()
    //         },
    //         Text::new("Player 1 wins!"),
    //         TextFont {
    //             font: game_assets.bold_font.clone_weak(),
    //             font_size: 60.0,
    //             ..default()
    //         },
    //         GameEndText,
    //     ));
    //     commands
    //         .spawn((
    //             Node {
    //                 margin: UiRect::top(Val::Px(10.0)),
    //                 ..default()
    //             },
    //             Button,
    //             Text::new("Back to main menu"),
    //             TextFont {
    //                 font: game_assets.mono_font.clone_weak(),
    //                 font_size: 20.0,
    //                 ..default()
    //             },
    //             Outline::new(Val::Px(5.0), Val::Px(5.0), Color::WHITE),
    //             BorderRadius::all(Val::Px(5.0)),
    //         ))
    //         .observe(
    //             |_: Trigger<Pointer<Click>>,
    //              mut commands: Commands,
    //              mut next_state: ResMut<NextState<MainState>>,
    //              mut ui_opacity: ResMut<TargetUiOpacity>,
    //              ui_tree: Query<Entity, With<GameEndUiTree>>| {
    //                 next_state.set(MainState::Menu);
    //                 ui_opacity.0 = 0.0;
    //                 let ui_tree = ui_tree.single().unwrap();
    //                 commands.spawn_task(move || async move {
    //                     AsyncWorld.sleep(1.0).await;
    //                     fetch!(ui_tree, Visibility).get_mut(|x| *x = Visibility::Hidden)?;
    //                     Ok(())
    //                 });
    //             },
    //         );
    // });
}
