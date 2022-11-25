use std::collections::HashMap;
use std::f64::consts::PI;

use log::info;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::plater::part::Part;
use crate::plater::placer::SortMode::{Shuffle, SurfaceDec, SurfaceInc};
use crate::plater::placer::{Placer, SortMode, GRAVITY_MODE_LIST};
use crate::plater::plate_shape::PlateShape;
use crate::plater::solution::Solution;

// DEFAULT_RESOLUTION is the default bitmap resolution, in pixels per mm.
pub const DEFAULT_RESOLUTION: f64 = 1000.0;

pub struct Request<S: PlateShape> {
    // plate_shape represents the size and shape of the build plate.
    pub(crate) plate_shape: S,
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
    pub(crate) algorithm: Algorithm,
}

#[derive(Clone)]
pub enum ThreadingMode {
    SingleThreaded,
    MultiThreaded,
}

#[derive(Clone)]
pub enum Strategy {
    PixelPack,
    SpiralPlace
}

#[derive(Clone)]
pub enum ConfigOrder {
    PointFirst,
    RotationFirst
}

#[derive(Clone)]
pub enum PointEnumerationMode {
    Row,
    Spiral
}

#[derive(Clone)]
pub enum BedExpansionMode {
    Linear,
    Exponential
}

#[derive(Clone)]
pub struct Algorithm {
    pub threading_mode: ThreadingMode,
    pub strategy: Strategy,
    pub order_config: ConfigOrder,
    pub point_enumeration_mode: PointEnumerationMode,
    pub bed_expansion_mode: BedExpansionMode
}

// TODO: Don't understand the original version
pub fn default_sort_modes() -> Vec<SortMode> {
    // let random_shuffles: usize = 3;
    // let sort_shuffle_as_usize: usize = SortShuffle.into();
    // let last_sort: usize = sort_shuffle_as_usize + random_shuffles - 1;

    // TODO: too many shuffles dynamically use shuffles if all existing solutions do not fit on the plate
    vec![SurfaceDec, SurfaceInc, Shuffle, Shuffle, Shuffle, Shuffle, Shuffle]
}

impl<S: PlateShape> Request<S> {
    pub fn new(plate_shape: S, resolution: f64, algorithm: Algorithm) -> Self {
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
            algorithm
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

    pub fn process<T>(&self, on_solution_found: impl Fn(&Solution) -> T) -> T {
        let strategy = match self.algorithm.strategy {
            Strategy::PixelPack => Request::pixelpack,
            Strategy::SpiralPlace => Request::spiral_place
        };

        strategy(&self, on_solution_found)
    }

    // Replace with explicit error handling
    pub fn spiral_place<T>(&self, on_solution_found: impl Fn(&Solution) -> T) -> T {
        let mut placer = Placer::new(self);
        placer.sort_parts(SortMode::SurfaceDec);


        for part in &placer.unlocked_parts {
            info!("{} {}", part.get_id(), part.get_surface());
        }


        let mut placers = vec![placer];

        let place =  match &self.algorithm.threading_mode {
            ThreadingMode::SingleThreaded => Request::place_all_single_threaded,
            ThreadingMode::MultiThreaded => Request::place_all_multi_threaded,
        };


        let solution = place(&mut placers);

        ;
        on_solution_found(&solution[0])
    }


    // Replace with explicit error handling
    pub fn pixelpack<T>(&self, on_solution_found: impl Fn(&Solution) -> T) -> T {
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

        let place_all_placers = match self.algorithm.threading_mode {
            ThreadingMode::SingleThreaded => Request::place_all_single_threaded,
            ThreadingMode::MultiThreaded => Request::place_all_multi_threaded,
        };

        let mut solutions = place_all_placers(&mut placers);

        info!("Solutions lenght: {}", solutions.len());
        let last = solutions.pop().unwrap();

        solutions.sort_by(|x, y| f64::partial_cmp(&x.plate_area(), &y.plate_area()).unwrap());
        solutions.iter().for_each(|s| {
            let (x, y) = s.dims();
            info!("Width: {} Height:{}  area: {}", x, y, s.plate_area())
        });
        on_solution_found(&solutions[0])
    }

    fn place_all_single_threaded<'a>(placers: &'a mut [Placer<'a, S>]) -> Vec<Solution<'a>> {
        info!("Starting single threaded place new");
        let mut k = None;
        placers
            .into_iter()
            .filter_map(|placer| {
                placer.smallest_observed_plate = k.clone();

                // info!("Updated k, Starting n");
                let mut cur = placer.place();

                // Update the best solution if we found something better
                if let Some(d) = &mut cur {
                    k = Option::clone(&d.best_so_far);
                    // info!("Updated k, k is {:#?}, width, height: {:#?}",k, d.dims());
                }
                cur
            })
            .collect::<Vec<_>>()
    }

    fn place_all_multi_threaded<'a>(placers: &'a mut [Placer<'a, S>]) -> Vec<Solution<'a>> {
        info!("Starting multi threaded place");
        placers
            .into_par_iter()
            .map(|placer| {
                info!("Starting");
                placer.place().unwrap()
            })
            .collect::<Vec<_>>()
    }
}
