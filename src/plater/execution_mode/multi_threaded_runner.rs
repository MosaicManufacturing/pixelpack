use std::time::Duration;

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::plater::placer::Placer;
use crate::plater::progress_config::ProgressConfig;
use crate::plater::request::{PlacingError, Request};
use crate::plater::solution::Solution;

pub struct MultiThreadedRunner<'r> {
    request: &'r Request,
}

fn place_all_multi_threaded<'a, T, F1: Fn(&Solution) -> T, F2: Fn(&str)>(
    placers: &'a mut [Placer<'a>],
    timeout: Option<Duration>,
    config: &ProgressConfig<T, F1, F2>,
) -> Vec<Solution<'a>> {
    let start = &instant::Instant::now();
    let timeout = &timeout;

    placers
        .into_par_iter()
        .filter_map(|placer| {
            if let Some(limit) = timeout {
                let now = instant::Instant::now();
                if now.saturating_duration_since(start.clone()) > *limit {
                    return None;
                }
            }

            placer.place()
        })
        .collect::<Vec<_>>()
}

impl<'r> MultiThreadedRunner<'r> {
    pub fn new(request: &'r Request) -> Self {
        MultiThreadedRunner { request }
    }
    pub fn place<T, F1: Fn(&Solution) -> T, F2: Fn(&str)>(
        &self,
        mut config: ProgressConfig<T, F1, F2>,
    ) -> Result<T, PlacingError> {
        let mut placers = self.request.get_placers_for_spiral_place();
        let solutions =
            place_all_multi_threaded(&mut placers, self.request.timeout.clone(), &config);

        let solution = solutions.get(0).ok_or(PlacingError::NoSolutionFound)?;

        Ok(config.on_sol(solution))
    }
}
