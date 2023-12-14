use crate::plater::placer::{Placer, N};
use crate::plater::progress_config::{FutureKillSwitch, ProgressConfig};
use crate::plater::recommender::{Recommender, Suggestion};
use crate::plater::request::{PlacingError, Request};
use crate::plater::solution::Solution;
use std::time::Duration;

pub struct SingleThreadedRunner<'r> {
    request: &'r Request,
}

fn place_all_single_threaded<'a, T, F1: Fn(&Solution) -> T, F2: Fn(&str)>(
    placers: &'a mut [Placer<'a>],
    timeout: Option<Duration>,
    config: &ProgressConfig<T, F1, F2>,
) -> Vec<Solution<'a>> {
    config.on_prog(|| format!("Starting, total placers: {}", placers.len()));

    let mut smallest_plate_index = None;
    let max_duration = timeout.unwrap_or_else(|| Duration::from_secs(10));
    let mut rec = Recommender::new(max_duration, placers.len());
    let rec = &mut rec;

    let mut results = vec![];
    for (index, placer) in placers.iter_mut().enumerate() {
        config.on_prog(|| format!("Working on placer # {}", index));

        if let Some(plate_index) = smallest_plate_index.clone() {
            if plate_index <= N {
                break;
            }
        }

        match rec.observe(smallest_plate_index.clone()) {
            Suggestion::Stop => {
                break;
            }
            Suggestion::Continue => {}
        }

        placer.smallest_observed_plate = smallest_plate_index.clone();

        // Update the best solution if we found something better
        if let Some(solution) = placer.place() {
            smallest_plate_index = Option::clone(&solution.best_so_far);
            results.push(solution)
        }
    }

    results
}

impl<'r> SingleThreadedRunner<'r> {
    pub fn new(request: &'r Request) -> Self {
        SingleThreadedRunner { request }
    }
    pub fn place<T, F1: Fn(&Solution) -> T, F2: Fn(&str)>(
        &self,
        mut config: ProgressConfig<T, F1, F2>,
    ) -> Result<T, PlacingError> {
        let mut placers = self.request.get_placers_for_spiral_place();
        let solutions =
            place_all_single_threaded(&mut placers, self.request.timeout.clone(), &config);

        let solution = solutions.get(0).ok_or(PlacingError::NoSolutionFound)?;

        Ok(config.on_sol(solution))
    }
}
