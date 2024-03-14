use std::future::Future;
use std::time::Duration;

use crate::plater::placer::{Placer, N};
use crate::plater::progress_config::{ProgressMessage, ProgressMessenger};
use crate::plater::recommender::{Recommender, Suggestion};
use crate::plater::request::{PlacingError, Request};
use crate::plater::solution::{get_smallest_solution, Solution};

pub struct AsyncJsRunner<'r> {
    request: &'r Request,
}

async fn place_async<'request_part, F2: Fn(ProgressMessage), F3: Future + Unpin>(
    placers: &mut [Placer<'request_part>],
    timeout: Option<Duration>,
    messenger: ProgressMessenger<F2>,
    mut cancellation_future: F3,
) -> Vec<Solution<'request_part>> {
    let mut smallest_plate_index = None;
    let max_duration = Duration::MAX;
    let mut rec = Recommender::new(max_duration, placers.len());
    let rec = &mut rec;

    let total = placers.len();

    messenger.send_message(|| ProgressMessage::PreRun {
        total_placers: total
            .try_into()
            .expect("Could not represent placement count as u32"),
    });

    let mut results = vec![];
    'placing_loop: for (index, placer) in placers.iter_mut().enumerate() {
        messenger.send_message(|| ProgressMessage::Placement {
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
            messenger.send_message(|| ProgressMessage::SolutionFound {
                placer_index: index as u32,
            });

            smallest_plate_index = Option::clone(&solution.best_so_far);
            results.push(solution)
        }
    }

    results
}

impl<'r> AsyncJsRunner<'r> {
    pub fn new(request: &'r Request) -> Self {
        AsyncJsRunner { request }
    }
    pub async fn place<F: Future, F2: Fn(ProgressMessage)>(
        &mut self,
        messenger: ProgressMessenger<F2>,
        cancellation_future: F,
    ) -> Result<Solution<'r>, PlacingError> {
        let pinned_future = Box::pin(cancellation_future);
        let mut placers: Vec<Placer<'r>> = self.request.get_placers_for_spiral_place();
        let mut solutions = place_async(
            &mut placers,
            self.request.timeout.clone(),
            messenger,
            pinned_future,
        )
        .await;

        get_smallest_solution(&mut solutions)
    }
}
