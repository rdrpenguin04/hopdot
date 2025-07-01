use std::{collections::VecDeque, time::Duration};

use bevy::{platform::collections::HashSet, prelude::*, render::render_resource::encase::private::Length};
use bevy_prng::WyRand;
use bevy_rand::{global::GlobalEntropy, prelude::Entropy};
use common::grid::Grid;
use rand::Rng as _;

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
    mut rng: GlobalEntropy<WyRand>,
    time: Res<Time>,
    mut timer: Local<Timer>,
    grid_tray: Query<Entity, With<GridTray>>,
) {
    if state.is_changed() {
        timer.set_mode(TimerMode::Once);
        timer.set_duration(Duration::from_secs_f32(0.75));
        timer.reset();
    }
    timer.tick(time.delta());
    if !timer.finished() || *state != GameOperation::Bot {
        return;
    }
    let mut simple_grid = Grid::new(grid.width(), grid.height());
    for (y, row) in grid.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            let (cell, cell_color, _) = cells.get(cell).unwrap();
            simple_grid[y][x].dots = cell.dots.length();
            simple_grid[y][x].owner = cell_color.player;
            let x_border = x == 0 || x == grid.width() - 1;
            let y_border = y == 0 || y == grid.height() - 1;
            simple_grid[y][x].capacity = if x_border && y_border {
                2
            } else if x_border || y_border {
                3
            } else {
                4
            };
        }
    }

    // Sanity check to make sure there's a legal move for us
    let mut can_move = false;
    'top: for row in &simple_grid {
        for cell in row {
            if cell.owner == 0 || cell.owner == current_player.0 {
                can_move = true;
                break 'top;
            }
        }
    }
    if !can_move {
        next_state.set(GameOperation::Animating); // We lost. Bail.
        return;
    }

    let PlayerConfigEntry::Bot { level, .. } = config.players[current_player.0 - 1] else {
        next_state.set(GameOperation::Animating); // Something has gone dreadfully wrong. Bail.
        return;
    };
    let cell = match level {
        0 => run_easiest(simple_grid, current_player.0, &mut rng),
        1 => run_easy(simple_grid, current_player.0, &mut rng),
        2 => run_medium(simple_grid, current_player.0, &mut rng),
        _ => todo!(),
    };
    if let Some((x, y)) = cell {
        let entity = grid[y][x];
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

fn run_easiest(grid: Grid, player: usize, rng: &mut Entropy<WyRand>) -> Option<(usize, usize)> {
    let mut new_cells = Vec::new();
    let mut mid_cells = Vec::new();
    let mut full_cells = Vec::new();
    let mut owned_cells = 0;
    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if cell.owner == 0 {
                new_cells.push((x, y));
            } else if cell.owner == player {
                owned_cells += 1;
                if cell.is_full() {
                    full_cells.push((x, y));
                } else {
                    mid_cells.push((x, y));
                }
            }
        }
    }
    if owned_cells < 2 && !new_cells.is_empty() {
        Some(new_cells[rng.random_range(0..new_cells.len())])
    } else if owned_cells > 0 {
        let choice = rng.random_range(0..10);
        if choice < 1 && !new_cells.is_empty() {
            Some(new_cells[rng.random_range(0..new_cells.len())])
        } else if choice < 4 && !full_cells.is_empty() {
            Some(full_cells[rng.random_range(0..full_cells.len())])
        } else if !mid_cells.is_empty() {
            Some(mid_cells[rng.random_range(0..mid_cells.len())])
        } else {
            None // Something went horribly wrong
        }
    } else {
        None
    }
}

fn run_easy(grid: Grid, player: usize, rng: &mut Entropy<WyRand>) -> Option<(usize, usize)> {
    let mut corner_count = 0;
    let mut viable_corners = Vec::new();
    for y in [0, grid.height() - 1] {
        for x in [0, grid.width() - 1] {
            if grid[y][x].owner == player {
                corner_count += 1;
            } else if grid[y][x].owner == 0 {
                viable_corners.push((x, y));
            }
        }
    }
    if corner_count < 2 && !viable_corners.is_empty() {
        return Some(viable_corners[rng.random_range(0..viable_corners.len())]);
    }
    let mut cascade_origins = Vec::new();
    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if cell.owner == player && cell.is_full() {
                cascade_origins.push(((x, y), 0));
            }
        }
    }
    for ((x, y), count) in &mut cascade_origins {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::from([(*x, *y)]);
        while let Some((x, y)) = queue.pop_front() {
            if visited.contains(&(x, y)) {
                continue;
            }
            visited.insert((x, y));
            *count += 1;
            if x > 0 && grid[y][x - 1].is_full() {
                queue.push_back((x - 1, y));
            }
            if y > 0 && grid[y - 1][x].is_full() {
                queue.push_back((x, y - 1));
            }
            if x < grid.width() - 1 && grid[y][x + 1].is_full() {
                queue.push_back((x + 1, y));
            }
            if y < grid.height() - 1 && grid[y + 1][x].is_full() {
                queue.push_back((x, y + 1));
            }
        }
    }
    // Ignore all cascades that don't start a chain. Baby want chaos.
    let cascade_origins = cascade_origins
        .into_iter()
        .filter_map(|(pos, count)| if count > 1 { Some(pos) } else { None })
        .collect::<Vec<_>>();
    if !cascade_origins.is_empty() {
        return Some(cascade_origins[rng.random_range(0..cascade_origins.len())]);
    }
    let mut owned_cells = Vec::new();
    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if cell.owner == player {
                owned_cells.push((x, y));
            }
        }
    }
    if !owned_cells.is_empty() {
        // This may include some minor cascades. Oh well.
        return Some(owned_cells[rng.random_range(0..owned_cells.len())]);
    }
    let mut unowned_cells = Vec::new();
    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if cell.owner == 0 {
                unowned_cells.push((x, y));
            }
        }
    }
    if !unowned_cells.is_empty() {
        // Last ditch: get a new cell
        return Some(unowned_cells[rng.random_range(0..unowned_cells.len())]);
    }
    None
}

fn run_medium(grid: Grid, player: usize, rng: &mut Entropy<WyRand>) -> Option<(usize, usize)> {
    let mut corner_count = 0;
    let mut viable_corners = Vec::new();
    for y in [0, grid.height() - 1] {
        for x in [0, grid.width() - 1] {
            if grid[y][x].owner == player {
                corner_count += 1;
            } else if grid[y][x].owner == 0 {
                viable_corners.push((x, y));
            }
        }
    }
    if corner_count < 2 && !viable_corners.is_empty() {
        return Some(viable_corners[rng.random_range(0..viable_corners.len())]);
    }
    let baseline_eval = grid.score_for_player(player);
    let mut evals = Vec::new();
    let mut winning_moves = Vec::new();
    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if cell.owner == player || cell.owner == 0 {
                let new_grid = grid.with_move(x, y, player);
                if let Some(new_grid) = new_grid {
                    evals.push(((x, y), new_grid.score_for_player(player)));
                } else {
                    winning_moves.push((x, y));
                }
            }
        }
    }
    if !winning_moves.is_empty() {
        // WE WON OMG WE ACTUALLY WON
        return Some(winning_moves[rng.random_range(0..winning_moves.len())]);
    }
    let max_eval = evals.iter().fold(baseline_eval, |prev_max, (_, eval)| prev_max.max(*eval));
    if max_eval - baseline_eval >= 2 {
        // We can actually make a dent if we do something. Let's do it.
        let candidates = evals
            .into_iter()
            .filter_map(|(pos, score)| if score == max_eval { Some(pos) } else { None })
            .collect::<Vec<_>>();
        return Some(candidates[rng.random_range(0..candidates.len())]);
    }
    // Alright, no cascades; let's look for anything else that doesn't shoot us in the foot
    let mut new_candidates = Vec::new();
    'outer: for &((x, y), eval) in &evals {
        // This is the "don't open the door" check. We don't want to build next to someone who will win.
        let cell = grid[y][x];
        let holes = cell.capacity - cell.dots;
        let mut neighbors = Vec::new();
        let mut eval = eval; // Copy; we don't want to edit the original evaluation.
        if x > 0 {
            neighbors.push(grid[y][x - 1]);
        }
        if y > 0 {
            neighbors.push(grid[y - 1][x]);
        }
        if x < grid.width() - 1 {
            neighbors.push(grid[y][x + 1]);
        }
        if y < grid.height() - 1 {
            neighbors.push(grid[y + 1][x]);
        }
        for neighbor in neighbors {
            if neighbor.owner == player {
                continue;
            }
            let n_holes = neighbor.capacity - neighbor.dots;
            if n_holes < holes {
                // They will cascade first. Don't chance it.
                continue 'outer;
            } else if n_holes > holes {
                // We're going to win. Let's do this.
                eval += 1;
            }
        }
        // Okay, we passed the check. It's a candidate move now.
        new_candidates.push(((x, y), eval));
    }
    let max_eval = new_candidates.iter().fold(baseline_eval, |prev_max, (_, eval)| prev_max.max(*eval));
    if !new_candidates.is_empty() {
        let final_candidates = new_candidates
            .into_iter()
            .filter_map(|(pos, score)| if score == max_eval { Some(pos) } else { None })
            .collect::<Vec<_>>();
        return Some(final_candidates[rng.random_range(0..final_candidates.len())]);
    }
    // If we got here, there are no good moves. Do something so we aren't deadlocked.
    let max_eval = evals.iter().fold(baseline_eval, |prev_max, (_, eval)| prev_max.max(*eval));
    if !evals.is_empty() {
        let really_final_candidates = evals
            .into_iter()
            .filter_map(|(pos, score)| if score == max_eval { Some(pos) } else { None })
            .collect::<Vec<_>>();
        return Some(really_final_candidates[rng.random_range(0..really_final_candidates.len())]);
    }
    None // Something has gone dreadfully wrong
}
