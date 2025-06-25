use std::time::Duration;

use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{
    CreditsUiTree, MainState, RulesUiTree, SettingsUiTree, add_hover_observers,
    anim::{SmoothingSettings, TargetTransform, TargetUiOpacity},
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect, SubStates)]
#[source(MainState = MainState::Menu)]
pub enum MenuState {
    #[default]
    Main,
    Pause,
}

#[derive(Component, Default, Reflect)]
#[require(
    TargetTransform(Transform::default()),
    SmoothingSettings { translation_decay_rate: 3.0, scale_decay_rate: 10.0, ..default() },
    Visibility::Hidden,
    Transform,
)]
#[component(on_insert = insert_menu_element)]
#[reflect(Component, Default)]
pub struct MenuElement {
    pub for_menu: MenuState,
    pub target: Option<Transform>,
    pub menu_action: Option<&'static str>,
    pub side: f32, // -1.0 or 1.0, probably
}

fn insert_menu_element(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let transform = *world.get(entity).unwrap();
    let mut menu_element = world.get_mut::<MenuElement>(entity).unwrap();
    if menu_element.target.is_none() {
        menu_element.target = Some(transform);
    }
    let menu_action = menu_element.menu_action.clone();
    match menu_action {
        Some(action) => {
            let mut commands = world.commands();
            let mut entity_commands = commands.entity(entity);
            add_hover_observers(&mut entity_commands);
            match action {
                "credits" => {
                    entity_commands.observe(
                        |_: Trigger<Pointer<Click>>,
                         mut next_state: ResMut<NextState<MainState>>,
                         mut credits_ui_tree: Query<&mut Visibility, With<CreditsUiTree>>,
                         mut ui_opacity: ResMut<TargetUiOpacity>| {
                            next_state.set(MainState::DimForUi);
                            *credits_ui_tree.single_mut().unwrap() = Visibility::Visible;
                            ui_opacity.0 = 1.0;
                        },
                    );
                }
                "rules" => {
                    entity_commands.observe(
                        |_: Trigger<Pointer<Click>>,
                         mut next_state: ResMut<NextState<MainState>>,
                         mut settings_ui_tree: Query<&mut Visibility, With<RulesUiTree>>,
                         mut ui_opacity: ResMut<TargetUiOpacity>| {
                            next_state.set(MainState::DimForUi);
                            *settings_ui_tree.single_mut().unwrap() = Visibility::Visible;
                            ui_opacity.0 = 1.0;
                        },
                    );
                }
                "settings" => {
                    entity_commands.observe(
                        |_: Trigger<Pointer<Click>>,
                         mut next_state: ResMut<NextState<MainState>>,
                         mut settings_ui_tree: Query<&mut Visibility, With<SettingsUiTree>>,
                         mut ui_opacity: ResMut<TargetUiOpacity>| {
                            next_state.set(MainState::DimForUi);
                            *settings_ui_tree.single_mut().unwrap() = Visibility::Visible;
                            ui_opacity.0 = 1.0;
                        },
                    );
                }
                "start-game" => {
                    entity_commands.observe(|_: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<MainState>>| {
                        next_state.set(MainState::Game);
                    });
                }
                x => {
                    warn!("unknown action: {x}");
                }
            }
        }
        None => {}
    }
}

fn switch_menus(
    cur_menu: Option<Res<State<MenuState>>>,
    mut prev_menu: Local<Option<MenuState>>,
    menu_elements: Query<(&MenuElement, &mut TargetTransform, &mut Transform, &mut Visibility)>,
) {
    for (el, mut target, mut transform, mut visibility) in menu_elements {
        if *prev_menu == Some(el.for_menu) {
            // Fly out
            let Some(mut new_transform) = el.target else {
                continue;
            };
            new_transform.translation += vec3(0.0, 0.0, -20.0);
            target.0 = new_transform;
        }
        if let Some(ref cur_menu) = cur_menu
            && **cur_menu == el.for_menu
        {
            // Fly in
            let Some(mut new_transform) = el.target else {
                continue;
            };
            new_transform.translation += el.side * vec3(20.0, 0.0, 0.0);
            *transform = new_transform;
            target.0 = el.target.unwrap();
            *visibility = Visibility::Inherited;
        }
    }
    *prev_menu = cur_menu.map(|x| **x);
}

fn cleanup_menus(
    cur_menu: Option<Res<State<MenuState>>>,
    mut prev_menu: Local<Option<MenuState>>,
    menu_elements: Query<(&MenuElement, &mut Visibility)>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    timer.set_duration(Duration::from_secs(1));
    if cur_menu.as_ref().map(|x| ***x) != *prev_menu {
        timer.reset();
    }
    timer.tick(time.delta());
    if timer.just_finished() {
        for (el, mut visibility) in menu_elements {
            if cur_menu.as_ref().map(|x| ***x) != Some(el.for_menu) {
                *visibility = Visibility::Hidden;
            }
        }
    }
    *prev_menu = cur_menu.map(|x| **x);
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (|key_input: Res<ButtonInput<KeyCode>>, mut main_state: ResMut<NextState<MainState>>| {
            if key_input.just_pressed(KeyCode::Escape) {
                main_state.set(MainState::Game);
            }
        })
        .run_if(in_state(MenuState::Pause)),
    )
    .add_systems(Update, switch_menus.run_if(state_changed::<MenuState>.or(state_changed::<MainState>)))
    .add_systems(Update, cleanup_menus)
    .add_sub_state::<MenuState>()
    .register_type::<MenuElement>();
}
