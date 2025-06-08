pub mod ai;
pub mod anim;

use std::{
    ops::{Index, IndexMut},
    time::Duration,
};

use bevy::{
    core_pipeline::{
        bloom::Bloom,
        experimental::taa::{TemporalAntiAliasPlugin, TemporalAntiAliasing},
    },
    pbr::ShadowFilteringMethod,
    platform::collections::HashSet,
    prelude::*,
};
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;

use crate::anim::{Bouncing, SmoothingSettings, TargetTransform};

#[derive(Resource, Reflect)]
pub struct GameAssets {
    table_scene: Handle<Scene>,
    dot_mesh: Handle<Mesh>,
    tile_mesh: Handle<Mesh>,
    splash_mesh: Handle<Mesh>,
    dot_color: Handle<StandardMaterial>,
    splash_material: Handle<StandardMaterial>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let table_scene =
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/table.glb"));

        let splash_image = asset_server.load("tex/splash.png");

        let mut meshes = world.resource_mut::<Assets<_>>();
        let dot_mesh = meshes.add(Sphere::new(0.1));
        let tile_mesh = meshes.add(Cuboid::new(0.95, 0.1, 0.95));
        let splash_mesh = meshes.add(Rectangle::new(7.2, 4.0));

        let mut materials = world.resource_mut::<Assets<_>>();
        let dot_color = materials.add(Color::srgb(1.0, 1.0, 1.0));
        let splash_material = materials.add(StandardMaterial {
            base_color: Color::linear_rgba(1.0, 1.0, 1.0, 0.0),
            base_color_texture: Some(splash_image.clone()),
            emissive: LinearRgba::new(0.0, 0.0, 0.0, 1.0),
            emissive_texture: Some(splash_image),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        Self {
            table_scene,
            dot_mesh,
            tile_mesh,
            splash_mesh,
            dot_color,
            splash_material,
        }
    }
}

#[derive(Clone, Component, Copy, Default, Reflect)]
pub struct CellColor {
    player: usize, // 0 = off, 1 = player 1, etc
}

#[derive(Component, Reflect)]
#[relationship_target(relationship = Dot)]
#[require(CellColor)]
pub struct DotCell {
    dots: Vec<Entity>,
}

#[derive(Component, Reflect)]
pub struct DotCellMeta {
    capacity: usize, // On a square grid: 2 for corners, 3 for edges, 4 for middle tiles
}

#[derive(Component, Reflect)]
#[relationship(relationship_target = DotCell)]
pub struct Dot(Entity);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect, States)]
pub enum MainState {
    #[default]
    Splash,
    Menu,
    Game,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect, States)]
pub enum GameOperation {
    #[default]
    Animating,
    Human,
    Bot,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect, SubStates)]
#[source(GameOperation = GameOperation::Animating)]
pub struct EndGame {
    game_ended: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect, States)]
pub struct CurrentTurn(usize);

#[derive(Clone, Reflect)]
pub enum PlayerConfigEntry {
    Human { color: Color, name: String },
    Bot { color: Color, level: usize },
}

#[derive(Resource, Reflect)]
pub struct Config {
    pub players: Vec<PlayerConfigEntry>,
    pub grid_size: (usize, usize),
}

#[derive(Default, Resource, Reflect)]
pub struct CellGrid {
    grid: Vec<Entity>, // would be technically more efficient to use Box<[Entity]>, but oh well
    width: usize,
}

#[derive(Component)]
pub struct Orbiter;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect, States)]
pub struct NeedNewBoard(bool);

impl Default for NeedNewBoard {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct GridTray;

#[derive(Component)]
#[require(TargetTransform(Transform::default()), SmoothingSettings { translation_decay_rate: 3.0, scale_decay_rate: 10.0, ..default() }, Visibility::Hidden)]
pub struct MenuElement {
    for_menu: MenuState,
    target: Transform,
    side: f32, // -1.0 or 1.0, probably
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect, SubStates)]
#[source(MainState = MainState::Menu)]
pub enum MenuState {
    #[default]
    Main,
    Pause,
}

impl CellGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: vec![Entity::PLACEHOLDER; width * height],
            width,
        }
    }

    pub fn new_inplace(&mut self, width: usize, height: usize) {
        self.grid = vec![Entity::PLACEHOLDER; width * height];
        self.width = width;
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.grid.len() / self.width
    }

    pub fn iter(&self) -> core::slice::ChunksExact<'_, Entity> {
        self.grid.chunks_exact(self.width)
    }
}

impl Index<usize> for CellGrid {
    type Output = [Entity];

    fn index(&self, index: usize) -> &[Entity] {
        &self.grid[(index * self.width)..((index + 1) * self.width)]
    }
}

impl IndexMut<usize> for CellGrid {
    fn index_mut(&mut self, index: usize) -> &mut [Entity] {
        &mut self.grid[(index * self.width)..((index + 1) * self.width)]
    }
}

impl<'a> IntoIterator for &'a CellGrid {
    type Item = &'a [Entity];

    type IntoIter = core::slice::ChunksExact<'a, Entity>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(MeshPickingPlugin)
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_plugins(anim::plugin);

    #[cfg(not(target_family = "wasm"))]
    {
        app.add_plugins(TemporalAntiAliasPlugin);
    }

    #[cfg(debug_assertions)]
    {
        use bevy_inspector_egui::{
            bevy_egui::EguiPlugin,
            quick::{StateInspectorPlugin, WorldInspectorPlugin},
        };

        app.add_plugins((
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            StateInspectorPlugin::<MainState>::default(),
            StateInspectorPlugin::<CurrentTurn>::default(),
            StateInspectorPlugin::<NeedNewBoard>::default(),
            WorldInspectorPlugin::default(),
        ));
    }

    app.init_resource::<GameAssets>()
        .init_resource::<CellGrid>()
        .insert_resource(AmbientLight {
            brightness: 1000.0,
            ..default()
        })
        .insert_resource(ClearColor(Color::srgb_u8(33, 34, 37)))
        .insert_resource(Config {
            players: vec![
                // PlayerConfigEntry::Human {
                //     color: Color::srgb(0.0, 1.0, 0.0),
                //     name: "Player 1".into(),
                // },
                // PlayerConfigEntry::Human {
                //     color: Color::srgb(0.0, 0.0, 1.0),
                //     name: "Player 2".into(),
                // },
                PlayerConfigEntry::Bot {
                    color: Color::srgb(1.0, 0.0, 0.0),
                    level: 2,
                },
                PlayerConfigEntry::Bot {
                    color: Color::srgb(0.0, 1.0, 1.0),
                    level: 2,
                },
                PlayerConfigEntry::Bot {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    level: 2,
                },
                PlayerConfigEntry::Bot {
                    color: Color::srgb(0.0, 0.0, 1.0),
                    level: 2,
                },
            ],
            grid_size: (12, 12),
        })
        .add_systems(Startup, setup_scene)
        .add_systems(OnEnter(MainState::Game), fly_in_game)
        .add_systems(OnExit(MainState::Game), fly_out_game)
        .add_systems(
            Update,
            switch_menus.run_if(state_changed::<MenuState>.or(state_changed::<MainState>)),
        )
        .add_systems(
            OnEnter(MainState::Menu),
            (fly_to_menu, |mut end_game: ResMut<NextState<EndGame>>| {
                // Defend against some nonsense
                end_game.set(EndGame { game_ended: false });
            }),
        )
        .add_systems(
            Update,
            (|key_input: Res<ButtonInput<KeyCode>>,
              mut main_state: ResMut<NextState<MainState>>| {
                if key_input.just_pressed(KeyCode::Escape) {
                    main_state.set(MainState::Game);
                }
            })
            .run_if(in_state(MenuState::Pause)),
        )
        .add_systems(
            Update,
            (
                ai::tick_ai,
                scatter_tick.run_if(ready_for_scatter),
                (orbit, game_ended).run_if(in_state(EndGame { game_ended: true })),
                esc_to_menu,
            )
                .run_if(in_state(MainState::Game)),
        )
        .add_systems(Update, run_splash)
        .add_systems(Update, cleanup_menus)
        .init_state::<MainState>()
        .init_state::<NeedNewBoard>()
        .init_state::<CurrentTurn>()
        .init_state::<GameOperation>()
        .add_sub_state::<MenuState>()
        .add_sub_state::<EndGame>()
        .register_type::<GameAssets>()
        .register_type::<CellColor>()
        .register_type::<CurrentTurn>()
        .register_type::<GameOperation>()
        .register_type::<DotCell>()
        .register_type::<Dot>()
        .register_type::<MainState>()
        .register_type::<Config>()
        .register_type::<SmoothingSettings>()
        .register_type::<TargetTransform>()
        .run();
}

pub const GRAY: Color = Color::Srgba(bevy::color::palettes::tailwind::GRAY_400);

#[derive(Component)]
pub struct Splash;

fn switch_menus(
    cur_menu: Option<Res<State<MenuState>>>,
    mut prev_menu: Local<Option<MenuState>>,
    menu_elements: Query<(
        &MenuElement,
        &mut TargetTransform,
        &mut Transform,
        &mut Visibility,
    )>,
) {
    for (el, mut target, mut transform, mut visibility) in menu_elements {
        if *prev_menu == Some(el.for_menu) {
            // Fly out
            let mut new_transform = el.target;
            new_transform.translation += vec3(0.0, 0.0, -20.0);
            target.0 = new_transform;
        }
        if let Some(ref cur_menu) = cur_menu
            && **cur_menu == el.for_menu
        {
            // Fly in
            let mut new_transform = el.target;
            new_transform.translation += el.side * vec3(20.0, 0.0, 0.0);
            *transform = new_transform;
            target.0 = el.target;
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

fn run_splash(
    splash: Query<&MeshMaterial3d<StandardMaterial>, With<Splash>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cur_state: Res<State<MainState>>,
    mut next_state: ResMut<NextState<MainState>>,
    time: Res<Time>,
) {
    let Ok(splash_material) = splash.single() else {
        return;
    };
    let Some(splash_material) = materials.get_mut(splash_material.id()) else {
        return;
    };
    if **cur_state != MainState::Splash {
        let Color::LinearRgba(base_color) = splash_material.base_color else {
            return;
        };

        let mut tmp = base_color.to_vec4();
        tmp.smooth_nudge(&Vec4::ZERO, 3.0, time.delta_secs());
        splash_material.base_color = Color::LinearRgba(LinearRgba::from_vec4(tmp));

        let mut tmp = splash_material.emissive.to_vec4();
        tmp.smooth_nudge(&Vec4::ZERO, 3.0, time.delta_secs());
        splash_material.emissive = LinearRgba::from_vec4(tmp);
        return;
    }
    match time.elapsed_secs() {
        0.0..0.5 => {} // Wait for a bit to spin up materials
        x @ 0.5..1.5 => {
            splash_material.base_color = Color::linear_rgba(1.0, 1.0, 1.0, x - 0.5);
            splash_material.emissive = LinearRgba::WHITE * ((x - 0.5) * 10.0).exp_m1() * 0.007;
        }
        1.5..4.0 => {
            splash_material.base_color = Color::WHITE;
            let mut tmp = splash_material.emissive.to_vec4();
            tmp.smooth_nudge(&Vec4::ONE, 3.0, time.delta_secs());
            splash_material.emissive = LinearRgba::from_vec4(tmp);
        }
        4.0.. => next_state.set(MainState::Menu),
        _ => {} // IDK what happened here
    }
}

fn esc_to_menu(
    key_input: Res<ButtonInput<KeyCode>>,
    mut next_need_new_board: ResMut<NextState<NeedNewBoard>>,
    mut main_state: ResMut<NextState<MainState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
) {
    if key_input.just_pressed(KeyCode::Escape) {
        next_need_new_board.set(NeedNewBoard(false)); // So we don't accidentally reset the board coming back from pause
        main_state.set(MainState::Menu);
        menu_state.set(MenuState::Pause);
    }
}

fn fly_in_game(
    mut commands: Commands,
    mut camera_pos: Query<&mut TargetTransform, With<Camera3d>>,
    config: Res<Config>,
    mut grid_tray: Query<
        (Entity, &mut Transform, &mut TargetTransform),
        (With<GridTray>, Without<Camera3d>),
    >,
    need_new_board: Res<State<NeedNewBoard>>,
    mut next_need_new_board: ResMut<NextState<NeedNewBoard>>,
    mut grid: ResMut<CellGrid>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,
    mut next_turn: ResMut<NextState<CurrentTurn>>,
    named_entities: Query<(Entity, &Name)>,
    mut game_operation: ResMut<NextState<GameOperation>>,
) {
    let (width, height) = config.grid_size;
    let max_dim = (width * 2 / 3).max(height);
    let true_max_dim = width.max(height);
    if let Ok(mut camera_pos) = camera_pos.single_mut() {
        **camera_pos = Transform::from_xyz(0.0, max_dim as f32 * 2.0, max_dim as f32)
            .looking_at(Vec3::ZERO, Vec3::Y);
    }
    for (table, name) in named_entities {
        if name.as_str() == "Table" {
            let scale = (true_max_dim + 2) as f32 / 8.0;
            commands.entity(table).insert((
                SmoothingSettings {
                    scale_decay_rate: 2.0,
                    ..default()
                },
                TargetTransform(Transform::from_scale(vec3(scale, 1.0, scale))),
            ));
        }
    }
    let Ok((grid_tray, mut transform, mut target)) = grid_tray.single_mut() else {
        return;
    };
    **target = Transform::default();

    if need_new_board.0 {
        let (width, height) = config.grid_size;
        grid.new_inplace(width, height);
        commands
            .entity(grid_tray)
            .despawn_related::<Children>()
            .with_children(|commands| {
                for y in 0..height {
                    for x in 0..width {
                        let x_border = x == 0 || x == grid.width() - 1;
                        let y_border = y == 0 || y == grid.height() - 1;
                        let capacity = if x_border && y_border {
                            2
                        } else if x_border || y_border {
                            3
                        } else {
                            4
                        };
                        grid[y][x] = spawn_cell(
                            commands,
                            &mut materials,
                            &game_assets,
                            x as f32 - width as f32 / 2.0 + 0.5,
                            y as f32 - height as f32 / 2.0 + 0.5,
                            capacity,
                        );
                    }
                }
            });
        game_operation.set(GameOperation::Animating);
        transform.translation = vec3(0.0, 30.0, 0.0);
        next_need_new_board.set(NeedNewBoard(false));
        next_turn.set(CurrentTurn(0));
    }
}

fn fly_out_game(mut grid_tray: Query<&mut TargetTransform, With<GridTray>>) {
    if let Ok(mut target) = grid_tray.single_mut() {
        target.translation = Vec3::new(0.0, 0.0, 30.0);
    }
}

fn fly_to_menu(
    mut commands: Commands,
    mut camera_pos: Query<&mut TargetTransform, With<Camera3d>>,
    mut orbiter: Query<&mut TargetTransform, (With<Orbiter>, Without<Camera3d>)>,
    named_entities: Query<(Entity, &Name)>,
) {
    for (table, name) in named_entities {
        if name.as_str() == "Table" {
            commands.entity(table).insert((
                SmoothingSettings {
                    scale_decay_rate: 2.0,
                    ..default()
                },
                TargetTransform(Transform::from_scale(vec3(1.0, 1.0, 1.0))),
            ));
        }
    }
    if let Ok(mut camera_pos) = camera_pos.single_mut() {
        **camera_pos = Transform::from_xyz(0.0, 12.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z);
    }
    if let Ok(mut orbiter) = orbiter.single_mut() {
        orbiter.rotation = Quat::from_axis_angle(Vec3::Y, 0.0);
    }
}

fn spawn_dot(x: f32, z: f32, game_assets: &GameAssets) -> impl Bundle {
    (
        Mesh3d(game_assets.dot_mesh.clone_weak()),
        MeshMaterial3d(game_assets.dot_color.clone_weak()),
        Transform::from_xyz(x, 1.0, z).with_scale(Vec3::ZERO),
        SmoothingSettings {
            translation_decay_rate: 8.0,
            rotation_decay_rate: 0.0, // unused
            scale_decay_rate: 20.0,
        },
        TargetTransform(Transform::from_xyz(x, 0.0, z)),
        Pickable::IGNORE,
    )
}

fn spawn_cell(
    commands: &mut ChildSpawnerCommands,
    materials: &mut Assets<StandardMaterial>,
    game_assets: &GameAssets,
    x: f32,
    z: f32,
    capacity: usize,
) -> Entity {
    commands
        .spawn((
            Mesh3d(game_assets.tile_mesh.clone_weak()),
            MeshMaterial3d(materials.add(GRAY)),
            Transform::from_xyz(x, -0.15, z),
            TargetTransform(Transform::from_xyz(x, -0.15, z)),
            SmoothingSettings {
                translation_decay_rate: 5.0,
                rotation_decay_rate: 0.0,
                scale_decay_rate: 10.0,
            },
            Pickable::default(),
            related!(DotCell[
                (spawn_dot(x, z, game_assets), ChildOf(commands.target_entity())),
            ]),
            DotCellMeta { capacity },
        ))
        .observe(
            |trigger: Trigger<Pointer<Over>>, mut targets: Query<&mut TargetTransform>| {
                let mut target = targets.get_mut(trigger.target).unwrap();
                target.scale = Vec3::splat(1.05);
            },
        )
        .observe(
            |trigger: Trigger<Pointer<Out>>, mut targets: Query<&mut TargetTransform>| {
                let mut target = targets.get_mut(trigger.target).unwrap();
                target.scale = Vec3::splat(1.0);
            },
        )
        .observe(
            move |trigger: Trigger<Pointer<Click>>,
                  mut commands: Commands,
                  mut colors: Query<&mut CellColor>,
                  game_assets: Res<GameAssets>,
                  state: Option<Res<State<GameOperation>>>,
                  next_state: Option<ResMut<NextState<GameOperation>>>,
                  current_turn: Option<Res<State<CurrentTurn>>>,
                  grid_tray: Query<Entity, With<GridTray>>| {
                if let (Some(state), Some(mut next_state), Some(current_turn)) =
                    (state, next_state, current_turn)
                    && *state == GameOperation::Human
                {
                    let mut color = colors.get_mut(trigger.target).unwrap();
                    if color.player == 0 || color.player == current_turn.0 {
                        color.player = current_turn.0;
                        commands.entity(trigger.target).with_related::<Dot>((
                            spawn_dot(x, z, &game_assets),
                            ChildOf(grid_tray.single().unwrap()),
                        ));
                        next_state.set(GameOperation::Animating);
                    }
                }
            },
        )
        .id()
}

fn add_hover_observers(entity_commands: &mut EntityCommands) {
    let id = entity_commands.id();
    entity_commands
        .observe(
            move |_: Trigger<Pointer<Over>>, mut targets: Query<&mut TargetTransform>| {
                targets.get_mut(id).unwrap().scale = Vec3::splat(1.05);
            },
        )
        .observe(
            move |_: Trigger<Pointer<Out>>, mut targets: Query<&mut TargetTransform>| {
                targets.get_mut(id).unwrap().scale = Vec3::splat(1.0);
            },
        );
}

fn setup_scene(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Mesh3d(game_assets.splash_mesh.clone_weak()),
        MeshMaterial3d(game_assets.splash_material.clone_weak()),
        Transform::from_xyz(0.0, 12.0, 8.0).looking_to(Dir3::NEG_Z, Dir3::Y),
        Splash,
    ));

    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/hopdot.glb"))),
        MenuElement {
            for_menu: MenuState::Main,
            target: Transform::from_xyz(-3.5, -0.2, -3.0),
            side: -1.0,
        },
    ));

    add_hover_observers(
        commands
            .spawn((
                SceneRoot(
                    asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/start-game.glb")),
                ),
                MenuElement {
                    for_menu: MenuState::Main,
                    target: Transform::from_xyz(-3.5, -0.2, -1.5),
                    side: -1.0,
                },
            ))
            .observe(
                |_: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<MainState>>| {
                    next_state.set(MainState::Game);
                },
            ),
    );

    add_hover_observers(
        commands
            .spawn((
                SceneRoot(
                    asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/credits.glb")),
                ),
                MenuElement {
                    for_menu: MenuState::Main,
                    target: Transform::from_xyz(2.05, -0.2, 3.5),
                    side: 1.0,
                },
            ))
            .observe(
                |_: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<MainState>>| {
                    next_state.set(MainState::Game);
                },
            ),
    );

    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/gamepaused.glb"))),
        MenuElement {
            for_menu: MenuState::Pause,
            target: Transform::from_xyz(-3.5, -0.2, -3.0),
            side: -1.0,
        },
    ));

    commands.spawn((
        GridTray,
        SmoothingSettings {
            translation_decay_rate: 2.0,
            ..default()
        },
        TargetTransform(Transform::from_xyz(0.0, 30.0, 0.0)),
        Transform::from_xyz(0.0, 30.0, 0.0),
    ));

    commands.spawn((
        SceneRoot(game_assets.table_scene.clone_weak()),
        Transform::from_xyz(0.0, -0.3, 0.0),
    ));

    commands.spawn((
        PointLight {
            color: Color::WHITE,
            shadows_enabled: true,
            soft_shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 4.0, 0.0),
    ));

    commands.spawn((
        PointLight {
            color: Color::WHITE,
            shadows_enabled: true,
            soft_shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, -4.0, 0.0),
    ));

    commands
        .spawn((
            Orbiter,
            Transform::default(),
            TargetTransform(Transform::from_rotation(Quat::from_axis_angle(
                Vec3::Y,
                0.0,
            ))),
            SmoothingSettings {
                rotation_decay_rate: 3.0,
                ..default()
            },
            Visibility::Visible,
        ))
        .with_children(|commands| {
            commands.spawn((
                Camera3d::default(),
                Camera {
                    hdr: true,
                    ..default()
                },
                Transform::from_xyz(0.0, 12.0, 16.0).looking_to(Dir3::NEG_Z, Dir3::Y),
                Msaa::Off,
                #[cfg(not(target_family = "wasm"))] TemporalAntiAliasing::default(),
                #[cfg(not(target_family = "wasm"))] ShadowFilteringMethod::Temporal,
                TargetTransform(
                    Transform::from_xyz(0.0, 12.0, 20.0).looking_to(Dir3::NEG_Z, Dir3::Y),
                ),
                SmoothingSettings {
                    translation_decay_rate: 1.0,
                    rotation_decay_rate: 1.0,
                    scale_decay_rate: 1.5,
                },
                Bloom::ANAMORPHIC,
            ));
        });
}

pub fn ready_for_scatter(
    mut timer: Local<Timer>,
    time: Res<Time>,
    state: Option<Res<State<GameOperation>>>,
) -> bool {
    timer.set_mode(TimerMode::Repeating);
    timer.set_duration(Duration::from_millis(500));

    if let Some(state) = state {
        if *state == GameOperation::Animating {
            timer.tick(time.delta());

            timer.just_finished() || state.is_changed()
        } else {
            timer.reset();

            false
        }
    } else {
        false
    }
}

pub fn scatter_tick(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameOperation>>,
    current_turn: Res<State<CurrentTurn>>,
    mut next_turn: ResMut<NextState<CurrentTurn>>,
    player_config: Res<Config>,
    grid: Res<CellGrid>,
    mut cells: Query<(
        &mut DotCell,
        &DotCellMeta,
        &mut CellColor,
        &MeshMaterial3d<StandardMaterial>,
        &mut Transform,
    )>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut end_game: ResMut<NextState<EndGame>>,
    need_new_board: Res<State<NeedNewBoard>>,
    mut next_need_new_board: ResMut<NextState<NeedNewBoard>>,
) {
    let mut scatter_temp = vec![vec![false; grid.width()]; grid.height()];
    let mut do_scatter = false;
    let mut colors = HashSet::new();
    for (y, row) in grid.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            let (cell, meta, color, _, _) = cells.get(cell).unwrap();
            colors.insert(color.player);
            if cell.dots.len() > meta.capacity {
                do_scatter = true;
                scatter_temp[y][x] = true;
            }
        }
    }
    let game_over = colors.len() == 1 && !colors.contains(&0);
    if game_over {
        end_game.set(EndGame { game_ended: true });
        next_need_new_board.set(NeedNewBoard(true));
    }
    if do_scatter {
        for (y, row) in scatter_temp.iter().enumerate() {
            for (x, &should_scatter) in row.iter().enumerate() {
                if should_scatter {
                    let (_, _, new_color, material, mut transform) =
                        cells.get_mut(grid[y][x]).unwrap();
                    let new_color = *new_color;
                    let material = materials.get_mut(material.id()).unwrap();
                    material.emissive = LinearRgba::rgb(30.0, 30.0, 30.0);
                    transform.translation.y = -0.1;
                    let elapsed = time.elapsed_secs_f64();
                    if x > 0 {
                        let removed = cells.get_mut(grid[y][x]).unwrap().0.dots.remove(0);
                        let (mut cell, _, mut color, _, _) = cells.get_mut(grid[y][x - 1]).unwrap();
                        cell.dots.push(removed);
                        commands.entity(removed).insert(Bouncing(elapsed));
                        *color = new_color;
                    }
                    if y > 0 {
                        let removed = cells.get_mut(grid[y][x]).unwrap().0.dots.remove(0);
                        let (mut cell, _, mut color, _, _) = cells.get_mut(grid[y - 1][x]).unwrap();
                        cell.dots.push(removed);
                        commands.entity(removed).insert(Bouncing(elapsed));
                        *color = new_color;
                    }
                    if x < grid.width() - 1 {
                        let removed = cells.get_mut(grid[y][x]).unwrap().0.dots.remove(0);
                        let (mut cell, _, mut color, _, _) = cells.get_mut(grid[y][x + 1]).unwrap();
                        cell.dots.push(removed);
                        commands.entity(removed).insert(Bouncing(elapsed));
                        *color = new_color;
                    }
                    if y < grid.height() - 1 {
                        let removed = cells.get_mut(grid[y][x]).unwrap().0.dots.remove(0);
                        let (mut cell, _, mut color, _, _) = cells.get_mut(grid[y + 1][x]).unwrap();
                        cell.dots.push(removed);
                        commands.entity(removed).insert(Bouncing(elapsed));
                        *color = new_color;
                    }
                }
            }
        }
    } else if !game_over && !need_new_board.0 {
        // Check so we keep orbiting if the game has ended and don't do stupid stuff if we need a new board
        let next_turn_idx = current_turn.0 % player_config.players.len();
        next_state.set(match player_config.players[next_turn_idx] {
            PlayerConfigEntry::Bot { .. } => GameOperation::Bot,
            PlayerConfigEntry::Human { .. } => GameOperation::Human,
        });
        next_turn.set(CurrentTurn(next_turn_idx + 1)); // current_turn is 1-indexed
    }
}

pub fn orbit(mut orbiter: Query<&mut TargetTransform, With<Orbiter>>, time: Res<Time>) {
    if let Ok(mut orbiter) = orbiter.single_mut() {
        orbiter.rotate_y(time.delta_secs() * 0.1);
    }
}

pub fn game_ended(
    mut camera_pos: Query<&mut TargetTransform, With<Camera3d>>,
    config: Res<Config>,
) {
    if let Ok(mut camera_pos) = camera_pos.single_mut() {
        let (width, height) = config.grid_size;
        let max_dim = (width * 2 / 3).max(height);
        *camera_pos = TargetTransform(
            Transform::from_xyz(0.0, max_dim as f32, 2.0 * max_dim as f32)
                .looking_at(Vec3::ZERO, Vec3::Y),
        );
    }
}
