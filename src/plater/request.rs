use std::collections::HashMap;
use std::f64::consts::PI;

use itertools::Itertools;
use log::info;
use rand::prelude::SliceRandom;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use thiserror::Error;

use crate::plater::part::Part;
use crate::plater::placer::{N, Placer, SortMode};
use crate::plater::placer::SortMode::{Shuffle, SurfaceDec};
use crate::plater::plate_shape::{PlateShape, Shape};
use crate::plater::recommender::{Recommender, Suggestion};
use crate::plater::solution::Solution;
use crate::stl;

// DEFAULT_RESOLUTION is the default bitmap resolution, in pixels per mm.
pub const DEFAULT_RESOLUTION: f64 = 1000.0;

pub struct Request {
    // plate_shape represents the size and shape of the build plate.
    pub(crate) plate_shape: Box<dyn PlateShape>,
    // single_plate_mode uses a single, expandable plate
    pub(crate) single_plate_mode: bool,
    // sort_modes is a list of sort modes to attempt when placing.
    pub(crate) sort_modes: Vec<SortMode>,
    // max_threads is the maximum number of goroutines to use when placing.
    // Set this to 0 or a negative value for no limit.
    pub(crate) max_threads: usize,
    pub(crate) precision: f64,
    // precision
    pub(crate) spacing: f64, // part spacing

    // brute-force deltas
    pub(crate) delta: f64,
    pub(crate) delta_r: f64,

    // Parts to place (TODO: revise, can this become vec)
    pub(crate) parts: HashMap<String, Part>,
    resolution: f64,
    // internal resolution (pixels per mm)
    pub(crate) algorithm: Algorithm,

    pub(crate) center_x: f64,
    pub(crate) center_y: f64,
}

#[derive(Clone)]
pub enum ThreadingMode {
    SingleThreaded,
    MultiThreaded,
}

#[derive(Clone)]
pub enum Strategy {
    PixelPack,
    SpiralPlace,
}

#[derive(Clone)]
pub enum ConfigOrder {
    PointFirst,
    RotationFirst,
}

#[derive(Clone)]
pub enum PointEnumerationMode {
    Row,
    Spiral,
}

#[derive(Clone)]
pub enum BedExpansionMode {
    Linear,
    Exponential,
}

#[derive(Error, Debug)]
pub enum PlacingError {
    #[error("No solutions found")]
    NoSolutionFound,
}

#[derive(Clone)]
pub struct Algorithm {
    pub threading_mode: ThreadingMode,
    pub strategy: Strategy,
    pub order_config: ConfigOrder,
    pub point_enumeration_mode: PointEnumerationMode,
    pub bed_expansion_mode: BedExpansionMode,
}

pub fn default_sort_modes() -> Vec<SortMode> {
    let mut modes = Vec::with_capacity(1001);
    modes.push(SurfaceDec);

    for i in 0..1000 {
        modes.push(Shuffle(i))
    }

    let xs: &mut [SortMode] = &mut (modes.as_mut_slice())[1..];

    let mut rng = rand::thread_rng();
    xs.shuffle(&mut rng);

    modes
}

impl Request {
    pub fn new(
        plate_shape: Shape,
        resolution: f64,
        algorithm: Algorithm,
        center_x: f64,
        center_y: f64,
    ) -> Self {
        let boxed_plate_shape: Box<dyn PlateShape> = match plate_shape {
            Shape::Rectangle(r) => Box::new(r),
            Shape::Circle(c) => Box::new(c),
        };

        Request {
            plate_shape: boxed_plate_shape,
            single_plate_mode: true,
            sort_modes: default_sort_modes(),
            max_threads: 1,
            precision: 0.5 * resolution,
            spacing: 1.5 * resolution,
            delta: 1.0 * resolution,
            delta_r: PI / 2.0,
            parts: Default::default(),
            resolution,
            algorithm,
            center_x: center_x * resolution,
            center_y: center_y * resolution,
        }
    }

    pub fn set_spacing(&mut self, spacing: f64) {
        self.spacing = spacing * self.resolution;
    }

    pub fn set_delta(&mut self, delta: f64) {
        self.delta = delta * self.resolution;
    }

    pub fn set_delta_r(&mut self, rotation_interval: f64) {
        self.delta_r = stl::util::deg_to_rad(rotation_interval as f64);
    }

    pub fn set_precision(&mut self, precision: f64) {
        self.precision = precision * self.resolution;
    }

    pub fn set_sort_modes(&mut self, sort_modes: Vec<SortMode>) {
        self.sort_modes = sort_modes;
    }

    pub fn set_max_threads(&mut self, max_threads: usize) {
        self.max_threads = max_threads;
    }

    pub fn get_spacing(&self) -> f64 {
        self.spacing
    }

    pub fn get_precision(&self) -> f64 {
        self.precision
    }

    pub fn get_delta_r(&self) -> f64 {
        self.delta_r
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

    pub fn process<T>(
        &self,
        on_solution_found: impl Fn(&Solution) -> T,
    ) -> Result<T, PlacingError> {
        let strategy = match self.algorithm.strategy {
            Strategy::PixelPack => Request::pixelpack,
            Strategy::SpiralPlace => Request::spiral_place,
        };

        strategy(&self, on_solution_found)
    }

    // Replace with explicit error handling
    pub fn pixelpack<T>(
        &self,
        on_solution_found: impl Fn(&Solution) -> T,
    ) -> Result<T, PlacingError> {
        let mut placer = Placer::new(self);
        placer.sort_parts(SurfaceDec);

        let mut placers = default_sort_modes()
            .into_iter()
            .map(|mode| {
                let mut p = Placer::new(self);
                p.sort_parts(mode);
                p
            })
            .collect_vec();

        let place = match &self.algorithm.threading_mode {
            ThreadingMode::SingleThreaded => Request::place_all_single_threaded,
            ThreadingMode::MultiThreaded => Request::place_all_multi_threaded,
        };

        let solutions = place(&mut placers);

        let solution = solutions.get(0).ok_or(PlacingError::NoSolutionFound)?;

        Ok(on_solution_found(solution))
    }

    // Replace with explicit error handling
    pub fn spiral_place<T>(
        &self,
        on_solution_found: impl Fn(&Solution) -> T,
    ) -> Result<T, PlacingError> {
        let mut placers = vec![];
        let sort_modes = Vec::clone(&self.sort_modes);

        for sort_mode in sort_modes {
            for rotate_offset in 0..2 {
                for rotate_direction in 0..2 {
                    let mut placer = Placer::new(self);
                    placer.sort_parts(sort_mode);
                    placer.set_rotate_direction(rotate_direction);
                    placer.set_rotate_offset(rotate_offset);
                    placers.push(placer)
                }
            }
        }
        // placers.shuffle(&mut thread_rng());

        let mut subset = { placers };

        let place_all_placers = match self.algorithm.threading_mode {
            ThreadingMode::SingleThreaded => Request::place_all_single_threaded,
            ThreadingMode::MultiThreaded => Request::place_all_multi_threaded,
        };

        let mut solutions = place_all_placers(&mut subset);
        solutions.sort_by(|x, y| f64::partial_cmp(&x.plate_area(), &y.plate_area()).unwrap());

        let solution = solutions.get(0).ok_or(PlacingError::NoSolutionFound)?;

        Ok(on_solution_found(solution))
    }

    fn place_all_single_threaded<'a>(placers: &'a mut [Placer<'a>]) -> Vec<Solution<'a>> {
        let mut k = None;
        let max_duration = instant::Duration::from_secs(10);
        let mut rec = Recommender::new(max_duration, placers.len());
        let rec = &mut rec;

        info!("Starting");
        placers
            .into_iter()
            .filter_map(|placer| {
                if let Some(x) = k.clone() {
                    if x <= N {
                        return None;
                    }
                }

                match rec.observe(k.clone()) {
                    Suggestion::Stop => {
                        return None;
                    }
                    Suggestion::Continue => {}
                }

                placer.smallest_observed_plate = k.clone();
                info!("Smallest plate {:#?}", k);
                let mut cur = placer.place();

                // Update the best solution if we found something better
                if let Some(d) = &mut cur {
                    k = Option::clone(&d.best_so_far);
                }
                cur
            })
            .collect::<Vec<_>>()
    }

    fn place_all_multi_threaded<'a>(placers: &'a mut [Placer<'a>]) -> Vec<Solution<'a>> {
        placers
            .into_par_iter()
            .map(|placer| placer.place().unwrap())
            .collect::<Vec<_>>()
    }
}
