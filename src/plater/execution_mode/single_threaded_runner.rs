use std::time::Duration;

use crate::plater::placer::{Placer, N};
use crate::plater::progress::{ProgressMessage, ProgressMessenger};
use crate::plater::recommender::{Recommender, Suggestion};
use crate::plater::request::{PlacingError, Request};
use crate::plater::solution::{get_smallest_solution, Solution};

pub struct SingleThreadedRunner<'r> {
    request: &'r Request,
}

fn place_all_single_threaded<'request, F2: Fn(ProgressMessage)>(
    placers: &mut [Placer<'request>],
    timeout: Option<Duration>,
    messenger: ProgressMessenger<F2>,
) -> Vec<Solution<'request>> {
    let mut smallest_plate_index = None;
    let max_duration = timeout.unwrap_or_else(|| Duration::from_secs(10));
    let mut rec = Recommender::new(max_duration, placers.len());
    let rec = &mut rec;

    let mut results = vec![];
    for (index, placer) in placers.iter_mut().enumerate() {
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
    pub fn place<F2: Fn(ProgressMessage)>(
        &self,
        messenger: ProgressMessenger<F2>,
    ) -> Result<Solution<'r>, PlacingError> {
        let mut placers = self.request.get_placers_for_spiral_place();
        let mut solutions =
            place_all_single_threaded(&mut placers, self.request.timeout.clone(), messenger);

        get_smallest_solution(&mut solutions)
    }
}
