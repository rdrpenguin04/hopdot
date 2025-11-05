use std::{collections::HashMap, time::Duration};

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{
    Config, GameCode, MainState, NeedNewBoard, add_hover_observers,
    anim::{SmoothingSettings, TargetMaterialColor, TargetTransform, TargetUiOpacity},
    ui_menu::{CreditsUiTree, CustomConfig, CustomGameSetupUiTree, HostGameUiTree, InfoText, JoinGameUiTree, RulesUiTree, SettingsUiTree},
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect)]
pub enum MainMenuSubState {
    #[default]
    Main,
    StartGame,
    Online,
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
        matches!(self, Self::Main(Some(MainMenuSubState::StartGame | MainMenuSubState::Online))) && *menu == Self::Main(Some(MainMenuSubState::Main))
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

#[derive(Clone, Component, Default, Reflect)]
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

#[derive(Component, Default, Reflect)]
#[reflect(Component, Default)]
#[component(on_insert = insert_menu_radio)]
pub struct MenuRadio {
    pub option_name: String,
    pub id: usize,
}

#[derive(Clone, Copy, Reflect)]
pub enum RadioState {
    Enabled(usize),
    Disabled(usize),
}

// Dirty special case
#[derive(Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub struct ContinueButton;

impl RadioState {
    #[must_use]
    pub const fn value(&self) -> usize {
        match self {
            Self::Enabled(x) | Self::Disabled(x) => *x,
        }
    }

    #[must_use]
    pub const fn value_opt(&self) -> Option<usize> {
        match self {
            Self::Enabled(x) => Some(*x),
            Self::Disabled(_) => None,
        }
    }

    pub const fn disable(&mut self) {
        *self = Self::Disabled(self.value());
    }

    pub const fn enable(&mut self) {
        *self = Self::Enabled(self.value());
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct MenuRadios {
    pub radios: HashMap<String, RadioState>,
}

const SELECTED_BACK: Color = Color::Srgba(Srgba::rgb(0.0, 1.0, 0.0));
const DESELECTED_BACK: Color = Color::Srgba(Srgba::rgb(0.0, 0.0, 1.0));
const DISABLED_BACK: Color = Color::Srgba(Srgba::rgb(0.4, 0.4, 0.45));

const SELECTED_TEXT: Color = Color::Srgba(Srgba::rgb(0.0, 0.0, 1.0));
const DESELECTED_TEXT: Color = Color::Srgba(Srgba::rgb(0.0, 1.0, 0.0));
const DISABLED_TEXT: Color = Color::Srgba(Srgba::rgb(0.2, 0.2, 0.25));

#[derive(Component)]
#[require(TargetMaterialColor = TargetMaterialColor(DESELECTED_BACK))]
pub struct MenuRadioBack(pub MenuRadio);

#[derive(Component)]
#[require(TargetMaterialColor = TargetMaterialColor(DESELECTED_TEXT))]
pub struct MenuRadioText(pub MenuRadio);

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
                    |_: On<Pointer<Click>>,
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
                    |_: On<Pointer<Click>>,
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
                    |_: On<Pointer<Click>>,
                     mut next_state: ResMut<NextState<MainState>>,
                     mut settings_ui_tree: Query<&mut Visibility, With<SettingsUiTree>>,
                     mut ui_opacity: ResMut<TargetUiOpacity>| {
                        next_state.set(MainState::DimForUi);
                        *settings_ui_tree.single_mut().unwrap() = Visibility::Visible;
                        ui_opacity.0 = 1.0;
                    },
                );
            }
            "custom-setup" => {
                entity_commands.observe(
                    |_: On<Pointer<Click>>,
                     mut next_state: ResMut<NextState<MainState>>,
                     mut game_setup_ui_tree: Query<&mut Visibility, With<CustomGameSetupUiTree>>,
                     mut ui_opacity: ResMut<TargetUiOpacity>,
                     config: Res<Config>,
                     mut custom_config: ResMut<CustomConfig>| {
                        next_state.set(MainState::DimForUi);
                        *game_setup_ui_tree.single_mut().unwrap() = Visibility::Visible;
                        ui_opacity.0 = 1.0;
                        if custom_config.players.is_empty() {
                            custom_config.clone_from(&config);
                        }
                    },
                );
            }
            "start-game" => {
                entity_commands.observe(
                    |_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<MenuState>>, mut game_code: ResMut<GameCode>, mut radios: ResMut<MenuRadios>| {
                        next_menu.set(MenuState::Main(Some(MainMenuSubState::StartGame)));
                        game_code.0 = None;
                        if let Some(game_type) = radios.radios.get_mut("game-type") {
                            game_type.enable();
                        }
                    },
                );
            }
            "go" => {
                entity_commands.observe(
                    |_: On<Pointer<Click>>,
                     mut next_state: ResMut<NextState<MainState>>,
                     menu: Option<Res<State<MenuState>>>,
                     mut new_board: ResMut<NextState<NeedNewBoard>>| {
                        if matches!(menu.map(|x| **x), Some(MenuState::Main(Some(MainMenuSubState::StartGame)))) {
                            new_board.set(NeedNewBoard(true));
                        }
                        next_state.set(MainState::Game);
                    },
                );
            }
            "main-menu" => {
                entity_commands.observe(|_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<MenuState>>| {
                    next_menu.set(MenuState::Main(Some(MainMenuSubState::Main)));
                });
            }
            "online" => {
                entity_commands.observe(
                    |_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<MenuState>>, mut game_code: ResMut<GameCode>| {
                        next_menu.set(MenuState::Main(Some(MainMenuSubState::Online)));
                        game_code.0 = Some(String::new());
                    },
                );
            }
            "host" => {
                entity_commands.observe(
                    |_: On<Pointer<Click>>,
                     mut next_state: ResMut<NextState<MainState>>,
                     mut host_game_ui_tree: Query<&mut Visibility, With<HostGameUiTree>>,
                     mut ui_opacity: ResMut<TargetUiOpacity>,
                     mut info_texts: Query<&mut Node, With<InfoText>>| {
                        next_state.set(MainState::DimForUi);
                        *host_game_ui_tree.single_mut().unwrap() = Visibility::Visible;
                        ui_opacity.0 = 1.0;
                        for mut node in &mut info_texts {
                            node.display = Display::None;
                        }
                    },
                );
            }
            "join" => {
                entity_commands.observe(
                    |_: On<Pointer<Click>>,
                     mut next_state: ResMut<NextState<MainState>>,
                     mut join_game_ui_tree: Query<&mut Visibility, With<JoinGameUiTree>>,
                     mut ui_opacity: ResMut<TargetUiOpacity>,
                     mut info_texts: Query<&mut Node, With<InfoText>>| {
                        next_state.set(MainState::DimForUi);
                        *join_game_ui_tree.single_mut().unwrap() = Visibility::Visible;
                        ui_opacity.0 = 1.0;
                        for mut node in &mut info_texts {
                            node.display = Display::None;
                        }
                    },
                );
            }
            "game-type-changed" | "game-difficulty-changed" => {}
            x => {
                warn!("unknown action: {x}");
            }
        }
    }
}

fn insert_menu_radio(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let menu_radio = world.get::<MenuRadio>(entity).unwrap();
    let option_name = menu_radio.option_name.clone();
    let id = menu_radio.id;
    world
        .get_resource_mut::<MenuRadios>()
        .unwrap()
        .radios
        .entry(option_name.clone())
        .or_insert(RadioState::Enabled(0));
    let children = world.get::<Children>(entity).unwrap().iter().collect::<Vec<_>>();
    for (i, child) in children.into_iter().enumerate() {
        let is_text = i == 1; // Dirty hack because the components that *would* let us check accurately don't exist yet :)
        let mut commands = world.commands();
        let menu_radio = MenuRadio {
            option_name: option_name.clone(),
            id,
        };
        if is_text {
            commands.entity(child).insert(MenuRadioText(menu_radio));
        } else {
            commands.entity(child).insert(MenuRadioBack(menu_radio));
        }
    }
    world
        .commands()
        .entity(entity)
        .observe(move |_: On<Pointer<Click>>, mut menu_radios: ResMut<MenuRadios>| {
            if let RadioState::Enabled(x) = menu_radios.radios.get_mut(&option_name).unwrap() {
                *x = id;
            }
        });
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

// Dirty special case
fn handle_continue_button(
    cur_menu: Option<Res<State<MenuState>>>,
    mut continue_button: Query<&mut Visibility, With<ContinueButton>>,
    need_new_board: Res<State<NeedNewBoard>>,
) {
    if matches!(cur_menu.as_ref().map(|x| ***x), Some(MenuState::Main(Some(MainMenuSubState::Main))))
        && let Ok(mut continue_button) = continue_button.single_mut()
    {
        *continue_button = if need_new_board.0 { Visibility::Hidden } else { Visibility::Inherited };
    }
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

fn animate_menu_radios(
    menu_radio_backs: Query<(&mut TargetMaterialColor, &MenuRadioBack), Without<MenuRadioText>>,
    menu_radio_texts: Query<(&mut TargetMaterialColor, &MenuRadioText), Without<MenuRadioBack>>,
    menu_radios: Res<MenuRadios>,
) {
    if menu_radios.is_changed() {
        for (mut color, back) in menu_radio_backs {
            if let RadioState::Enabled(id) = menu_radios.radios[&back.0.option_name] {
                if id == back.0.id {
                    color.0 = SELECTED_BACK;
                } else {
                    color.0 = DESELECTED_BACK;
                }
            } else {
                color.0 = DISABLED_BACK;
            }
        }
        for (mut color, back) in menu_radio_texts {
            if let RadioState::Enabled(id) = menu_radios.radios[&back.0.option_name] {
                if id == back.0.id {
                    color.0 = SELECTED_TEXT;
                } else {
                    color.0 = DESELECTED_TEXT;
                }
            } else {
                color.0 = DISABLED_TEXT;
            }
        }
    }
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
    .add_systems(
        Update,
        (switch_menus, handle_continue_button)
            .chain()
            .run_if(state_changed::<MenuState>.or(state_changed::<MainState>)),
    )
    .add_systems(Update, animate_menu_radios)
    .add_systems(Update, cleanup_menus)
    .add_sub_state::<MenuState>()
    .init_resource::<MenuRadios>()
    .register_type::<MenuElement>()
    .register_type::<MenuRadio>()
    .register_type::<MenuRadios>();
}
