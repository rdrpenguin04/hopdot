pub mod ai;

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
    prelude::*,
};
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;

#[derive(Resource, Reflect)]
pub struct GameAssets {
    dot_mesh: Handle<Mesh>,
    tile_mesh: Handle<Mesh>,
    dot_color: Handle<StandardMaterial>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<_>>();
        let dot_mesh = meshes.add(Sphere::new(0.1));
        let tile_mesh = meshes.add(Cuboid::new(0.95, 0.1, 0.95));
        let mut materials = world.resource_mut::<Assets<_>>();
        let dot_color = materials.add(Color::srgb(1.0, 1.0, 1.0));
        Self {
            dot_mesh,
            tile_mesh,
            dot_color,
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

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct TargetTransform(pub Transform);

#[derive(Component, Reflect)]
pub struct SmoothingSettings {
    translation_decay_rate: f32,
    rotation_decay_rate: f32,
    scale_decay_rate: f32,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect, States)]
pub enum MainState {
    #[default]
    Game,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect, SubStates)]
#[source(MainState = MainState::Game)]
pub enum GameOperation {
    #[default]
    Animating,
    Human,
    Bot,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect, SubStates)]
#[source(MainState = MainState::Game)]
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

    pub fn iter(&self) -> core::slice::ChunksExact<Entity> {
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
        .add_plugins(TemporalAntiAliasPlugin)
        .add_plugins(MeshPickingPlugin)
        .add_plugins(EntropyPlugin::<WyRand>::default());
    // .add_plugins(RngPlugin);

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
            StateInspectorPlugin::<GameOperation>::default(),
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
                PlayerConfigEntry::Human {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    name: "Player 1".into(),
                },
                // PlayerConfigEntry::Human {
                //     color: Color::srgb(0.0, 0.0, 1.0),
                //     name: "Player 2".into(),
                // },
                PlayerConfigEntry::Bot {
                    color: Color::srgb(0.0, 0.0, 1.0),
                    level: 0,
                },
            ],
            grid_size: (6, 6),
        })
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                arrange_dots,
                animate_cell_colors,
                smooth_transform,
                run_bouncing,
                ai::tick_ai,
                scatter_tick.run_if(ready_for_scatter),
            ),
        )
        .init_state::<MainState>()
        .init_state::<GameOperation>()
        .init_state::<CurrentTurn>()
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

const GRAY: Color = Color::Srgba(bevy::color::palettes::tailwind::GRAY_400);

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
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    game_assets: &GameAssets,
    x: f32,
    z: f32,
    capacity: usize,
) -> Entity {
    commands
        .spawn((
            Mesh3d(game_assets.tile_mesh.clone_weak()),
            MeshMaterial3d(materials.add(Color::from(GRAY))),
            Transform::from_xyz(x, -0.15, z),
            TargetTransform(Transform::from_xyz(x, -0.15, z)),
            SmoothingSettings {
                translation_decay_rate: 5.0,
                rotation_decay_rate: 0.0,
                scale_decay_rate: 10.0,
            },
            Pickable::default(),
            related!(DotCell[
                spawn_dot(x, z, &game_assets),
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
                  state: Res<State<GameOperation>>,
                  mut next_state: ResMut<NextState<GameOperation>>,
                  current_turn: Res<State<CurrentTurn>>| {
                if *state == GameOperation::Human {
                    let mut color = colors.get_mut(trigger.target).unwrap();
                    if color.player == 0 || color.player == current_turn.0 {
                        color.player = current_turn.0;
                        commands
                            .entity(trigger.target)
                            .with_related::<Dot>(spawn_dot(x, z, &game_assets));
                        next_state.set(GameOperation::Animating);
                    }
                }
            },
        )
        .id()
}

fn setup_scene(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grid: ResMut<CellGrid>,
    game_assets: Res<GameAssets>,
    config: Res<Config>,
) {
    let (width, height) = config.grid_size;
    let max_dim = width.max(height);
    grid.new_inplace(width, height);
    for y in 0..height {
        for x in 0..width {
            let x_border = x == 0 || x == grid.width() - 1;
            let y_border = y == 0 || y == grid.width() - 1;
            let capacity = if x_border && y_border {
                2
            } else if x_border || y_border {
                3
            } else {
                4
            };
            grid[y][x] = spawn_cell(
                &mut commands,
                &mut materials,
                &game_assets,
                x as f32 - width as f32 / 2.0 + 0.5,
                y as f32 - height as f32 / 2.0 + 0.5,
                capacity,
            );
        }
    }

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
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Transform::from_xyz(0.0, max_dim as f32 * 2.0, -(max_dim as f32))
            .looking_at(Vec3::ZERO, Vec3::Y),
        Msaa::Off,
        TemporalAntiAliasing::default(),
        ShadowFilteringMethod::Temporal,
        TargetTransform(
            Transform::from_xyz(0.0, max_dim as f32 * 2.0, -(max_dim as f32))
                .looking_at(Vec3::ZERO, Vec3::Y),
        ),
        SmoothingSettings {
            translation_decay_rate: 2.0,
            rotation_decay_rate: 1.8,
            scale_decay_rate: 1.5,
        },
        Bloom::NATURAL,
    ));
}

fn smooth_transform(
    mut transforms: Query<(&TargetTransform, &SmoothingSettings, &mut Transform)>,
    time: Res<Time>,
) {
    for (target, settings, mut transform) in &mut transforms {
        transform.translation.smooth_nudge(
            &target.translation,
            settings.translation_decay_rate,
            time.delta_secs(),
        );
        transform.rotation.smooth_nudge(
            &target.rotation,
            settings.rotation_decay_rate,
            time.delta_secs(),
        );
        transform
            .scale
            .smooth_nudge(&target.scale, settings.scale_decay_rate, time.delta_secs());
    }
}

fn arrange_dots(
    cells: Query<(&DotCell, &Transform)>,
    mut dots: Query<&mut TargetTransform, With<Dot>>,
) {
    for (cell, transform) in &cells {
        let (cell_x, cell_z) = (transform.translation.x, transform.translation.z);
        let arrangement: &[(f32, f32)] = match cell.dots.len() {
            0 => continue, // Something has gone dreadfully wrong. Or we're early. Fail with grace.
            1 => &[(0.0, 0.0)],
            2 => &[(-0.25, 0.25), (0.25, -0.25)],
            3 => &[(-0.25, 0.25), (0.25, -0.25), (0.0, 0.0)],
            4 => &[(-0.25, 0.25), (0.25, -0.25), (-0.25, -0.25), (0.25, 0.25)],
            5 => &[
                (-0.25, 0.25),
                (0.25, -0.25),
                (-0.25, -0.25),
                (0.25, 0.25),
                (0.0, 0.0),
            ],
            6 => &[
                (-0.25, 0.25),
                (0.25, -0.25),
                (-0.25, -0.25),
                (0.25, 0.25),
                (-0.25, 0.0),
                (0.25, 0.0),
            ],
            7 => &[
                (-0.25, 0.25),
                (0.25, -0.25),
                (-0.25, -0.25),
                (0.25, 0.25),
                (-0.25, 0.0),
                (0.25, 0.0),
                (0.0, 0.0),
            ],
            8 => &[
                (-0.25, 0.25),
                (0.25, -0.25),
                (-0.25, -0.25),
                (0.25, 0.25),
                (-0.25, 0.0),
                (0.25, 0.0),
                (0.0, -0.25),
                (0.0, 0.25),
            ],
            _ => unreachable!("Something has gone cataclysmically wrong."),
        };
        for (dot, (x, z)) in cell.dots.iter().zip(arrangement) {
            dots.get_mut(*dot).unwrap().translation = Vec3::new(x + cell_x, 0.0, z + cell_z);
        }
    }
}

pub fn ready_for_scatter(
    mut timer: Local<Timer>,
    time: Res<Time>,
    state: Res<State<GameOperation>>,
) -> bool {
    timer.set_mode(TimerMode::Repeating);
    timer.set_duration(Duration::from_millis(500));

    if *state == GameOperation::Animating {
        timer.tick(time.delta());

        timer.just_finished() || state.is_changed()
    } else {
        timer.reset();

        false
    }
}

#[derive(Component, Reflect)]
pub struct Bouncing(pub f64);

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
) {
    let mut scatter_temp = vec![vec![false; grid.width()]; grid.height()];
    let mut do_scatter = false;
    for (y, row) in grid.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            let (cell, meta, _, _, _) = cells.get(cell).unwrap();
            if cell.dots.len() > meta.capacity {
                do_scatter = true;
                scatter_temp[y][x] = true;
            }
        }
    }
    if do_scatter {
        for (y, row) in scatter_temp.iter().enumerate() {
            for (x, &should_scatter) in row.iter().enumerate() {
                if should_scatter {
                    let (_, _, new_color, material, mut transform) =
                        cells.get_mut(grid[y][x]).unwrap();
                    let new_color = *new_color;
                    let material = materials.get_mut(material.id()).unwrap();
                    material.emissive = LinearRgba::rgb(100.0, 100.0, 100.0);
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
    } else {
        let next_turn_idx = current_turn.0 % player_config.players.len();
        next_state.set(match player_config.players[next_turn_idx] {
            PlayerConfigEntry::Bot { .. } => GameOperation::Bot,
            PlayerConfigEntry::Human { .. } => GameOperation::Human,
        });
        next_turn.set(CurrentTurn(next_turn_idx + 1)); // current_turn is 1-indexed
    }
}

pub fn animate_cell_colors(
    cells: Query<(&MeshMaterial3d<StandardMaterial>, &CellColor)>,
    player_config: Res<Config>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (material, color_idx) in &cells {
        let target_color = if color_idx.player == 0 {
            GRAY
        } else {
            match player_config.players[color_idx.player - 1] {
                PlayerConfigEntry::Human { color, .. } => color,
                PlayerConfigEntry::Bot { color, .. } => color,
            }
        };
        if let Color::Srgba(target_color) = target_color {
            let material = materials.get_mut(material.id()).unwrap();
            if let Color::Srgba(srgba) = &mut material.base_color {
                let mut temp = srgba.to_vec4();
                let target_color_vec = target_color.to_vec4();
                temp.smooth_nudge(&target_color_vec, 3.0, time.delta_secs());
                *srgba = Srgba::from_vec4(temp);
            }
            let mut temp = material.emissive.to_vec4();
            temp.smooth_nudge(&Vec4::ZERO, 10.0, time.delta_secs());
            material.emissive = LinearRgba::from_vec4(temp);
        }
    }
}

pub fn run_bouncing(
    mut commands: Commands,
    mut bouncing_objects: Query<(Entity, &Bouncing, &mut TargetTransform, &mut Transform)>,
    time: Res<Time>,
) {
    let elapsed = time.elapsed_secs_f64();
    for (entity, bouncing, mut target, mut transform) in &mut bouncing_objects {
        let t = (elapsed - bouncing.0) as f32;
        if t < 0.5 {
            // Directly pilot the ball's height
            let scalar = (vec2(target.0.translation.x, target.0.translation.z)
                - vec2(transform.translation.x, transform.translation.z))
            .length();
            target.0.translation.y = 16.0 * (t - 2.0 * t * t) * scalar;
            transform.translation.y = 16.0 * (t - 2.0 * t * t) * scalar;
        } else {
            target.0.translation.y = 0.0;
            commands.entity(entity).remove::<Bouncing>();
        }
    }
}
