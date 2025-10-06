use std::time::Duration;

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;
use common::{
    ai::{Ai, Easiest, Easy, Hard, Medium},
    grid::Grid,
};

use crate::{CellColor, Config, CurrentTurn, Dot, DotCell, GameAssets, GameOperation, GridTray, PlayerConfigEntry, VisualGrid, spawn_dot};

pub fn tick_ai(
    mut commands: Commands,
    config: Res<Config>,
    current_player: Res<State<CurrentTurn>>,
    state: Res<State<GameOperation>>,
    mut next_state: ResMut<NextState<GameOperation>>,
    grid: Res<VisualGrid>,
    mut cells: Query<(&DotCell, &mut CellColor, &Transform)>,
    game_assets: Res<GameAssets>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    time: Res<Time>,
    mut timer: Local<Timer>,
    grid_tray: Query<Entity, With<GridTray>>,
    mut ais: Local<Vec<Box<dyn Ai>>>,
) {
    if ais.is_empty() {
        // Init
        ais.push(Box::new(Easiest::default()));
        ais.push(Box::new(Easy::default()));
        ais.push(Box::new(Medium::default()));
        ais.push(Box::new(Hard::default()));
    }

    if current_player.0 == 0 || *state != GameOperation::Bot {
        return;
    }

    let PlayerConfigEntry::Bot { level, .. } = config.players[current_player.0 - 1] else {
        next_state.set(GameOperation::Animating); // Something has gone dreadfully wrong. Bail.
        return;
    };
    let ai = &mut ais[level];

    let mut simple_grid = Grid::new(grid.width() as u8, grid.height() as u8, config.players.len() as u8);
    for (y, row) in grid.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            let (cell, cell_color, _) = cells.get(cell).unwrap();
            simple_grid[y][x].dots = cell.dots.len() as u8;
            simple_grid[y][x].owner = cell_color.player as u8;
        }
    }
    simple_grid.init_capacity();
    if state.is_changed() {
        timer.set_mode(TimerMode::Once);
        timer.set_duration(Duration::from_secs_f32(0.75));
        timer.reset();
        ai.start_move(&simple_grid);
    }

    // Sanity check to make sure there's a legal move for us
    let mut can_move = false;
    'top: for row in &simple_grid {
        for cell in row {
            if cell.owner == 0 || cell.owner == current_player.0 as u8 {
                can_move = true;
                break 'top;
            }
        }
    }
    if !can_move {
        next_state.set(GameOperation::Animating); // We lost. Bail.
        return;
    }
    let cell = ai.tick(&simple_grid, current_player.0 as u8, &mut rng);

    timer.tick(time.delta());
    if !timer.is_finished() {
        return;
    }

    if let Some((x, y)) = cell {
        if simple_grid[y][x].owner != 0 && simple_grid[y][x].owner != current_player.0 as u8 {
            // This is an illegal move. Don't do it.
            ai.start_move(&simple_grid);
            return;
        }
        let entity = grid[y as usize][x as usize];
        let (
            _,
            mut color,
            Transform {
                translation: Vec3 { x, z, .. },
                ..
            },
        ) = cells.get_mut(entity).unwrap();
        commands
            .entity(entity)
            .with_related::<Dot>((spawn_dot(*x, *z, &game_assets), ChildOf(grid_tray.single().unwrap())));
        color.player = current_player.0;
        timer.reset();
        next_state.set(GameOperation::Animating);
    }
}
