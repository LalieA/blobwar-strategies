//! Alpha - Beta algorithm.
use rayon::prelude::*;
use std::fmt;

use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;

/// Anytime alpha beta algorithm.
/// Any time algorithms will compute until a deadline is hit and the process is killed.
/// They are therefore run in another process and communicate through shared memory.
/// This function is intended to be called from blobwar_iterative_deepening.
pub fn alpha_beta_anytime(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    for depth in 1..100 {
        let chosen_movement = AlphaBeta(depth).compute_next_move(state);
        movement.store(chosen_movement);
    }
}

/// Alpha - Beta algorithm with given maximum number of recursions.
pub struct AlphaBeta(pub u8);

impl fmt::Display for AlphaBeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Alpha - Beta (max level: {})", self.0)
    }
}

impl Strategy for AlphaBeta {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        fn alphabeta_movement_value(state: &Configuration, depth: u8, maximizing_player: bool, i: i8) -> i8 {
            if depth == 0 || state.movements().next().is_none() {
                return if maximizing_player { state.skip_play().value() } else { state.value() }
            }

            let mut j = i8::MIN + 1;

            return state.movements()
                .find_map(|m| {
                    j = std::cmp::max(j, alphabeta_movement_value(&state.play(&m), depth - 1, maximizing_player, j));
                    if -j <= i {
                        return Some(-j);
                    }
                    return None;
                })
                .unwrap_or(-j);
        }
        return state.movements()
            .par_bridge()
            .map(|m| (m, -alphabeta_movement_value(&state.play(&m), self.0 - 1, false, i8::MIN + 1)))
            .min_by_key(|(_, v)| *v)
            .map(|(m, _)| m);
    }
}