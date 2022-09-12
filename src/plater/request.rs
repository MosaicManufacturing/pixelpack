use std::collections::HashMap;
use std::f64::consts::PI;

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::plater::part::Part;
use crate::plater::placer::{GRAVITY_MODE_LIST, Placer, SortMode};
use crate::plater::placer::SortMode::{SortShuffle, SortSurfaceDec, SortSurfaceInc};
use crate::plater::plate_shape::PlateShape;
use crate::plater::solution::Solution;

// DEFAULT_RESOLUTION is the default bitmap resolution, in pixels per mm.
const DEFAULT_RESOLUTION: f64 = 1000.0;

pub struct Request<'a, Shape: PlateShape> {
    // plate_shape represents the size and shape of the build plate.
    pub(crate) plate_shape: &'a Shape,
    // single_plate_mode uses a single, expandable plate
    pub(crate) single_plate_mode: bool,
    // sort_modes is a list of sort modes to attempt when placing.
    sort_modes: Vec<SortMode>,
    // max_threads is the maximum number of goroutines to use when placing.
    // Set this to 0 or a negative value for no limit.
    max_threads: usize,
    pub(crate) precision: f64,
    // precision
    spacing: f64, // part spacing

    // brute-force deltas
    pub(crate) delta: f64,
    pub(crate) delta_r: f64,

    // Parts to place
    pub(crate) parts: HashMap<String, &'a Part>,
    resolution: f64, // internal resolution (pixels per mm)
}

// TODO: Don't understand the original version
fn default_sort_modes() -> Vec<SortMode> {
    // let random_shuffles: usize = 3;
    // let sort_shuffle_as_usize: usize = SortShuffle.into();
    // let last_sort: usize = sort_shuffle_as_usize + random_shuffles - 1;
    vec![SortSurfaceDec, SortSurfaceInc, SortShuffle]
}

impl<'a, Shape: PlateShape> Request<'a, Shape> {
    pub(crate) fn new(plate_shape: &'a Shape, resolution: f64) -> Self {
        Request {
            plate_shape,
            single_plate_mode: false,
            sort_modes: default_sort_modes(),
            max_threads: 1,
            precision: 0.5 * resolution,
            spacing: 1.5 * resolution,
            delta: 1.0 * resolution,
            delta_r: PI / 2.0,
            parts: Default::default(),
            resolution,
        }
    }

    // TODO: replace option with explicit error handling (this is weird)
    fn add_part(&mut self, part: &'a Part) -> Option<()> {
        let x = self.parts.get(part.id.as_str());
        if x.is_some() {
            return None;
        }

        let part_id = part.id.clone();
        self.parts.insert(part_id, part);
        Some(())
    }

    // Replace with explicit error handling
    fn process(&'a self) -> Option<Solution<'a>> {
        let mut placers = vec![];
        let sort_modes = Vec::clone(&self.sort_modes);

        for sort_mode in sort_modes {
            for rotate_offset in 0..2 {
                for rotate_direction in 0..2 {
                    for gravity_mode in GRAVITY_MODE_LIST {
                        let mut placer = Placer::new(self);
                        placer.sort_parts(sort_mode);
                        placer.set_gravity_mode(gravity_mode);
                        placer.set_rotate_direction(rotate_direction);
                        placer.set_rotate_offset(rotate_offset);
                        placers.push(placer)
                    }
                }
            }
        }

        let mut solutions = (&mut placers)
            .into_par_iter()
            .map(Placer::place)
            .collect::<Vec<_>>();

        solutions.sort_by(|x, y| f64::partial_cmp(&x.score(), &y.score()).unwrap());

        let best_solution = &solutions[0];
        None
    }
}
