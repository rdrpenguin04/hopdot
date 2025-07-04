use std::{
    collections::{HashSet, VecDeque},
    ops::{Index, IndexMut},
};

#[derive(Clone, Copy)]
pub struct GridCell {
    pub dots: usize,
    pub owner: usize,
    pub capacity: usize,
}

impl GridCell {
    pub fn is_full(&self) -> bool {
        self.dots == self.capacity
    }
}

impl Default for GridCell {
    fn default() -> Self {
        Self {
            dots: 1,
            owner: 0,
            capacity: 0,
        }
    }
}

#[derive(Clone)]
pub struct Grid {
    grid: Vec<GridCell>,
    width: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: vec![GridCell::default(); width * height],
            width,
        }
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.grid.len() / self.width
    }

    pub fn iter(&self) -> core::slice::ChunksExact<'_, GridCell> {
        self.grid.chunks_exact(self.width)
    }

    // If this returns None, the board went into a loop.
    pub fn with_move(&self, x: usize, y: usize, player: usize) -> Option<Self> {
        let mut result = self.clone();
        result[y][x].dots += 1;
        result[y][x].owner = player;

        let mut visited = HashSet::new();
        let mut visited_count = 0;
        let mut cascade_queue = VecDeque::from([(x, y)]);

        while let Some((x, y)) = cascade_queue.pop_front() {
            // We've hit every square on the board. The game is over.
            if visited_count == result.width() * result.height() {
                return None;
            }
            if visited.insert((x, y)) {
                visited_count += 1;
            }
            result[y][x].owner = player;
            if result[y][x].dots > result[y][x].capacity {
                result[y][x].dots -= result[y][x].capacity;

                if x > 0 {
                    result[y][x - 1].dots += 1;
                    cascade_queue.push_back((x - 1, y));
                }
                if y > 0 {
                    result[y - 1][x].dots += 1;
                    cascade_queue.push_back((x, y - 1));
                }
                if x < result.width() - 1 {
                    result[y][x + 1].dots += 1;
                    cascade_queue.push_back((x + 1, y));
                }
                if y < result.height() - 1 {
                    result[y + 1][x].dots += 1;
                    cascade_queue.push_back((x, y + 1));
                }
            }
        }

        Some(result)
    }

    pub fn score_for_player(&self, player: usize) -> i32 {
        let mut result = 0;
        for cell in &self.grid {
            if cell.owner == player {
                result += 1;
            } else if cell.owner != player && cell.owner != 0 {
                result -= 1;
            }
        }
        result
    }
}

impl Index<usize> for Grid {
    type Output = [GridCell];

    fn index(&self, index: usize) -> &[GridCell] {
        &self.grid[(index * self.width)..((index + 1) * self.width)]
    }
}

impl IndexMut<usize> for Grid {
    fn index_mut(&mut self, index: usize) -> &mut [GridCell] {
        &mut self.grid[(index * self.width)..((index + 1) * self.width)]
    }
}

impl<'a> IntoIterator for &'a Grid {
    type Item = &'a [GridCell];

    type IntoIter = core::slice::ChunksExact<'a, GridCell>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
