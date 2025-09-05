use std::time::Duration;

use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{
    MainState, add_hover_observers,
    anim::{SmoothingSettings, TargetTransform, TargetUiOpacity},
    ui_menu::{CreditsUiTree, RulesUiTree, SettingsUiTree},
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect)]
pub enum MainMenuSubState {
    #[default]
    Main,
    StartGame,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect, SubStates)]
#[source(MainState = MainState::Menu)]
pub enum MenuState {
    Main(Option<MainMenuSubState>),
    Pause,
}

#[allow(dead_code)] // Currently not using some directions
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlyDirection {
    North,
    South,
    West,
    East,
    Side,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlyAction {
    Stay,
    FlyFrom(FlyDirection),
    FlyTo(FlyDirection),
}

impl MenuState {
    fn eq_top_menu(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }

    fn shows_for_menu(&self, menu: &Self) -> bool {
        match (self, menu) {
            (Self::Main(None), Self::Main(_)) => true,
            (Self::Main(x), Self::Main(y)) if x == y => true,
            (Self::Pause, Self::Pause) => true,
            _ => false,
        }
    }

    fn is_deeper_than(&self, menu: &Self) -> bool {
        // TODO: generalize
        *self == Self::Main(Some(MainMenuSubState::StartGame)) && *menu == Self::Main(Some(MainMenuSubState::Main))
    }

    fn is_top_level(&self) -> bool {
        matches!(self, Self::Main(None) | Self::Pause)
    }

    fn action_for_menu(&self, prev: Option<&Self>, menu: Option<&Self>) -> FlyAction {
        use FlyAction::*;
        use FlyDirection::*;

        match (self, prev, menu) {
            // Menu hasn't changed
            (_, y, z) if y == z => Stay,
            // Entering menu, we're in the new menu
            (x, None, Some(z)) if x.shows_for_menu(z) => FlyFrom(Side),
            // Exiting menu, we're in the old menu
            (x, Some(y), None) if x.shows_for_menu(y) => FlyTo(North),
            // Top-level menu has changed, we're in the new menu
            (x, Some(y), Some(z)) if x.shows_for_menu(z) && !y.eq_top_menu(z) => FlyFrom(Side),
            // Top-level menu has changed, we're in the old menu
            (x, Some(y), Some(z)) if x.shows_for_menu(y) && !y.eq_top_menu(z) => FlyTo(North),
            // Changing submenu level and we don't care
            (x, Some(y), Some(z)) if x.is_top_level() && y.eq_top_menu(z) => Stay,
            // Going to deeper menu level, we're in the new menu
            (x, Some(y), Some(z)) if x.shows_for_menu(z) && z.is_deeper_than(y) => FlyFrom(Side),
            // Going to deeper menu level, we're in the old menu
            (x, Some(y), Some(z)) if x.shows_for_menu(y) && z.is_deeper_than(y) => FlyTo(South),
            // Going to shallower menu level, we're in the new menu
            (x, Some(y), Some(z)) if x.shows_for_menu(z) && !z.is_deeper_than(y) => FlyFrom(South),
            // Going to shallower menu level, we're in the old menu
            (x, Some(y), Some(z)) if x.shows_for_menu(y) && !z.is_deeper_than(y) => FlyTo(Side),
            // We're not involved in this transition
            _ => Stay,
        }
    }
}

impl Default for MenuState {
    fn default() -> Self {
        Self::Main(Some(MainMenuSubState::Main))
    }
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
    pub menu_action: Option<String>,
    pub side: f32, // -1.0 or 1.0, probably
}

impl MenuElement {
    fn vec_from_dir(&self, dir: FlyDirection) -> Vec3 {
        match dir {
            FlyDirection::North => vec3(0.0, 0.0, -20.0),
            FlyDirection::South => vec3(0.0, 0.0, 20.0),
            FlyDirection::West => vec3(-20.0, 0.0, 0.0),
            FlyDirection::East => vec3(20.0, 0.0, 0.0),
            FlyDirection::Side => self.side * vec3(20.0, 0.0, 0.0),
        }
    }
}

fn insert_menu_element(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let transform = *world.get(entity).unwrap();
    let mut menu_element = world.get_mut::<MenuElement>(entity).unwrap();
    if menu_element.target.is_none() {
        menu_element.target = Some(transform);
    }
    let menu_action = menu_element.menu_action.clone();
    if let Some(action) = menu_action.as_deref() {
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
                entity_commands.observe(|_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<MenuState>>| {
                    next_menu.set(MenuState::Main(Some(MainMenuSubState::StartGame)));
                });
                // entity_commands.observe(|_: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<MainState>>| {
                //     next_state.set(MainState::Game);
                // });
            }
            x => {
                warn!("unknown action: {x}");
            }
        }
    }
}

fn switch_menus(
    cur_menu: Option<Res<State<MenuState>>>,
    mut prev_menu: Local<Option<MenuState>>,
    menu_elements: Query<(&MenuElement, &mut TargetTransform, &mut Transform, &mut Visibility)>,
) {
    for (el, mut target, mut transform, mut visibility) in menu_elements {
        match el.for_menu.action_for_menu(prev_menu.as_ref(), cur_menu.as_ref().map(|x| &***x)) {
            FlyAction::FlyTo(dir) => {
                let Some(mut new_transform) = el.target else {
                    continue;
                };
                new_transform.translation += el.vec_from_dir(dir);
                target.0 = new_transform;
            }
            FlyAction::FlyFrom(dir) => {
                let Some(mut new_transform) = el.target else {
                    continue;
                };
                new_transform.translation += el.vec_from_dir(dir);
                *transform = new_transform;
                target.0 = el.target.unwrap();
                *visibility = Visibility::Inherited;
            }
            FlyAction::Stay => {}
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
            if cur_menu.as_ref().map(|x| el.for_menu.shows_for_menu(x)) != Some(true) {
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
