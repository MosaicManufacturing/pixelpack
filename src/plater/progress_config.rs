use std::future::Future;
use std::pin::Pin;

use crate::plater::solution::Solution;

pub trait FutureKillSwitch: Future {}

impl<T: Future> FutureKillSwitch for T {}

pub struct ProgressConfig<T, F1: Fn(&Solution) -> T, F2: Fn(&str), F3: Future> {
    on_solution_found: F1,
    on_progress: F2,
    cancellation_future: Pin<Box<F3>>,
}

impl<T, F1: Fn(&Solution) -> T, F2: Fn(&str), F3: Future> ProgressConfig<T, F1, F2, F3> {
    pub fn new(f1: F1, f2: F2, f3: F3) -> Self {
        ProgressConfig {
            on_solution_found: f1,
            on_progress: f2,
            cancellation_future: Box::pin(f3),
        }
    }

    pub fn get_cancellation_future_mut(&mut self) -> &mut Pin<Box<F3>> {
        &mut self.cancellation_future
    }

    pub fn on_sol(&mut self, solution: &Solution) -> T {
        (self.on_solution_found)(solution)
    }

    pub fn on_prog<F: Fn() -> String>(&self, f: F) {
        (self.on_progress)(&f());
    }
}
