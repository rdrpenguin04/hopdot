use std::time::Duration;

use bevy::{
    core_pipeline::experimental::taa::{TemporalAntiAliasPlugin, TemporalAntiAliasing},
    pbr::ShadowFilteringMethod,
    prelude::*,
};

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

#[derive(Component, Default, Reflect)]
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

#[derive(Reflect)]
pub enum PlayerConfigEntry {
    Human { color: Color, name: String },
    Bot { color: Color, level: usize },
}

#[derive(Resource, Reflect)]
pub struct PlayerConfig {
    pub players: Vec<PlayerConfigEntry>,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(TemporalAntiAliasPlugin)
        .add_plugins(MeshPickingPlugin);

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
        .insert_resource(AmbientLight {
            brightness: 1000.0,
            ..default()
        })
        .insert_resource(ClearColor(Color::srgb_u8(33, 34, 37)))
        .insert_resource(PlayerConfig {
            players: vec![
                PlayerConfigEntry::Human {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    name: "Player 1".into(),
                },
                PlayerConfigEntry::Human {
                    color: Color::srgb(0.0, 0.0, 1.0),
                    name: "Player 2".into(),
                },
            ],
        })
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                arrange_dots,
                animate_cell_colors,
                smooth_transform,
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
        .register_type::<PlayerConfig>()
        .register_type::<SmoothingSettings>()
        .register_type::<TargetTransform>()
        .run();
}

const GRAY: Color = Color::Srgba(bevy::color::palettes::tailwind::GRAY_400);

fn spawn_cell(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    game_assets: &GameAssets,
    x: f32,
    z: f32,
) -> Entity {
    commands
        .spawn((
            Mesh3d(game_assets.tile_mesh.clone_weak()),
            MeshMaterial3d(materials.add(Color::from(GRAY))),
            Transform::from_xyz(x, -0.15, z),
            TargetTransform(Transform::default()), // It's ok, we're not changing translation
            SmoothingSettings {
                translation_decay_rate: 0.0,
                rotation_decay_rate: 0.0,
                scale_decay_rate: 10.0,
            },
            Pickable::default(),
            related!(DotCell[
                (
                    Mesh3d(game_assets.dot_mesh.clone_weak()),
                    MeshMaterial3d(game_assets.dot_color.clone_weak()),
                    Transform::from_xyz(x, 0.0, z),
                    SmoothingSettings {
                        translation_decay_rate: 10.0,
                        rotation_decay_rate: 0.0, // unused
                        scale_decay_rate: 20.0,
                    },
                    TargetTransform(Transform::from_xyz(x, 0.0, z)),
                    Pickable::IGNORE,
                ),
            ]),
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
                        commands.entity(trigger.target).with_related::<Dot>((
                            Mesh3d(game_assets.dot_mesh.clone_weak()),
                            MeshMaterial3d(game_assets.dot_color.clone_weak()),
                            Transform::from_xyz(x, 1.0, z).with_scale(Vec3::ZERO),
                            SmoothingSettings {
                                translation_decay_rate: 10.0,
                                rotation_decay_rate: 0.0, // unused
                                scale_decay_rate: 20.0,
                            },
                            TargetTransform(Transform::from_xyz(x, 0.0, z)),
                            Pickable::IGNORE,
                        ));
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
    game_assets: Res<GameAssets>,
) {
    spawn_cell(&mut commands, &mut materials, &game_assets, -1.0, -1.0);
    spawn_cell(&mut commands, &mut materials, &game_assets, 0.0, -1.0);
    spawn_cell(&mut commands, &mut materials, &game_assets, 1.0, -1.0);
    spawn_cell(&mut commands, &mut materials, &game_assets, -1.0, 0.0);
    spawn_cell(&mut commands, &mut materials, &game_assets, 0.0, 0.0);
    spawn_cell(&mut commands, &mut materials, &game_assets, 1.0, 0.0);
    spawn_cell(&mut commands, &mut materials, &game_assets, -1.0, 1.0);
    spawn_cell(&mut commands, &mut materials, &game_assets, 0.0, 1.0);
    spawn_cell(&mut commands, &mut materials, &game_assets, 1.0, 1.0);

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
        Transform::from_xyz(0.0, 6.0, -3.0).looking_at(Vec3::ZERO, Vec3::Y),
        Msaa::Off,
        TemporalAntiAliasing::default(),
        ShadowFilteringMethod::Temporal,
        TargetTransform(Transform::from_xyz(0.0, 6.0, -3.0).looking_at(Vec3::ZERO, Vec3::Y)),
        SmoothingSettings {
            translation_decay_rate: 2.0,
            rotation_decay_rate: 1.8,
            scale_decay_rate: 1.5,
        },
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

pub fn scatter_tick(
    mut next_state: ResMut<NextState<GameOperation>>,
    current_turn: Res<State<CurrentTurn>>,
    mut next_turn: ResMut<NextState<CurrentTurn>>,
    player_config: Res<PlayerConfig>,
) {
    // TODO: actually handle animation/scatter
    let next_turn_idx = current_turn.0 % player_config.players.len();
    next_state.set(match player_config.players[next_turn_idx] {
        PlayerConfigEntry::Bot { .. } => GameOperation::Bot,
        PlayerConfigEntry::Human { .. } => GameOperation::Human,
    });
    next_turn.set(CurrentTurn(next_turn_idx + 1)); // current_turn is 1-indexed
}

pub fn animate_cell_colors(
    cells: Query<(&MeshMaterial3d<StandardMaterial>, &CellColor)>,
    player_config: Res<PlayerConfig>,
    time: Res<Time>,
    mut colors: ResMut<Assets<StandardMaterial>>,
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
            if let Color::Srgba(srgba) = &mut colors.get_mut(material.id()).unwrap().base_color {
                let mut temp = vec4(srgba.red, srgba.green, srgba.blue, srgba.alpha);
                let target_color_vec = vec4(
                    target_color.red,
                    target_color.green,
                    target_color.blue,
                    target_color.alpha,
                );
                temp.smooth_nudge(&target_color_vec, 3.0, time.delta_secs());
                srgba.red = temp.x;
                srgba.green = temp.y;
                srgba.blue = temp.z;
                srgba.alpha = temp.w;
            }
        }
    }
}
