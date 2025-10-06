use bevy::prelude::*;
use bevy_defer::{AsyncCommandsExtension as _, AsyncWorld, fetch};

use crate::MainState;
pub use crate::{
    GameAssets,
    anim::{AnimateBackgroundColor, TargetUiOpacity},
};

pub use bevy::ui_widgets::observe;

pub fn back_to_main_menu<T: Component>(ga: &GameAssets) -> impl Bundle {
    (
        Node {
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        },
        Button,
        p(ga, "Back to main menu"),
        Outline::new(Val::Px(5.0), Val::Px(5.0), Color::WHITE),
        BorderRadius::all(Val::Px(5.0)),
        observe(
            |_: On<Pointer<Click>>,
             mut commands: Commands,
             mut next_state: ResMut<NextState<MainState>>,
             mut ui_opacity: ResMut<TargetUiOpacity>,
             ui_tree: Query<Entity, With<T>>| {
                next_state.set(MainState::Menu);
                ui_opacity.0 = 0.0;
                let ui_tree = ui_tree.single().unwrap();
                commands.spawn_task(move || async move {
                    AsyncWorld.sleep(1.0).await;
                    fetch!(ui_tree, Visibility).get_mut(|x| *x = Visibility::Hidden)?;
                    Ok(())
                });
            },
        ),
    )
}

pub fn h1(ga: &GameAssets, text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font: ga.bold_font.clone(),
            font_size: 60.0,
            ..default()
        },
    )
}

pub fn h2(ga: &GameAssets, text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font: ga.bold_font.clone(),
            font_size: 40.0,
            ..default()
        },
    )
}

pub fn p(ga: &GameAssets, text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font: ga.mono_font.clone(),
            font_size: 20.0,
            ..default()
        },
    )
}

pub fn button_with_bg(ga: &GameAssets, text: impl Into<String>, color: Color) -> impl Bundle {
    (
        Node {
            margin: UiRect::horizontal(Val::Px(5.0)),
            padding: UiRect::axes(Val::Px(3.0), Val::Px(3.0)),
            ..default()
        },
        Button,
        AnimateBackgroundColor,
        BackgroundColor(color),
        Outline::new(Val::Px(3.0), Val::Px(0.0), Color::WHITE),
        BorderRadius::all(Val::Px(5.0)),
        children![(
            Text::new(text),
            TextFont {
                font: ga.mono_font.clone(),
                font_size: 15.0,
                ..default()
            },
        )],
    )
}

pub fn left_button(ga: &GameAssets) -> impl Bundle {
    button_with_bg(ga, "<", Color::srgb(0.2, 0.2, 0.2))
}

pub fn right_button(ga: &GameAssets) -> impl Bundle {
    button_with_bg(ga, ">", Color::srgb(0.2, 0.2, 0.2))
}
