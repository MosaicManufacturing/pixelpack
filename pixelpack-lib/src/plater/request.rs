use std::collections::HashMap;
use std::f64::consts::PI;

use log::info;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::plater::part::Part;
use crate::plater::placer::{GRAVITY_MODE_LIST, Placer, SortMode};
use crate::plater::placer::SortMode::{Shuffle, SurfaceDec, SurfaceInc};
use crate::plater::plate_shape::PlateShape;
use crate::plater::solution::Solution;

// DEFAULT_RESOLUTION is the default bitmap resolution, in pixels per mm.
pub const DEFAULT_RESOLUTION: f64 = 1000.0;

pub struct Request<'a, Shape: PlateShape> {
    // plate_shape represents the size and shape of the build plate.
    pub(crate) plate_shape: &'a Shape,
    // single_plate_mode uses a single, expandable plate
    pub(crate) single_plate_mode: bool,
    // sort_modes is a list of sort modes to attempt when placing.
    pub sort_modes: Vec<SortMode>,
    // max_threads is the maximum number of goroutines to use when placing.
    // Set this to 0 or a negative value for no limit.
    pub max_threads: usize,
    pub precision: f64,
    // precision
    pub spacing: f64, // part spacing

    // brute-force deltas
    pub delta: f64,
    pub delta_r: f64,

    // Parts to place (TODO: revice, can this become vec)
    pub(crate) parts: HashMap<String, Part>,
    resolution: f64, // internal resolution (pixels per mm)
}

// TODO: Don't understand the original version
pub fn default_sort_modes() -> Vec<SortMode> {
    // let random_shuffles: usize = 3;
    // let sort_shuffle_as_usize: usize = SortShuffle.into();
    // let last_sort: usize = sort_shuffle_as_usize + random_shuffles - 1;
    vec![SurfaceDec, SurfaceInc, Shuffle]
}

impl<'a, Shape: PlateShape> Request<'a, Shape> {
    pub fn new(plate_shape: &'a Shape, resolution: f64) -> Self {
        Request {
            plate_shape,
            single_plate_mode: true,
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
    pub fn add_part(&mut self, part: Part) -> Option<()> {
        let x = self.parts.get(part.id.as_str());
        if x.is_some() {
            return None;
        }

        let part_id = part.id.clone();
        self.parts.insert(part_id, part);
        Some(())
    }

    // Replace with explicit error handling
    pub fn process<T>(&'a self, on_solution_found: impl Fn(&Solution) -> T) -> T {
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
            .map(|placer| {
                info!("Starting");
                placer.place()
            })
            .collect::<Vec<_>>();

        solutions.sort_by(|x, y| f64::partial_cmp(&x.score(), &y.score()).unwrap());

        on_solution_found(&solutions[0])
    }
}
