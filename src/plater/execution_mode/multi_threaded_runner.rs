use std::time::Duration;

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::plater::placer::Placer;
use crate::plater::progress_config::{ProgressConfig, ProgressMessage};
use crate::plater::request::{PlacingError, Request};
use crate::plater::solution::{get_smallest_solution, Solution};

pub struct MultiThreadedRunner<'r> {
    request: &'r Request,
}

fn place_all_multi_threaded<'request, F2: Fn(ProgressMessage)>(
    placers: &mut [Placer<'request>],
    timeout: Option<Duration>,
    config: ProgressConfig<F2>,
) -> Vec<Solution<'request>> {
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
    pub fn place<F2: Fn(ProgressMessage)>(
        &self,
        config: ProgressConfig<F2>,
    ) -> Result<Solution<'r>, PlacingError> {
        let mut placers: Vec<Placer<'r>> = self.request.get_placers_for_spiral_place();
        let mut solutions: Vec<Solution<'r>> =
            place_all_multi_threaded(&mut placers, self.request.timeout.clone(), config);

        get_smallest_solution(&mut solutions)
    }
}
