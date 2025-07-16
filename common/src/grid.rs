use core::fmt;
use std::{
    cell::RefCell,
    collections::VecDeque,
    ops::{Index, IndexMut},
    slice::ChunksExact,
};

use bytemuck::TransparentWrapper;

#[derive(Clone, Copy, Debug)]
pub struct GridCell {
    pub dots: u8,
    pub owner: u8,
    pub capacity: u8,
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

#[derive(Clone, Debug)]
pub struct Grid {
    grid: Vec<GridCell>,
    width: u8,
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut line_sep = "";
        for row in self {
            f.write_str(line_sep)?;
            for cell in row {
                write!(f, "[{};{}]", cell.owner, cell.dots)?;
            }
            line_sep = "\n";
        }
        Ok(())
    }
}

thread_local! {
    pub static VISITED_BUF: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
}

impl Grid {
    pub fn new(width: u8, height: u8) -> Self {
        Self {
            grid: vec![GridCell::default(); width as usize * height as usize],
            width,
        }
    }

    pub fn init_capacity(&mut self) {
        let width = self.width() as usize;
        let height = self.height() as usize;
        for (y, row) in self.iter_mut().enumerate() {
            let ud_edge = y == 0 || y == height - 1;
            for (x, cell) in row.iter_mut().enumerate() {
                let lr_edge = x == 0 || x == width - 1;
                if ud_edge && lr_edge {
                    cell.capacity = 2;
                } else if ud_edge || lr_edge {
                    cell.capacity = 3;
                } else {
                    cell.capacity = 4;
                }
            }
        }
    }

    pub const fn width(&self) -> u8 {
        self.width
    }

    pub const fn height(&self) -> u8 {
        (self.grid.len() / self.width as usize) as u8
    }

    pub fn iter(&self) -> GridIter<'_> {
        GridIter::new(self)
    }

    pub fn iter_mut(&mut self) -> core::slice::ChunksExactMut<'_, GridCell> {
        self.grid.chunks_exact_mut(self.width as usize)
    }

    // If this returns None, the board went into a loop.
    pub fn with_move(&self, x: u8, y: u8, player: u8) -> (Option<Self>, bool) {
        VISITED_BUF.with_borrow_mut(|visited| {
            if visited.len() < self.grid.len() {
                visited.extend(core::iter::repeat_n(false, self.grid.len() - visited.len()));
            }
            #[allow(clippy::needless_range_loop)] // Looks cleaner than the alternative
            for i in 0..self.grid.len() {
                visited[i] = false;
            }
            let mut result = self.clone();
            result[y][x].dots += 1;
            result[y][x].owner = player;

            let mut visited_count = 0;
            let mut cascade_queue = VecDeque::from([(x, y)]);
            let mut cascaded = false;

            while let Some((x, y)) = cascade_queue.pop_front() {
                // We've hit every square on the board. The game is over.
                if visited_count == result.width() * result.height() {
                    return (None, true);
                }
                let idx = y as usize * self.width as usize + x as usize;
                if !visited[idx] {
                    visited_count += 1;
                }
                visited[idx] = true;
                // TODO: maybe replacing these with result.grid[idx] (and modifications) would be faster? Needs analysis
                result[y][x].owner = player;
                if result[y][x].dots > result[y][x].capacity {
                    cascaded = true;
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

            (Some(result), cascaded)
        })
    }

    pub fn score_for_player(&self, player: u8) -> i32 {
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

    // TODO: Add this as a field
    pub fn player_count(&self) -> u8 {
        2
    }
}

#[allow(clippy::type_complexity)] // TODO: decide if this is worth fixing
pub struct GridIter<'a>(core::iter::Map<ChunksExact<'a, GridCell>, fn(&[GridCell]) -> &GridRow>);

impl<'a> GridIter<'a> {
    pub fn new(grid: &'a Grid) -> Self {
        Self(
            grid.grid
                .chunks_exact(grid.width as usize)
                .map(|x| GridRow::wrap_ref(x)),
        )
    }

    pub fn enumerate_u8(self) -> impl Iterator<Item = (u8, &'a GridRow)> {
        self.0.enumerate().map(|(i, x)| (i as u8, x))
    }
}

impl<'a> Iterator for GridIter<'a> {
    type Item = &'a GridRow;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl Index<usize> for Grid {
    type Output = GridRow;

    fn index(&self, index: usize) -> &GridRow {
        let row = &self.grid[(index * self.width as usize)..((index + 1) * self.width as usize)];
        GridRow::wrap_ref(row)
    }
}

impl Index<u8> for Grid {
    type Output = GridRow;

    fn index(&self, index: u8) -> &GridRow {
        &self[index as usize]
    }
}

impl IndexMut<usize> for Grid {
    fn index_mut(&mut self, index: usize) -> &mut GridRow {
        let row =
            &mut self.grid[(index * self.width as usize)..((index + 1) * self.width as usize)];
        GridRow::wrap_mut(row)
    }
}

impl IndexMut<u8> for Grid {
    fn index_mut(&mut self, index: u8) -> &mut GridRow {
        &mut self[index as usize]
    }
}

impl<'a> IntoIterator for &'a Grid {
    type Item = &'a GridRow;

    type IntoIter = impl Iterator<Item = &'a GridRow>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(TransparentWrapper)]
#[repr(transparent)]
pub struct GridRow([GridCell]);

impl GridRow {
    pub fn iter(&self) -> GridRowIter<'_> {
        GridRowIter::new(self)
    }
}

impl Index<usize> for GridRow {
    type Output = GridCell;

    fn index(&self, index: usize) -> &GridCell {
        &self.0[index]
    }
}

impl Index<u8> for GridRow {
    type Output = GridCell;

    fn index(&self, index: u8) -> &GridCell {
        &self[index as usize]
    }
}

impl IndexMut<usize> for GridRow {
    fn index_mut(&mut self, index: usize) -> &mut GridCell {
        &mut self.0[index]
    }
}

impl IndexMut<u8> for GridRow {
    fn index_mut(&mut self, index: u8) -> &mut GridCell {
        &mut self[index as usize]
    }
}

impl<'a> IntoIterator for &'a GridRow {
    type Item = &'a GridCell;

    type IntoIter = impl Iterator<Item = &'a GridCell>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct GridRowIter<'a>(core::slice::Iter<'a, GridCell>);

impl<'a> GridRowIter<'a> {
    pub fn new(row: &'a GridRow) -> Self {
        Self(row.0.iter())
    }

    pub fn enumerate_u8(self) -> impl Iterator<Item = (u8, &'a GridCell)> {
        self.0.enumerate().map(|(i, x)| (i as u8, x))
    }
}

impl<'a> Iterator for GridRowIter<'a> {
    type Item = &'a GridCell;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
