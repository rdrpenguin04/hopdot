use bevy::prelude::*;

use crate::{CellColor, Config, Dot, DotCell, GRAY, PlayerConfigEntry};

#[derive(Component, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct TargetTransform(pub Transform);

#[derive(Component, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct TargetMaterialColor(pub Color);

#[derive(Component, Default, Reflect)]
pub struct SmoothingSettings {
    pub translation_decay_rate: f32,
    pub rotation_decay_rate: f32,
    pub scale_decay_rate: f32,
}

#[derive(Component, Reflect)]
pub struct Bouncing(pub f64);

#[derive(Resource, Reflect)]
pub struct TargetUiOpacity(pub f32);

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            arrange_dots,
            animate_cell_colors,
            animate_material_colors.after(animate_cell_colors),
            smooth_transform,
            run_bouncing,
            run_ui_opacity,
        ),
    )
    .insert_resource(TargetUiOpacity(0.0));
}

fn smooth_transform(mut transforms: Query<(&TargetTransform, &SmoothingSettings, &mut Transform)>, time: Res<Time>) {
    for (target, settings, mut transform) in &mut transforms {
        transform
            .translation
            .smooth_nudge(&target.translation, settings.translation_decay_rate, time.delta_secs());
        transform
            .rotation
            .smooth_nudge(&target.rotation, settings.rotation_decay_rate, time.delta_secs());
        transform.scale.smooth_nudge(&target.scale, settings.scale_decay_rate, time.delta_secs());
    }
}

fn arrange_dots(cells: Query<(&DotCell, &Transform)>, mut dots: Query<&mut TargetTransform, With<Dot>>) {
    for (cell, transform) in &cells {
        let (cell_x, cell_z) = (transform.translation.x, transform.translation.z);
        let arrangement: &[(f32, f32)] = match cell.dots.len() {
            0 => continue, // Something has gone dreadfully wrong. Or we're early. Fail with grace.
            1 => &[(0.0, 0.0)],
            2 => &[(-0.25, 0.25), (0.25, -0.25)],
            3 => &[(-0.25, 0.25), (0.25, -0.25), (0.0, 0.0)],
            4 => &[(-0.25, 0.25), (0.25, -0.25), (-0.25, -0.25), (0.25, 0.25)],
            5 => &[(-0.25, 0.25), (0.25, -0.25), (-0.25, -0.25), (0.25, 0.25), (0.0, 0.0)],
            6 => &[(-0.25, 0.25), (0.25, -0.25), (-0.25, -0.25), (0.25, 0.25), (-0.25, 0.0), (0.25, 0.0)],
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

fn animate_cell_colors(mut cells: Query<(&mut TargetMaterialColor, &CellColor)>, player_config: Res<Config>) {
    for (mut material, color_idx) in &mut cells {
        let target_color = if color_idx.player == 0 {
            GRAY
        } else {
            match player_config.players[color_idx.player - 1] {
                PlayerConfigEntry::Human { color, .. } => color,
                PlayerConfigEntry::Bot { color, .. } => color,
            }
        };
        material.0 = target_color;
    }
}

fn animate_material_colors(
    meshes: Query<(&MeshMaterial3d<StandardMaterial>, &TargetMaterialColor)>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (material, target_color) in &meshes {
        if let Color::Srgba(target_color) = target_color.0 {
            let material = materials.get_mut(material.id()).unwrap();
            if let Color::Srgba(srgba) = &mut material.base_color {
                let mut temp = srgba.to_vec4();
                let target_color_vec = target_color.to_vec4();
                temp.smooth_nudge(&target_color_vec, 3.0, time.delta_secs());
                *srgba = Srgba::from_vec4(temp);
            } else if let Color::LinearRgba(srgba) = &mut material.base_color {
                let mut temp = srgba.to_vec4();
                let target_color_vec = target_color.to_vec4();
                temp.smooth_nudge(&target_color_vec, 3.0, time.delta_secs());
                *srgba = LinearRgba::from_vec4(temp);
            } else {
                dbg!(material.base_color);
            }
            let mut temp = material.emissive.to_vec4();
            temp.smooth_nudge(&Vec4::ZERO, 10.0, time.delta_secs());
            material.emissive = LinearRgba::from_vec4(temp);
        }
    }
}

fn run_bouncing(mut commands: Commands, mut bouncing_objects: Query<(Entity, &Bouncing, &mut TargetTransform, &mut Transform)>, time: Res<Time>) {
    let elapsed = time.elapsed_secs_f64();
    for (entity, bouncing, mut target, mut transform) in &mut bouncing_objects {
        let t = (elapsed - bouncing.0) as f32;
        if t < 0.5 {
            // Directly pilot the ball's height
            let scalar = (vec2(target.0.translation.x, target.0.translation.z) - vec2(transform.translation.x, transform.translation.z)).length();
            target.0.translation.y = 16.0 * (t - 2.0 * t * t) * scalar;
            transform.translation.y = 16.0 * (t - 2.0 * t * t) * scalar;
        } else {
            target.0.translation.y = 0.0;
            commands.entity(entity).remove::<Bouncing>();
        }
    }
}

#[derive(Component)]
pub struct AnimateBackgroundColor;

fn run_ui_opacity(
    text_colors: Query<&mut TextColor>,
    outlines: Query<&mut Outline>,
    background_colors: Query<&mut BackgroundColor, With<AnimateBackgroundColor>>,
    target_ui_opacity: Res<TargetUiOpacity>,
    time: Res<Time>,
) {
    for mut text_color in text_colors {
        let mut alpha = text_color.alpha();
        alpha.smooth_nudge(&target_ui_opacity.0, 5.0, time.delta_secs());
        text_color.set_alpha(alpha);
    }
    for mut outline in outlines {
        let mut alpha = outline.color.alpha();
        alpha.smooth_nudge(&target_ui_opacity.0, 5.0, time.delta_secs());
        outline.color.set_alpha(alpha);
    }
    for mut background_color in background_colors {
        let mut alpha = background_color.0.alpha();
        alpha.smooth_nudge(&target_ui_opacity.0, 5.0, time.delta_secs());
        background_color.0.set_alpha(alpha);
    }
}
