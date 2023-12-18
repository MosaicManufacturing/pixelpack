use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use crate::plater::placer::{Placer, N};
use crate::plater::progress_config::{ProgressConfig, ProgressMessage};
use crate::plater::recommender::{Recommender, Suggestion};
use crate::plater::request::{PlacingError, Request};
use crate::plater::solution::{get_smallest_solution, Solution};

pub struct AsyncJsRunner<'r, F: Future> {
    request: &'r Request,
    cancellation_future: Pin<Box<F>>,
}

async fn place_async<'a, T, F1: Fn(&Solution) -> T, F2: Fn(ProgressMessage), F3: Future + Unpin>(
    placers: &'a mut [Placer<'a>],
    timeout: Option<Duration>,
    config: &mut ProgressConfig<T, F1, F2>,
    mut cancellation_future: F3,
) -> Vec<Solution<'a>> {
    let mut smallest_plate_index = None;
    // TODO: fix duration issue
    let max_duration = Duration::MAX;
    let mut rec = Recommender::new(max_duration, placers.len());
    let rec = &mut rec;

    let total = placers.len();

    config.on_prog(|| ProgressMessage::PreRun {
        total_placers: total
            .try_into()
            .expect("Could not represent placement count as u32"),
    });

    let mut results = vec![];
    'placing_loop: for (index, placer) in placers.iter_mut().enumerate() {
        config.on_prog(|| ProgressMessage::Placement {
            placer_index: index as u32,
            percentage: index as f64 * 100.00 / total as f64,
            total_placers: total as u32,
        });
        // This line is required to periodically yield to the executor for fairer scheduling
        let mut timer = gloo_timers::future::sleep(Duration::from_millis(0));
        match futures::future::select(&mut cancellation_future, &mut timer).await {
            futures::future::Either::Left(_) => break 'placing_loop,
            _ => {}
        }

        if let Some(plate_index) = smallest_plate_index.clone() {
            if plate_index <= N {
                break 'placing_loop;
            }
        }

        match rec.observe(smallest_plate_index.clone()) {
            Suggestion::Stop => break 'placing_loop,
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

impl<'r, F: Future> AsyncJsRunner<'r, F> {
    pub fn new(request: &'r Request, cancellation_future: F) -> Self {
        AsyncJsRunner {
            request,
            cancellation_future: Box::pin(cancellation_future),
        }
    }
    pub async fn place<T, F1: Fn(&Solution) -> T, F2: Fn(ProgressMessage)>(
        &mut self,
        mut config: ProgressConfig<T, F1, F2>,
    ) -> Result<T, PlacingError> {
        let mut placers = self.request.get_placers_for_spiral_place();
        let solutions = place_async(
            &mut placers,
            self.request.timeout.clone(),
            &mut config,
            &mut self.cancellation_future,
        )
        .await;

        let solution = get_smallest_solution(&solutions).ok_or(PlacingError::NoSolutionFound)?;

        Ok(config.on_sol(solution))
    }
}
