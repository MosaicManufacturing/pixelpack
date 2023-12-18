use std::future::Future;

use crate::plater::solution::Solution;

pub trait FutureKillSwitch: Future {}

impl<T: Future> FutureKillSwitch for T {}

pub enum ProgressMessage {
    PreRun {
        total_placers: u32,
    },
    Placement {
        placer_index: u32,
        percentage: f64,
        total_placers: u32,
    },
    StringMessage(String),
}

pub struct ProgressConfig<T, F1: Fn(&Solution) -> T, F2: Fn(ProgressMessage)> {
    on_solution_found: F1,
    on_progress: F2,
}

impl<T, F1: Fn(&Solution) -> T, F2: Fn(ProgressMessage)> ProgressConfig<T, F1, F2> {
    pub fn new(f1: F1, f2: F2) -> Self {
        ProgressConfig {
            on_solution_found: f1,
            on_progress: f2,
        }
    }

    pub fn on_sol(&mut self, solution: &Solution) -> T {
        (self.on_solution_found)(solution)
    }

    pub fn on_prog<F: Fn() -> ProgressMessage>(&self, f: F) {
        (self.on_progress)(f());
    }
}
