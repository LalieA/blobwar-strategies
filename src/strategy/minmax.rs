//! Implementation of the min max algorithm.
use rayon::prelude::*;
use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;
use std::fmt;


/// Min-Max algorithm with a given recursion depth.
pub struct MinMax(pub u8);

impl Strategy for MinMax {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        fn minmax_movement_value(state: &Configuration, depth: u8, maximizing_player: bool) -> i8 {
            if depth == 0 || state.movements().next().is_none() {
                return if maximizing_player { state.skip_play().value() } else { state.value() }
            }

            return state.movements()
                .map(|m| -minmax_movement_value(&state.play(&m), depth - 1, maximizing_player))
                .max()
                .unwrap();
        }
        return state.movements()
            .par_bridge()
            .map(|m| (m, -minmax_movement_value(&state.play(&m), self.0 - 1, self.0 % 2 == 1)))
            .min_by_key(|(_, v)| *v)
            .map(|(m, _)| m);
    }
}

impl fmt::Display for MinMax {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Min - Max (max level: {})", self.0)
    }
}

/// Anytime min max algorithm.
/// Any time algorithms will compute until a deadline is hit and the process is killed.
/// They are therefore run in another process and communicate through shared memory.
/// This function is intended to be called from blobwar_iterative_deepening.
pub fn min_max_anytime(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    for depth in 1..100 {
        movement.store(MinMax(depth).compute_next_move(state));
    }
}
