use core::fmt;
use std::collections::VecDeque;

use ahash::{HashSet, HashSetExt as _};
use rand::{Rng, RngCore};

use crate::grid::Grid;

pub trait Ai: Send + Sync {
    /// Called at the start of a new move's analysis time
    fn start_move(&mut self, grid: &Grid);

    /// Called once per frame to update the AI's state. Should be limited to roughly 1/60th of a second in time.
    ///
    /// Returns `None` if the AI isn't ready yet or `Some((x, y))` if it is. This function is expected to continue returning `Some` for every tick after it first has a result, though the specific cell chosen is allowed to change.
    fn tick(&mut self, grid: &Grid, player: u8, rng: &mut dyn RngCore) -> Option<(u8, u8)>;

    fn name(&self) -> &str;
}

impl<'a> core::fmt::Debug for dyn Ai + 'a {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Default)]
pub struct Easiest(Option<(u8, u8)>);

impl Easiest {
    fn tick_inner(&mut self, grid: &Grid, player: u8, rng: &mut dyn RngCore) -> Option<(u8, u8)> {
        let mut new_cells = Vec::new();
        let mut mid_cells = Vec::new();
        let mut full_cells = Vec::new();
        let mut owned_cells = 0;
        for (y, row) in grid.iter().enumerate_u8() {
            for (x, cell) in row.iter().enumerate_u8() {
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
}

impl Ai for Easiest {
    fn start_move(&mut self, _: &Grid) {
        self.0 = None;
    }

    fn tick(&mut self, grid: &Grid, player: u8, rng: &mut dyn RngCore) -> Option<(u8, u8)> {
        if self.0.is_none() {
            self.0 = self.tick_inner(grid, player, rng);
        }
        self.0
    }

    fn name(&self) -> &str {
        "Easiest"
    }
}

#[derive(Default)]
pub struct Easy(Option<(u8, u8)>);

impl Easy {
    fn tick_inner(&mut self, grid: &Grid, player: u8, rng: &mut dyn RngCore) -> Option<(u8, u8)> {
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
        for (y, row) in grid.iter().enumerate_u8() {
            for (x, cell) in row.iter().enumerate_u8() {
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
        for (y, row) in grid.iter().enumerate_u8() {
            for (x, cell) in row.iter().enumerate_u8() {
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
        for (y, row) in grid.iter().enumerate_u8() {
            for (x, cell) in row.iter().enumerate_u8() {
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
}

impl Ai for Easy {
    fn start_move(&mut self, _: &Grid) {
        self.0 = None;
    }

    fn tick(&mut self, grid: &Grid, player: u8, rng: &mut dyn RngCore) -> Option<(u8, u8)> {
        if self.0.is_none() {
            self.0 = self.tick_inner(grid, player, rng);
        }
        self.0
    }

    fn name(&self) -> &str {
        "Easy"
    }
}

#[derive(Default)]
pub struct Medium<const FAIL_CHANCE: u8>(Option<(u8, u8)>);

impl<const FAIL_CHANCE: u8> Medium<FAIL_CHANCE> {
    fn tick_inner(&mut self, grid: &Grid, player: u8, rng: &mut dyn RngCore) -> Option<(u8, u8)> {
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
        for (y, row) in grid.iter().enumerate_u8() {
            for (x, cell) in row.iter().enumerate_u8() {
                if cell.owner == player || cell.owner == 0 {
                    let (new_grid, _) = grid.with_move(x, y, player);
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
        let max_eval = evals
            .iter()
            .fold(baseline_eval, |prev_max, (_, eval)| prev_max.max(*eval));
        if max_eval - baseline_eval >= 2 && rng.random::<u8>() >= FAIL_CHANCE {
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
        let max_eval = new_candidates
            .iter()
            .fold(baseline_eval, |prev_max, (_, eval)| prev_max.max(*eval));
        if !new_candidates.is_empty() && rng.random::<u8>() >= FAIL_CHANCE {
            let final_candidates = new_candidates
                .into_iter()
                .filter_map(|(pos, score)| if score == max_eval { Some(pos) } else { None })
                .collect::<Vec<_>>();
            return Some(final_candidates[rng.random_range(0..final_candidates.len())]);
        }
        // If we got here, there are no good moves. Do something so we aren't deadlocked.
        let max_eval = evals
            .iter()
            .fold(baseline_eval, |prev_max, (_, eval)| prev_max.max(*eval));
        if !evals.is_empty() {
            let really_final_candidates = evals
                .into_iter()
                .filter_map(|(pos, score)| if score == max_eval { Some(pos) } else { None })
                .collect::<Vec<_>>();
            return Some(
                really_final_candidates[rng.random_range(0..really_final_candidates.len())],
            );
        }
        None // Something has gone dreadfully wrong
    }
}

impl<const FAIL_CHANCE: u8> Ai for Medium<FAIL_CHANCE> {
    fn start_move(&mut self, _: &Grid) {
        self.0 = None;
    }

    fn tick(&mut self, grid: &Grid, player: u8, rng: &mut dyn RngCore) -> Option<(u8, u8)> {
        if self.0.is_none() {
            self.0 = self.tick_inner(grid, player, rng);
        }
        self.0
    }

    fn name(&self) -> &str {
        "Medium"
    }
}

use std::fmt::Debug;

#[derive(Clone)]
pub struct TreeNodeState<T: PartialOrd + Copy> {
    grid: Option<Grid>, // If this is `None`, the game is over.
    moves: Box<[TreeNode<T>]>,
    score: T,
    unvisited_children: u16,
}

impl<T: PartialOrd + Copy> TreeNodeState<T> {
    fn move_mut(&mut self, x: u8, y: u8) -> &mut TreeNode<T> {
        let width = self.grid.as_ref().unwrap().width();
        &mut self.moves[x as usize + y as usize * width as usize]
    }

    fn moves_iter(&self) -> impl Iterator<Item = ((u8, u8), &TreeNode<T>)> {
        let width = self.grid.as_ref().unwrap().width();
        self.moves.iter().enumerate().map(move |(i, x)| {
            (
                ((i % (width as usize)) as u8, (i / (width as usize)) as u8),
                x,
            )
        })
    }
}

impl<T: PartialOrd + Copy + Debug> fmt::Debug for TreeNodeState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeNodeState")
            .field("moves", &self.moves)
            .field("score", &self.score)
            .field("unvis", &self.unvisited_children)
            .finish()
    }
}

#[derive(Clone, Default, Debug)]
pub enum TreeNode<T: PartialOrd + Copy> {
    #[default]
    Vacant,
    State(TreeNodeState<T>),
}

#[derive(Clone, Copy)]
struct MoveSegment((u8, u8));

impl MoveSegment {
    const fn from_depth(depth: u8) -> Self {
        Self((depth, 0xFF))
    }

    const fn from_move(m: (u8, u8)) -> Self {
        Self(m)
    }

    const fn as_depth(self) -> Option<u8> {
        if self.0.1 == 255 {
            Some(self.0.0)
        } else {
            None
        }
    }

    const fn as_move(self) -> Option<(u8, u8)> {
        if self.0.1 == 255 { None } else { Some(self.0) }
    }
}

#[derive(Default)]
pub struct MoveQueue(Vec<MoveSegment>);

impl MoveQueue {
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn push(&mut self, m: &[(u8, u8)]) {
        assert!(m.len() < 256, "analysis depth above 255 is not supported");
        self.0.extend(m.iter().map(|x| MoveSegment::from_move(*x)));
        self.0.push(MoveSegment::from_depth(m.len() as u8));
    }

    pub fn push_suffixed(&mut self, m: &[(u8, u8)], next: (u8, u8)) {
        assert!(m.len() < 255, "analysis depth above 255 is not supported");
        self.0.extend(m.iter().map(|x| MoveSegment::from_move(*x)));
        self.0.push(MoveSegment::from_move(next));
        self.0.push(MoveSegment::from_depth(m.len() as u8 + 1));
    }

    pub fn pop(&mut self) -> Option<impl ExactSizeIterator<Item = (u8, u8)>> {
        let depth = self.0.pop()?;
        let Some(depth) = depth.as_depth() else {
            panic!("queue in invalid state")
        };
        Some(self.0.drain((self.0.len() - depth as usize)..).map(|x| {
            let Some(m) = x.as_move() else {
                panic!("queue in invalid state")
            };
            m
        }))
    }
}

pub enum EvalStatus {
    Done,
    Cascaded,
    Uneventful,
}

pub struct TreeState<T: PartialOrd + Copy> {
    root: TreeNode<T>,
    me: u8,
    grid: Option<Grid>,
    eval_queue: MoveQueue,
    moves_buf: Vec<(u8, u8)>,
}

impl<T: PartialOrd + Copy + Debug> fmt::Debug for TreeState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeState")
            .field("root", &self.root)
            .field("me", &self.me)
            .finish()
    }
}

impl<T: PartialOrd + Copy + Debug> Default for TreeState<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PartialOrd + Copy + Debug> TreeState<T> {
    pub fn new() -> Self {
        Self {
            root: TreeNode::Vacant,
            me: 0,
            grid: None,
            eval_queue: MoveQueue::default(),
            moves_buf: Vec::with_capacity(32), // Some extra buffer
        }
    }

    pub fn set_grid(&mut self, grid: Grid) {
        self.grid = Some(grid);
        self.eval_queue.clear();
        self.eval_queue.push(&[]);
    }

    pub fn set_player(&mut self, me: u8) {
        self.me = me;
    }

    /// The evaluator funtion takes three parameters:
    /// * the current grid to evaluate (or None if the player won)
    /// * the player whose turn it currently is
    /// * the player whose score we care about (corresponds to the player set by [`Self::set_player`])
    ///
    /// # Panics
    /// This function may panic if the state has not been initialized properly, such as:
    /// * if the active grid has not been set with [`Self::set_grid`]
    /// * if the current player has not been set with [`Self::set_player`]
    pub fn eval_next(
        &mut self,
        eval: impl FnOnce(Option<&Grid>, u8, u8) -> T,
        max_depth: usize,
    ) -> EvalStatus {
        let Some(grid) = &self.grid else {
            panic!("call `set_grid` before `eval_next`")
        };
        if self.me == 0 {
            panic!("call `set_player` before `eval_next`");
        }

        let Some(moves) = self.eval_queue.pop() else {
            return EvalStatus::Done; // Out of legal moves
        };
        self.moves_buf.clear();
        self.moves_buf.extend(moves);
        let mut cur_grid = Some(grid);
        let player_count = grid.player_count();
        let mut cur_player = (self.me + player_count - 1) % player_count;
        let mut node = &mut self.root;

        // println!("\nStarting move: {:?}", self.moves_buf);

        let (grid, player, cascade) = if self.moves_buf.is_empty() {
            (cur_grid.cloned(), cur_player, false)
        } else {
            let mut moves_iter = self.moves_buf.iter().copied().peekable();
            loop {
                let m = moves_iter.next().unwrap();
                let TreeNode::State(node_inner) = node else {
                    panic!("Error in pruning")
                };
                // println!("m = {m:?}, node = {node_inner:?}");
                cur_grid = node_inner.grid.as_ref();
                let grid = cur_grid.unwrap();
                node_inner.unvisited_children -= 1;
                cur_player = cur_player % player_count + 1;
                if moves_iter.peek().is_some() {
                    node = node_inner.move_mut(m.0, m.1);
                } else {
                    let (grid, cascade) = grid.with_move(m.0, m.1, cur_player);
                    node = node_inner.move_mut(m.0, m.1);
                    break (grid, cur_player, cascade);
                }
            }
        };

        // if let Some(grid) = &grid {
        //     println!("{grid}");
        // } else {
        //     println!("gone infinite");
        // }

        let score = eval(grid.as_ref(), player, self.me);

        cur_player = cur_player % player_count + 1;
        let mut num_moves = 0;
        let moves = {
            if self.moves_buf.len() < max_depth
                && let Some(grid) = &grid
            {
                for (y, row) in grid.iter().enumerate_u8() {
                    for (x, cell) in row.iter().enumerate_u8() {
                        if cell.owner == 0 || cell.owner == cur_player {
                            num_moves += 1;
                            self.eval_queue.push_suffixed(&self.moves_buf, (x, y));
                        }
                    }
                }

                vec![TreeNode::Vacant; grid.width() as usize * grid.height() as usize]
                    .into_boxed_slice()
            } else {
                Box::new([])
            }
        };
        *node = TreeNode::State(TreeNodeState {
            grid,
            moves, // populated on explore
            score,
            unvisited_children: num_moves,
        });

        let mut node_to_update = &mut self.root;
        for m in &self.moves_buf {
            let TreeNode::State(s) = node_to_update else {
                unreachable!()
            };
            s.unvisited_children += num_moves;
            node_to_update = s.move_mut(m.0, m.1);
        }

        Self::propagate_recursive(&mut self.root, &self.moves_buf, self.me, self.me);

        if cascade {
            EvalStatus::Cascaded
        } else {
            EvalStatus::Uneventful
        }
    }

    pub fn clear(&mut self) {
        self.root = TreeNode::Vacant;
        self.me = 0;
    }

    fn propagate_recursive(node: &mut TreeNode<T>, moves: &[(u8, u8)], player: u8, me: u8) {
        let TreeNode::State(node) = node else {
            unreachable!("all nodes up to this point should have a state")
        };
        let Some((m, rest)) = moves.split_first() else {
            return;
        };
        if !rest.is_empty() {
            let player_count = node.grid.as_ref().unwrap().player_count();
            Self::propagate_recursive(node.move_mut(m.0, m.1), rest, player % player_count + 1, me);
        }
        if !node.moves.is_empty() {
            let mut new_score = None;
            for node in &node.moves {
                if let TreeNode::State(s) = node {
                    if let Some(new_score) = &mut new_score {
                        if (player == me && s.score > *new_score)
                            || (player != me && s.score < *new_score)
                        {
                            *new_score = s.score;
                        }
                    } else {
                        new_score = Some(s.score);
                    }
                }
            }
            if let Some(new_score) = new_score {
                node.score = new_score;
                if node.unvisited_children == 0 {
                    for node in &mut node.moves {
                        let remove = if let TreeNode::State(s) = node {
                            (player == me && s.score < new_score)
                                || (player != me && s.score > new_score)
                        } else {
                            false
                        };
                        if remove {
                            *node = TreeNode::Vacant;
                        }
                    }
                }
            }
        }
    }

    pub fn iter_moves_and_score(&self) -> impl Iterator<Item = ((u8, u8), T)> {
        let TreeNode::State(s) = &self.root else {
            panic!("need at least one evaluation round")
        };
        s.moves_iter().filter_map(|(m, n)| {
            if let TreeNode::State(n) = n {
                Some((m, n.score))
            } else {
                None
            }
        })
    }
}

#[derive(Default)]
pub struct Hard<Fallback: Ai + Default, const FALLBACK_CHANCE: u8> {
    decision: Option<(u8, u8)>,
    tree_state: TreeState<i32>,
    num_players: u8,
    fallback: Fallback,
}

impl<Fallback: Ai + Default, const FALLBACK_CHANCE: u8> Hard<Fallback, FALLBACK_CHANCE> {
    fn tick_inner(
        &mut self,
        _: &Grid, // TODO: should this be removed from tick() if it'll stay the same the whole time?
        player: u8, // TODO: should this be moved to `start_move`?
        rng: &mut dyn RngCore,
    ) -> Option<(u8, u8)> {
        self.tree_state.set_player(player);

        const MAX_MOVES: usize = 50000;
        const MAX_CASCADES: usize = 10000;

        let mut total_moves = 0;
        let mut total_cascades = 0;
        while total_moves < MAX_MOVES && total_cascades < MAX_CASCADES {
            match self.tree_state.eval_next(
                |grid, cur_turn, me| {
                    if let Some(grid) = grid {
                        grid.score_for_player(me)
                    } else if cur_turn != me {
                        i32::MIN
                    } else {
                        i32::MAX
                    }
                },
                self.num_players as usize + 1,
            ) {
                EvalStatus::Cascaded => {
                    total_cascades += 1;
                }
                EvalStatus::Uneventful => {}
                EvalStatus::Done => {
                    // figure out which move to return
                    let mut best_moves = Vec::new();
                    let mut best_score = i32::MIN;
                    for (m, score) in self.tree_state.iter_moves_and_score() {
                        if score >= best_score {
                            if score > best_score {
                                best_moves.clear();
                                best_score = score;
                            }
                            best_moves.push(m);
                        }
                    }
                    return Some(best_moves[rng.random_range(0..best_moves.len())]);
                }
            }
            total_moves += 1;
        }

        None
    }
}

impl<Fallback: Ai + Default, const FALLBACK_CHANCE: u8> Ai for Hard<Fallback, FALLBACK_CHANCE> {
    fn start_move(&mut self, grid: &Grid) {
        self.decision = None;
        self.tree_state.clear();
        self.tree_state.set_grid(grid.clone());
        self.num_players = grid.player_count();
        self.fallback.start_move(grid);
    }

    fn tick(&mut self, grid: &Grid, player: u8, rng: &mut dyn RngCore) -> Option<(u8, u8)> {
        if self.decision.is_none() {
            let biased_fallback_max = (128
                - grid.score_for_player(player) * grid.width() as i32 * grid.height() as i32 / 256)
                .clamp(0, 255) as u8;
            self.decision = if rng.random_range(0..=biased_fallback_max) < FALLBACK_CHANCE {
                self.fallback.tick(grid, player, rng)
            } else {
                self.tick_inner(grid, player, rng)
            };
        }
        self.decision
    }

    fn name(&self) -> &str {
        "Hard"
    }
}

#[cfg(test)]
mod test {
    use rand::rand_core;

    use super::*;

    struct DeterministicRng<const N: usize> {
        data: [u32; N],
        i: usize,
    }

    impl<const N: usize> DeterministicRng<N> {
        pub fn new(data: [u32; N]) -> Self {
            Self { data, i: 0 }
        }
    }

    impl<const N: usize> RngCore for DeterministicRng<N> {
        fn next_u32(&mut self) -> u32 {
            let result = self.data[self.i];
            self.i += 1;
            if self.i >= self.data.len() {
                self.i = 0;
            }
            result
        }

        fn next_u64(&mut self) -> u64 {
            ((self.next_u32() as u64) << 32) + self.next_u32() as u64
        }

        fn fill_bytes(&mut self, dst: &mut [u8]) {
            rand_core::impls::fill_bytes_via_next(self, dst);
        }
    }

    #[test]
    fn basic_hard_test() {
        // This is a "don't be dumb" test for the Hard AI. It exists because the Hard AI was, in fact, dumb.
        let mut ai: Hard<Medium<0>, 0> = Hard::default();
        // On a 2x2 board, the only correct move for player 2 is the opposite corner as player 1.
        // As the Hard AI is supposed to have lookahead, this shouldn't be difficult.
        for y in [0, 1] {
            for x in [0, 1] {
                let mut grid = Grid::new(2, 2, 2);
                grid.init_capacity();
                let grid = grid.with_move(x, y, 1).0.unwrap();
                ai.start_move(&grid);
                let result = ai.tick_inner(&grid, 2, &mut DeterministicRng::new([0]));
                assert_eq!(result, Some((1 - x, 1 - y)));
            }
        }
    }
}
