#![feature(iter_collect_into)]
#![feature(iter_intersperse)]

pub mod ai;
pub mod anim;
pub mod menu;
pub mod net;
pub mod projection;
pub mod ui_menu;

use std::{
    iter,
    ops::{Index, IndexMut},
    time::Duration,
};

use bevy::audio::Volume;
#[allow(unused_imports)] // WASM
use bevy::{anti_alias::taa::TemporalAntiAliasing, light::ShadowFilteringMethod, platform::collections::HashSet, post_process::bloom::Bloom, prelude::*};
use bevy_defer::{AsyncCommandsExtension, AsyncPlugin, AsyncWorld, fetch};
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;
use bevy_skein::SkeinPlugin;
#[cfg(not(target_family = "wasm"))]
use bevy_tokio_tasks::TokioTasksPlugin;

use crate::{
    ai::Ais,
    anim::{Bouncing, SmoothingSettings, TargetMaterialColor, TargetTransform, TargetUiOpacity},
    menu::MenuState,
    net::{NetManagerMessage, NetServerboundSender},
    projection::PerspectiveMinAspect,
    ui_menu::{GameEndText, GameEndUiTree, GameHudUiTree, support::fade_out_ui},
};

#[derive(Resource, Reflect)]
pub struct GameAssets {
    table_scene: Handle<Scene>,
    bump_sfx: Handle<AudioSource>,
    bold_font: Handle<Font>,
    mono_font: Handle<Font>,
    dot_mesh: Handle<Mesh>,
    tile_mesh: Handle<Mesh>,
    splash_mesh: Handle<Mesh>,
    dot_color: Handle<StandardMaterial>,
    splash_material: Handle<StandardMaterial>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let table_scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/table.glb"));

        let bump_sfx = asset_server.load("sound/bump.flac");

        let bold_font = asset_server.load("fonts/FiraSans-Bold.ttf");
        let mono_font = asset_server.load("fonts/FiraMono-Medium.ttf");

        let splash_image = asset_server.load("tex/splash.png");

        let mut meshes = world.resource_mut::<Assets<_>>();
        let dot_mesh = meshes.add(Sphere::new(0.1).mesh().ico(2).unwrap());
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
            bump_sfx,
            bold_font,
            mono_font,
            dot_mesh,
            tile_mesh,
            splash_mesh,
            dot_color,
            splash_material,
        }
    }
}

#[derive(Clone, Component, Copy, Default, Reflect)]
#[require(TargetMaterialColor)]
#[reflect(Component)]
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
    DimForUi,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect, States)]
pub enum GameOperation {
    #[default]
    Animating,
    Human,
    Bot,
    OnlinePlayer,
    Connecting,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect, SubStates)]
#[source(GameOperation = GameOperation::Animating)]
pub struct EndGame {
    game_ended: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect, States)]
pub struct CurrentTurn(usize);

#[derive(Clone, Debug, Reflect)]
pub enum PlayerConfigEntry {
    Human {
        color: Color,
        name: String,
        _level: usize,
        online: bool,
    },
    Bot {
        color: Color,
        _name: String,
        level: usize,
        online: bool,
    },
    Disabled {
        _color: Color,
        _name: String,
        _level: usize,
        _online: bool,
    },
}

impl PlayerConfigEntry {
    pub fn is_human(&self) -> bool {
        matches!(self, Self::Human { .. })
    }
    pub fn is_bot(&self) -> bool {
        matches!(self, Self::Bot { .. })
    }
    pub fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled { .. })
    }

    pub fn to_human(&mut self) {
        *self = Self::Human {
            color: self.color(),
            name: self.name().to_owned(),
            _level: self.level(),
            online: self.online(),
        };
    }

    pub fn to_bot(&mut self) {
        *self = Self::Bot {
            color: self.color(),
            _name: self.name().to_owned(),
            level: self.level(),
            online: self.online(),
        };
    }

    pub fn to_disabled(&mut self) {
        *self = Self::Disabled {
            _color: self.color(),
            _name: self.name().to_owned(),
            _level: self.level(),
            _online: self.online(),
        };
    }

    pub fn to_online(&mut self) {
        match self {
            Self::Human { online, .. } | Self::Bot { online, .. } | Self::Disabled { _online: online, .. } => *online = true,
        }
    }

    pub fn as_human(mut self) -> Self {
        self.to_human();
        self
    }

    pub fn as_bot(mut self) -> Self {
        self.to_bot();
        self
    }

    pub fn as_disabled(mut self) -> Self {
        self.to_disabled();
        self
    }

    pub fn as_online(mut self) -> Self {
        self.to_online();
        self
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Human { color, .. } | Self::Bot { color, .. } | Self::Disabled { _color: color, .. } => *color,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Human { name, .. } | Self::Bot { _name: name, .. } | Self::Disabled { _name: name, .. } => name,
        }
    }

    pub fn level(&self) -> usize {
        match self {
            Self::Human { _level: level, .. } | Self::Bot { level, .. } | Self::Disabled { _level: level, .. } => *level,
        }
    }

    pub fn online(&self) -> bool {
        match self {
            Self::Human { online, .. } | Self::Bot { online, .. } | Self::Disabled { _online: online, .. } => *online,
        }
    }

    pub fn set_level(&mut self, new_level: usize) {
        match self {
            Self::Human { _level: level, .. } | Self::Bot { level, .. } | Self::Disabled { _level: level, .. } => *level = new_level,
        }
    }

    pub fn set_online(&mut self, new_online: bool) {
        match self {
            Self::Human { online, .. } | Self::Bot { online, .. } | Self::Disabled { _online: online, .. } => *online = new_online,
        }
    }

    pub fn default_for_player(player: usize) -> Self {
        match player {
            1 => PlayerConfigEntry::Human {
                color: Color::srgb(0.0, 1.0, 0.0),
                name: "Player 1".into(),
                _level: 0,
                online: false,
            },
            2 => PlayerConfigEntry::Bot {
                color: Color::srgb(0.0, 0.0, 1.0),
                _name: "Player 2".into(),
                level: 0,
                online: false,
            },
            3 => PlayerConfigEntry::Disabled {
                _color: Color::srgb(1.0, 0.0, 1.0),
                _name: "Player 3".into(),
                _level: 0,
                _online: false,
            },
            4 => PlayerConfigEntry::Disabled {
                _color: Color::srgb(0.0, 1.0, 1.0),
                _name: "Player 4".into(),
                _level: 0,
                _online: false,
            },
            _ => panic!("invalid player number"),
        }
    }
}

#[derive(Clone, Resource, Reflect)]
pub struct Config {
    pub players: Vec<PlayerConfigEntry>,
    pub grid_size: (usize, usize),
}

#[derive(Default, Resource, Reflect)]
pub struct VisualGrid {
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

impl VisualGrid {
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

impl Index<usize> for VisualGrid {
    type Output = [Entity];

    fn index(&self, index: usize) -> &[Entity] {
        &self.grid[(index * self.width)..((index + 1) * self.width)]
    }
}

impl IndexMut<usize> for VisualGrid {
    fn index_mut(&mut self, index: usize) -> &mut [Entity] {
        &mut self.grid[(index * self.width)..((index + 1) * self.width)]
    }
}

impl<'a> IntoIterator for &'a VisualGrid {
    type Item = &'a [Entity];

    type IntoIter = core::slice::ChunksExact<'a, Entity>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Resource)]
pub struct FlashIntensity(f32);

const TABLE_BASE_COLOR: Color = Color::Srgba(Srgba::rgb(0.904, 0.943, 1.0));
const TABLE_DARK_COLOR: Color = Color::Srgba(Srgba::rgb(0.0, 0.005, 0.008));

#[derive(Component, Reflect)]
#[require(TargetMaterialColor = TargetMaterialColor(TABLE_BASE_COLOR))]
#[reflect(Component)]
pub struct TableMaterial;

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct GameCode(Option<String>);

#[derive(Component)]
pub struct GameCodeText;

#[bevy_main]
pub fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: format!("Hopdot v{}", env!("CARGO_PKG_VERSION")),
            name: Some("lcdev.hopdot".into()),
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(AsyncPlugin::default_settings())
    .add_plugins(MeshPickingPlugin)
    .add_plugins(EntropyPlugin::<WyRand>::default())
    .add_plugins(SkeinPlugin::default())
    .add_plugins(anim::plugin)
    .add_plugins(menu::plugin)
    .add_plugins(ui_menu::plugin)
    .add_plugins(net::plugin);

    #[cfg(not(target_family = "wasm"))]
    app.add_plugins(TokioTasksPlugin::default());

    #[cfg(feature = "bevy-inspector-egui")]
    {
        use bevy_inspector_egui::{
            bevy_egui::EguiPlugin,
            quick::{StateInspectorPlugin, WorldInspectorPlugin},
        };

        app.add_plugins((
            EguiPlugin::default(),
            StateInspectorPlugin::<MainState>::default(),
            StateInspectorPlugin::<CurrentTurn>::default(),
            StateInspectorPlugin::<NeedNewBoard>::default(),
            WorldInspectorPlugin::default(),
        ));
    }

    app.init_resource::<GameAssets>()
        .init_resource::<VisualGrid>()
        .init_resource::<Ais>()
        .insert_resource(GlobalAmbientLight {
            brightness: 1000.0,
            ..default()
        })
        .insert_resource(ClearColor(Color::srgb_u8(33, 34, 37)))
        .insert_resource(FlashIntensity(0.3))
        .insert_resource(Config {
            players: vec![PlayerConfigEntry::default_for_player(1), PlayerConfigEntry::default_for_player(2)],
            grid_size: (6, 6),
        })
        .init_resource::<GameCode>()
        .add_systems(Startup, setup_scene)
        .add_systems(OnEnter(MainState::Game), fly_in_game)
        .add_systems(OnExit(MainState::Game), fly_out_game)
        .add_systems(
            OnEnter(MainState::DimForUi),
            |lights: Query<&mut PointLight>, mut table_material: Query<&mut TargetMaterialColor, With<TableMaterial>>| {
                for mut light in lights {
                    light.intensity = 0.0;
                }
                if let Ok(mut x) = table_material.single_mut() {
                    x.0 = TABLE_DARK_COLOR;
                }
            },
        )
        .add_systems(
            OnExit(MainState::DimForUi),
            |lights: Query<&mut PointLight>, mut table_material: Query<&mut TargetMaterialColor, With<TableMaterial>>| {
                for mut light in lights {
                    light.intensity = 1_000_000.0;
                }
                if let Ok(mut x) = table_material.single_mut() {
                    x.0 = TABLE_BASE_COLOR;
                }
            },
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
            (
                ai::tick_ai,
                scatter_tick.run_if(ready_for_scatter),
                (orbit, game_ended).run_if(in_state(EndGame { game_ended: true })),
            )
                .run_if(in_state(MainState::Game)),
        )
        .add_systems(Update, (run_splash, esc_to_menu.after(game_ended)))
        .add_systems(
            OnEnter(MainState::Splash),
            |mut commands: Commands, mut ui_opacity: ResMut<TargetUiOpacity>, ui_trees: Query<Entity, (With<Node>, Without<ChildOf>)>| {
                ui_opacity.0 = 0.0;
                for ui_tree in &ui_trees {
                    commands.spawn_task(move || async move {
                        AsyncWorld.sleep(0.75).await;
                        fetch!(ui_tree, Visibility).get_mut(|x| *x = Visibility::Hidden)?;
                        Ok(())
                    });
                }
            },
        )
        .add_systems(Update, |game_code: Res<GameCode>, texts: Query<&mut Text, With<GameCodeText>>| {
            if let Some(code) = &game_code.0 {
                let code_formatted = code.chars().chain(iter::repeat('-')).take(4).intersperse(' ').collect::<String>();
                for mut text in texts {
                    text.0 = code_formatted.clone();
                }
            }
        })
        .init_state::<MainState>()
        .init_state::<NeedNewBoard>()
        .init_state::<CurrentTurn>()
        .init_state::<GameOperation>()
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
        .register_type::<TableMaterial>()
        .run();
}

pub const GRAY: Color = Color::Srgba(bevy::color::palettes::tailwind::GRAY_400);

#[derive(Component)]
pub struct Splash;

#[derive(Component)]
pub struct BumpPlayer;

fn run_splash(
    mut commands: Commands,
    splash: Query<(Entity, &MeshMaterial3d<StandardMaterial>), With<Splash>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cur_state: Res<State<MainState>>,
    mut next_state: ResMut<NextState<MainState>>,
    time: Res<Time>,
) {
    let Ok((splash, splash_material)) = splash.single() else {
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
        4.0.. => {
            next_state.set(MainState::Menu);
            commands.spawn_task(move || async move {
                AsyncWorld.sleep(0.75).await;
                fetch!(splash, Visibility).get_mut(|x| *x = Visibility::Hidden)?;
                Ok(())
            });
        }
        _ => {} // IDK what happened here
    }
}

#[allow(clippy::collapsible_if)] // TODO
fn esc_to_menu(
    key_input: Res<ButtonInput<KeyCode>>,
    mut main_state: ResMut<NextState<MainState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    cur_state: Res<State<MainState>>,
    end_game: Option<Res<State<EndGame>>>,
    mut ui_opacity: ResMut<TargetUiOpacity>,
    mut commands: Commands,
    ui_tree: Query<Entity, With<GameEndUiTree>>,
) {
    if key_input.just_pressed(KeyCode::Escape) {
        if *cur_state == MainState::Game {
            if let Some(end_game) = end_game
                && end_game.game_ended
            {
                main_state.set(MainState::Menu);
                fade_out_ui(&mut commands, &mut ui_opacity, &ui_tree);
            } else {
                main_state.set(MainState::Menu);
                menu_state.set(MenuState::Pause);
            }
        }
    }
}

fn fly_in_game(
    mut commands: Commands,
    mut camera_pos: Query<&mut TargetTransform, With<Camera3d>>,
    config: Res<Config>,
    mut grid_tray: Query<(Entity, &mut Transform, &mut TargetTransform), (With<GridTray>, Without<Camera3d>)>,
    need_new_board: Res<State<NeedNewBoard>>,
    mut next_need_new_board: ResMut<NextState<NeedNewBoard>>,
    mut grid: ResMut<VisualGrid>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,
    mut next_turn: ResMut<NextState<CurrentTurn>>,
    named_entities: Query<(Entity, &Name)>,
    mut game_operation: ResMut<NextState<GameOperation>>,
    game_hud: Query<Entity, With<GameHudUiTree>>,
) {
    let (width, height) = config.grid_size;
    let max_dim = (width * 2 / 3).max(height);
    let true_max_dim = width.max(height);
    if let Ok(mut camera_pos) = camera_pos.single_mut() {
        **camera_pos = Transform::from_xyz(0.0, max_dim as f32 * 2.0, max_dim as f32).looking_at(Vec3::ZERO, Vec3::Y);
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
        commands.entity(grid_tray).despawn_related::<Children>().with_children(|commands| {
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
                        (x, y),
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

    let game_hud = game_hud.single().unwrap();

    commands.spawn_task(move || async move {
        AsyncWorld.sleep(0.75).await;
        fetch!(game_hud, Visibility).get_mut(|x| *x = Visibility::Inherited)?;
        fetch!(TargetUiOpacity).get_mut(|ui_opacity| ui_opacity.0 = 1.0)?;
        Ok(())
    });
}

fn fly_out_game(
    mut commands: Commands,
    mut grid_tray: Query<&mut TargetTransform, With<GridTray>>,
    game_hud: Query<Entity, With<GameHudUiTree>>,
    mut ui_opacity: ResMut<TargetUiOpacity>,
) {
    if let Ok(mut target) = grid_tray.single_mut() {
        target.translation = Vec3::new(0.0, 0.0, 30.0);
    }

    ui_opacity.0 = 0.0;
    let game_hud = game_hud.single().unwrap();
    commands.spawn_task(move || async move {
        AsyncWorld.sleep(0.75).await;
        fetch!(game_hud, Visibility).get_mut(|x| *x = Visibility::Hidden)?;
        Ok(())
    });
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
        Mesh3d(game_assets.dot_mesh.clone()),
        MeshMaterial3d(game_assets.dot_color.clone()),
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
    pos: (usize, usize),
    capacity: usize,
) -> Entity {
    commands
        .spawn((
            Mesh3d(game_assets.tile_mesh.clone()),
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
        .observe(|trigger: On<Pointer<Over>>, mut targets: Query<&mut TargetTransform>| {
            let mut target = targets.get_mut(trigger.original_event_target()).unwrap();
            target.scale = Vec3::splat(1.05);
        })
        .observe(|trigger: On<Pointer<Out>>, mut targets: Query<&mut TargetTransform>| {
            let mut target = targets.get_mut(trigger.original_event_target()).unwrap();
            target.scale = Vec3::splat(1.0);
        })
        .observe(
            move |trigger: On<Pointer<Click>>,
                  mut commands: Commands,
                  mut colors: Query<&mut CellColor>,
                  game_assets: Res<GameAssets>,
                  state: Option<Res<State<GameOperation>>>,
                  next_state: Option<ResMut<NextState<GameOperation>>>,
                  current_turn: Option<Res<State<CurrentTurn>>>,
                  grid_tray: Query<Entity, With<GridTray>>,
                  net_tx: Res<NetServerboundSender>| {
                if let (Some(state), Some(mut next_state), Some(current_turn)) = (state, next_state, current_turn)
                    && *state == GameOperation::Human
                {
                    let mut color = colors.get_mut(trigger.original_event_target()).unwrap();
                    if color.player == 0 || color.player == current_turn.0 {
                        color.player = current_turn.0;
                        commands
                            .entity(trigger.original_event_target())
                            .with_related::<Dot>((spawn_dot(x, z, &game_assets), ChildOf(grid_tray.single().unwrap())));
                        next_state.set(GameOperation::Animating);
                        net_tx
                            .force_send(NetManagerMessage::Move {
                                x: pos.0 as u8,
                                y: pos.1 as u8,
                            })
                            .unwrap();
                    }
                }
            },
        )
        .id()
}

fn add_hover_observers(entity_commands: &mut EntityCommands) {
    let id = entity_commands.id();
    entity_commands
        .observe(move |_: On<Pointer<Over>>, mut targets: Query<&mut TargetTransform>| {
            targets.get_mut(id).unwrap().scale = Vec3::splat(1.05);
        })
        .observe(move |_: On<Pointer<Out>>, mut targets: Query<&mut TargetTransform>| {
            targets.get_mut(id).unwrap().scale = Vec3::splat(1.0);
        });
}

fn setup_scene(mut commands: Commands, game_assets: Res<GameAssets>, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer(game_assets.bump_sfx.clone()),
        BumpPlayer,
        PlaybackSettings {
            volume: Volume::Decibels(-6.0),
            ..default()
        },
    ));

    commands.spawn((
        Mesh3d(game_assets.splash_mesh.clone()),
        MeshMaterial3d(game_assets.splash_material.clone()),
        Transform::from_xyz(0.0, 12.0, 8.0).looking_to(Dir3::NEG_Z, Dir3::Y),
        Splash,
    ));

    commands.spawn(SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/main-menu.glb"))));

    commands.spawn((
        GridTray,
        SmoothingSettings {
            translation_decay_rate: 2.0,
            ..default()
        },
        TargetTransform(Transform::from_xyz(0.0, 30.0, 0.0)),
        Transform::from_xyz(0.0, 30.0, 0.0),
    ));

    commands.spawn((SceneRoot(game_assets.table_scene.clone()), Transform::from_xyz(0.0, -0.3, 0.0)));

    commands.spawn((
        PointLight {
            color: Color::WHITE,
            shadows_enabled: true,
            soft_shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 4.0, 0.0),
    ));

    #[cfg(not(target_family = "wasm"))]
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
            TargetTransform(Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, 0.0))),
            SmoothingSettings {
                rotation_decay_rate: 3.0,
                ..default()
            },
            Visibility::Visible,
        ))
        .with_children(|commands| {
            commands.spawn((
                Camera3d::default(),
                Projection::custom(PerspectiveMinAspect::default()),
                Transform::from_xyz(0.0, 12.0, 16.0).looking_to(Dir3::NEG_Z, Dir3::Y),
                Msaa::Off,
                #[cfg(not(target_family = "wasm"))]
                TemporalAntiAliasing::default(),
                #[cfg(not(target_family = "wasm"))]
                ShadowFilteringMethod::Temporal,
                TargetTransform(Transform::from_xyz(0.0, 12.0, 20.0).looking_to(Dir3::NEG_Z, Dir3::Y)),
                SmoothingSettings {
                    translation_decay_rate: 1.0,
                    rotation_decay_rate: 1.0,
                    scale_decay_rate: 1.5,
                },
                Bloom::ANAMORPHIC,
            ));
        });
}

pub fn ready_for_scatter(mut timer: Local<Timer>, time: Res<Time>, state: Option<Res<State<GameOperation>>>) -> bool {
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
    grid: Res<VisualGrid>,
    mut cells: Query<(&mut DotCell, &DotCellMeta, &mut CellColor, &MeshMaterial3d<StandardMaterial>, &mut Transform)>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut end_game: ResMut<NextState<EndGame>>,
    need_new_board: Res<State<NeedNewBoard>>,
    mut next_need_new_board: ResMut<NextState<NeedNewBoard>>,
    bump_player: Query<Entity, With<BumpPlayer>>,
    intensity: Res<FlashIntensity>,
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
        commands.entity(bump_player.single().unwrap()).remove::<AudioSink>(); // Cheap restart
        for (y, row) in scatter_temp.iter().enumerate() {
            for (x, &should_scatter) in row.iter().enumerate() {
                if should_scatter {
                    let (_, _, new_color, material, mut transform) = cells.get_mut(grid[y][x]).unwrap();
                    let new_color = *new_color;
                    let material = materials.get_mut(material.id()).unwrap();
                    material.emissive = LinearRgba::WHITE * 100.0 * intensity.0;
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
        next_state.set(match &player_config.players[next_turn_idx] {
            x if x.online() => GameOperation::OnlinePlayer,
            PlayerConfigEntry::Bot { .. } => GameOperation::Bot,
            PlayerConfigEntry::Human { .. } => GameOperation::Human,
            _ => unreachable!(), // Disabled should never be in the final config
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
    mut ui_opacity: ResMut<TargetUiOpacity>,
    mut game_end_ui: Query<&mut Visibility, With<GameEndUiTree>>,
    mut game_end_text: Query<&mut Text, With<GameEndText>>,
    current_turn: Res<State<CurrentTurn>>,
    // ais: Res<Ais>,
) {
    if let Ok(mut camera_pos) = camera_pos.single_mut() {
        let (width, height) = config.grid_size;
        let max_dim = (width * 2 / 3).max(height);
        *camera_pos = TargetTransform(Transform::from_xyz(0.0, max_dim as f32, 2.0 * max_dim as f32).looking_at(Vec3::ZERO, Vec3::Y));
        ui_opacity.0 = 1.0;
        *game_end_ui.single_mut().unwrap() = Visibility::Visible;
        // let player = &config.players[current_turn.0 - 1];
        // game_end_text.single_mut().unwrap().0 = format!(
        //     "Player {}{} wins!",
        //     current_turn.0,
        //     if player.is_human() {
        //         if player.name() == &format!("Player {}", current_turn.0) {
        //             "".into()
        //         } else {
        //             format!(" ({})", player.name())
        //         }
        //     } else {
        //         format!(" ({})", ais[player.level()].name())
        //     }
        // );
        game_end_text.single_mut().unwrap().0 = format!("Player {} wins!", current_turn.0);
    }
}
