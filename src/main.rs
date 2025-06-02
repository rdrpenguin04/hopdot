use bevy::{color::palettes, pbr::ShadowFilteringMethod, prelude::*};

#[derive(Resource)]
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

#[derive(Component)]
#[relationship_target(relationship = DotOf)]
pub struct DotCell(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = DotCell)]
pub struct DotOf(Entity);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_scene)
        .init_resource::<GameAssets>()
        .insert_resource(ClearColor(Color::srgb_u8(35, 34, 33)))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,
) {
    // Four
    commands.spawn((
        Mesh3d(game_assets.tile_mesh.clone_weak()),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 1.0, 0.0))),
        Transform::from_xyz(0.0, -0.15, 0.0),
        related!(DotCell[
            (
                Mesh3d(game_assets.dot_mesh.clone_weak()),
                MeshMaterial3d(game_assets.dot_color.clone_weak()),
                Transform::from_xyz(-0.25, 0.0, -0.25),
            ),
            (
                Mesh3d(game_assets.dot_mesh.clone_weak()),
                MeshMaterial3d(game_assets.dot_color.clone_weak()),
                Transform::from_xyz(0.25, 0.0, -0.25),
            ),
            (
                Mesh3d(game_assets.dot_mesh.clone_weak()),
                MeshMaterial3d(game_assets.dot_color.clone_weak()),
                Transform::from_xyz(-0.25, 0.0, 0.25),
            ),
            (
                Mesh3d(game_assets.dot_mesh.clone_weak()),
                MeshMaterial3d(game_assets.dot_color.clone_weak()),
                Transform::from_xyz(0.25, 0.0, 0.25),
            ),
        ]),
    ));

    // Three
    commands.spawn((
        Mesh3d(game_assets.tile_mesh.clone_weak()),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
        Transform::from_xyz(1.0, -0.15, 0.0),
        related!(DotCell[
            (
                Mesh3d(game_assets.dot_mesh.clone_weak()),
                MeshMaterial3d(game_assets.dot_color.clone_weak()),
                Transform::from_xyz(1.25, 0.0, -0.25),
            ),
            (
                Mesh3d(game_assets.dot_mesh.clone_weak()),
                MeshMaterial3d(game_assets.dot_color.clone_weak()),
                Transform::from_xyz(1.0, 0.0, 0.0),
            ),
            (
                Mesh3d(game_assets.dot_mesh.clone_weak()),
                MeshMaterial3d(game_assets.dot_color.clone_weak()),
                Transform::from_xyz(0.75, 0.0, 0.25),
            ),
        ]),
    ));

    // One
    commands.spawn((
        Mesh3d(game_assets.dot_mesh.clone_weak()),
        MeshMaterial3d(game_assets.dot_color.clone_weak()),
        Transform::from_xyz(-1.0, 0.0, 0.0),
    ));
    commands.spawn((
        Mesh3d(game_assets.tile_mesh.clone_weak()),
        MeshMaterial3d(materials.add(Color::from(palettes::tailwind::GRAY_400))),
        Transform::from_xyz(-1.0, -0.15, 0.0),
    ));

    // One
    commands.spawn((
        Mesh3d(game_assets.dot_mesh.clone_weak()),
        MeshMaterial3d(game_assets.dot_color.clone_weak()),
        Transform::from_xyz(-1.0, 0.0, 1.0),
    ));
    commands.spawn((
        Mesh3d(game_assets.tile_mesh.clone_weak()),
        MeshMaterial3d(materials.add(Color::from(palettes::tailwind::GRAY_400))),
        Transform::from_xyz(-1.0, -0.15, 1.0),
    ));

    // One
    commands.spawn((
        Mesh3d(game_assets.dot_mesh.clone_weak()),
        MeshMaterial3d(game_assets.dot_color.clone_weak()),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));
    commands.spawn((
        Mesh3d(game_assets.tile_mesh.clone_weak()),
        MeshMaterial3d(materials.add(Color::from(palettes::tailwind::GRAY_400))),
        Transform::from_xyz(0.0, -0.15, 1.0),
    ));

    // One
    commands.spawn((
        Mesh3d(game_assets.dot_mesh.clone_weak()),
        MeshMaterial3d(game_assets.dot_color.clone_weak()),
        Transform::from_xyz(1.0, 0.0, 1.0),
    ));
    commands.spawn((
        Mesh3d(game_assets.tile_mesh.clone_weak()),
        MeshMaterial3d(materials.add(Color::from(palettes::tailwind::GRAY_400))),
        Transform::from_xyz(1.0, -0.15, 1.0),
    ));

    // One
    commands.spawn((
        Mesh3d(game_assets.dot_mesh.clone_weak()),
        MeshMaterial3d(game_assets.dot_color.clone_weak()),
        Transform::from_xyz(-1.0, 0.0, -1.0),
    ));
    commands.spawn((
        Mesh3d(game_assets.tile_mesh.clone_weak()),
        MeshMaterial3d(materials.add(Color::from(palettes::tailwind::GRAY_400))),
        Transform::from_xyz(-1.0, -0.15, -1.0),
    ));

    // One
    commands.spawn((
        Mesh3d(game_assets.dot_mesh.clone_weak()),
        MeshMaterial3d(game_assets.dot_color.clone_weak()),
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
    commands.spawn((
        Mesh3d(game_assets.tile_mesh.clone_weak()),
        MeshMaterial3d(materials.add(Color::from(palettes::tailwind::GRAY_400))),
        Transform::from_xyz(0.0, -0.15, -1.0),
    ));

    // One
    commands.spawn((
        Mesh3d(game_assets.dot_mesh.clone_weak()),
        MeshMaterial3d(game_assets.dot_color.clone_weak()),
        Transform::from_xyz(1.0, 0.0, -1.0),
    ));
    commands.spawn((
        Mesh3d(game_assets.tile_mesh.clone_weak()),
        MeshMaterial3d(materials.add(Color::from(palettes::tailwind::GRAY_400))),
        Transform::from_xyz(1.0, -0.15, -1.0),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.9, 0.7),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 4.0, 0.0),
    ));
    commands.spawn((
        PointLight {
            intensity: 10_000_000.0,
            color: Color::srgb(1.0, 0.5, 0.0),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 8.0, 8.0),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 6.0, -3.0).looking_at(Vec3::ZERO, Vec3::Y),
        Msaa::Sample8,
        ShadowFilteringMethod::Gaussian,
    ));
}
