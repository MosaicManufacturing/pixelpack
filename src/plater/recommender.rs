use std::collections::HashMap;

use instant;

use crate::plater::recommender::Suggestion::{Continue, Stop};

pub(crate) enum Suggestion {
    Stop,
    Continue,
}

pub struct Recommender {
    map: HashMap<Option<usize>, usize>,
    start: instant::Instant,
    max_duration: instant::Duration,
    attempts: usize,
    observation_count: usize,
}

impl Recommender {
    pub(crate) fn new(max_duration: instant::Duration, attempts: usize) -> Self {
        let start = instant::Instant::now();
        Recommender {
            map: HashMap::new(),
            start,
            max_duration,
            attempts,
            observation_count: 0,
        }
    }

    pub(crate) fn observe(&mut self, value: Option<usize>) -> Suggestion {
        self.attempts += 1;
        let now = instant::Instant::now();

        if instant::Instant::saturating_duration_since(&self.start, now) > self.max_duration {
            return Stop;
        }

        match self.map.get(&value) {
            None => {
                self.map.insert(value.clone(), 1);
            }
            Some(_) => {
                self.map.get_mut(&value).map(|x| *x = *x + 1);
            }
        }

        let v = *self.map.get(&value).unwrap();

        let cut: usize = match value {
            None => 10,
            Some(_) => 5,
        };

        // At least 50% of total placers should be attempted
        if (self.attempts as f64) / (self.observation_count as f64) < 0.5 {
            return Continue;
        }

        if v < cut {
            Continue
        } else {
            Stop
        }
    }
}
