use std::{
    ops::{Index, IndexMut},
    time::Duration,
};

use bevy::{prelude::*, render::render_resource::encase::private::Length};
use bevy_prng::WyRand;
use bevy_rand::{global::GlobalEntropy, prelude::Entropy};
use rand::Rng as _;

use crate::{
    CellColor, CellGrid, Config, CurrentTurn, Dot, DotCell, GameAssets, GameOperation,
    PlayerConfigEntry, spawn_dot,
};

#[derive(Clone, Copy)]
struct SimpleCell {
    dots: usize,
    owner: usize,
    capacity: usize,
}

impl SimpleCell {
    pub fn is_full(&self) -> bool {
        self.dots == self.capacity
    }
}

impl Default for SimpleCell {
    fn default() -> Self {
        Self {
            dots: 1,
            owner: 0,
            capacity: 0,
        }
    }
}

#[derive(Clone)]
struct SimpleGrid {
    grid: Vec<SimpleCell>,
    width: usize,
}

impl SimpleGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: vec![SimpleCell::default(); width * height],
            width,
        }
    }

    pub fn new_inplace(&mut self, width: usize, height: usize) {
        self.grid = vec![SimpleCell::default(); width * height];
        self.width = width;
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.grid.len() / self.width
    }

    pub fn iter(&self) -> core::slice::ChunksExact<SimpleCell> {
        self.grid.chunks_exact(self.width)
    }
}

impl Index<usize> for SimpleGrid {
    type Output = [SimpleCell];

    fn index(&self, index: usize) -> &[SimpleCell] {
        &self.grid[(index * self.width)..((index + 1) * self.width)]
    }
}

impl IndexMut<usize> for SimpleGrid {
    fn index_mut(&mut self, index: usize) -> &mut [SimpleCell] {
        &mut self.grid[(index * self.width)..((index + 1) * self.width)]
    }
}

impl<'a> IntoIterator for &'a SimpleGrid {
    type Item = &'a [SimpleCell];

    type IntoIter = core::slice::ChunksExact<'a, SimpleCell>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub fn tick_ai(
    mut commands: Commands,
    config: Res<Config>,
    current_player: Res<State<CurrentTurn>>,
    state: Res<State<GameOperation>>,
    mut next_state: ResMut<NextState<GameOperation>>,
    grid: Res<CellGrid>,
    mut cells: Query<(&DotCell, &mut CellColor, &Transform)>,
    game_assets: Res<GameAssets>,
    mut rng: GlobalEntropy<WyRand>,
    time: Res<Time>,
    mut timer: Local<Timer>,
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
    let mut simple_grid = SimpleGrid::new(grid.width(), grid.height());
    for (y, row) in grid.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            let (cell, cell_color, _) = cells.get(cell).unwrap();
            simple_grid[y][x].dots = cell.dots.length();
            simple_grid[y][x].owner = cell_color.player;
            let x_border = x == 0 || x == grid.width() - 1;
            let y_border = y == 0 || y == grid.width() - 1;
            simple_grid[y][x].capacity = if x_border && y_border {
                2
            } else if x_border || y_border {
                3
            } else {
                4
            };
        }
    }

    let PlayerConfigEntry::Bot { level, .. } = config.players[current_player.0 - 1] else {
        next_state.set(GameOperation::Animating); // Something has gone dreadfully wrong. Bail.
        return;
    };
    let cell = match level {
        0 => run_easiest(simple_grid, current_player.0, &mut rng),
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
            .with_related::<Dot>(spawn_dot(*x, *z, &game_assets));
        color.player = current_player.0;
        timer.reset();
        next_state.set(GameOperation::Animating);
    }
}

fn run_easiest(
    grid: SimpleGrid,
    player: usize,
    rng: &mut Entropy<WyRand>,
) -> Option<(usize, usize)> {
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
