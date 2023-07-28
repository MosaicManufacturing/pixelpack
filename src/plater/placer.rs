use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::vec;

use log::info;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::plater::placed_part::PlacedPart;
use crate::plater::placer::helpers::find_solution;
use crate::plater::placer::rect::Rect;
use crate::plater::placer::score::ScoreOrder;
use crate::plater::placer::search::{binary_search, exponential_search_simple, Attempts};
use crate::plater::placer::GravityMode::{GravityEQ, GravityXY, GravityYX};
use crate::plater::plate::Plate;
use crate::plater::plate_shape::PlateShape;
use crate::plater::request::{BedExpansionMode, Request};
use crate::plater::solution::Solution;

pub(crate) const N: usize = 128;

#[derive(Clone, Copy)]
pub enum SortMode {
    // SortSurfaceDec sorts parts in descending order of surface area.
    SurfaceDec,
    // SortSurfaceInc sorts parts in ascending order of surface area.
    SurfaceInc,
    // SortShuffle sorts parts in random order.
    Shuffle(usize),
    WidthDec,
    HeightDec,
}

impl From<SortMode> for usize {
    fn from(x: SortMode) -> Self {
        match x {
            SortMode::SurfaceDec => 0,
            SortMode::SurfaceInc => 1,
            SortMode::Shuffle(_) => 2,
            SortMode::WidthDec => 3,
            SortMode::HeightDec => 4,
        }
    }
}

pub enum GravityMode {
    // GravityYX gives Y score a weighting of 10 times the X score.
    GravityYX,
    // GravityXY gives X score a weighting of 10 times the Y score.
    GravityXY,
    // GravityEQ gives X and Y scores equal weighting.
    GravityEQ,
}

pub(crate) const GRAVITY_MODE_LIST: [GravityMode; 3] = [GravityYX, GravityXY, GravityEQ];

impl From<GravityMode> for usize {
    fn from(x: GravityMode) -> Self {
        match x {
            GravityYX => 0,
            GravityXY => 1,
            GravityEQ => 2,
        }
    }
}

type PlateId = usize;

#[derive(Clone)]
pub(crate) struct Placer<'a> {
    rotate_offset: i32,
    rotate_direction: i32,
    // 0 = CCW, 1 = CW, TODO: make an enum
    cache: HashMap<PlateId, HashMap<String, bool>>,

    // scoring weights
    x_coef: f64,
    y_coef: f64,
    // input data
    locked_parts: Vec<PlacedPart<'a>>,
    pub(crate) unlocked_parts: Vec<PlacedPart<'a>>,
    pub(crate) request: &'a Request,
    // center_x, center_y, width, height
    current_bounding_box: Option<Rect>,
    pub smallest_observed_plate: Option<usize>,
    pub(crate) score_order: Option<ScoreOrder>,
}

impl<'a> Placer<'a> {
    pub(crate) fn new(request: &'a Request) -> Self {
        let mut p = Placer {
            rotate_offset: 0,
            rotate_direction: 0,
            cache: Default::default(),
            x_coef: 0.0,
            y_coef: 0.0,
            locked_parts: vec![],
            unlocked_parts: vec![],
            request,
            current_bounding_box: None,
            smallest_observed_plate: None,
            score_order: None,
        };

        for part in request.parts.values() {
            let (off_x, off_y) = (
                part.center_x - part.width / 2.0,
                part.center_y - part.height / 2.0,
            );
            let mut placed_part = PlacedPart::new_placed_part(part);

            if part.locked {
                placed_part.set_offset(off_x, off_y);
                p.locked_parts.push(placed_part)
            } else {
                p.unlocked_parts.push(placed_part);
            }
        }

        p
    }

    fn reset_cache(&mut self) {
        self.cache.clear();
    }

    pub(crate) fn sort_parts(&mut self, sort_mode: SortMode) {
        match sort_mode {
            SortMode::SurfaceDec => self.unlocked_parts.sort_by(|x, y| {
                let s1 = x.get_surface();
                let s2 = y.get_surface();
                f64::partial_cmp(&s1, &s2).unwrap()
            }),
            SortMode::SurfaceInc => {
                self.unlocked_parts.sort_by(|x, y| {
                    let s1 = x.get_surface();
                    let s2 = y.get_surface();
                    f64::partial_cmp(&s2, &s1).unwrap()
                });
            }
            SortMode::Shuffle(seed) => {
                let mut rng = StdRng::seed_from_u64(seed as u64);
                self.unlocked_parts.shuffle(&mut rng)
            }
            SortMode::WidthDec => {
                self.unlocked_parts.sort_by(|x, y| {
                    let s1 = x.part.get_bitmap(0).width;
                    let s2 = y.part.get_bitmap(0).width;
                    i32::partial_cmp(&s1, &s2).unwrap()
                });
            }
            SortMode::HeightDec => {
                self.unlocked_parts.sort_by(|x, y| {
                    let s1 = x.part.get_bitmap(0).width;
                    let s2 = y.part.get_bitmap(0).width;
                    i32::partial_cmp(&s2, &s1).unwrap()
                });
            }
        }
    }

    pub(crate) fn set_gravity_mode(&mut self, gravity_mode: GravityMode) {
        let (new_x_coef, new_y_coef) = match gravity_mode {
            GravityYX => (1.0, 10.0),
            GravityXY => (10.0, 1.0),
            GravityEQ => (1.0, 1.0),
        };

        self.x_coef = new_x_coef;
        self.y_coef = new_y_coef;
    }

    pub(crate) fn set_rotate_direction(&mut self, direction: i32) {
        self.rotate_direction = direction;
    }

    pub(crate) fn set_rotate_offset(&mut self, offset: i32) {
        self.rotate_offset = offset;
    }

    pub(crate) fn set_score_order(&mut self, score_order: ScoreOrder) {
        self.score_order = Some(score_order);
    }

    fn place_single_plate_linear(&mut self) -> Option<Solution> {
        let mut shape = Clone::clone(&self.request.plate_shape);
        let mut plate = Plate::make_plate_with_placed_parts(
            shape.as_ref(),
            self.request.precision,
            &mut Vec::clone(&self.locked_parts),
            self.request.center_x,
            self.request.center_y,
        )?;

        for (i, part) in self.unlocked_parts.iter_mut().enumerate() {
            part.insertion_index = i;
        }

        let mut expansion_needed = false;
        let expand_mm = 10.0;
        while !self.unlocked_parts.is_empty() {
            if expansion_needed {
                // Expand and try again
                shape = shape.expand(expand_mm);
                plate = Plate::make_plate_with_placed_parts(
                    shape.as_ref(),
                    self.request.precision,
                    &mut Vec::clone(&self.locked_parts),
                    self.request.center_x,
                    self.request.center_y,
                )?;
                expansion_needed = false;
            }

            // // TODO: this will not handle locked parts correctly as locked parts were drained out
            // if !helpers::all_parts_can_be_attempted(&self.unlocked_parts, shape.as_ref()) {
            //     expansion_needed = true;
            //     continue;
            // }

            while let Some(cur_part) = self.unlocked_parts.pop() {
                match self.place_unlocked_part(&mut plate, cur_part) {
                    None => {}
                    Some(part) => {
                        self.reset_cache();
                        self.unlocked_parts.push(part);
                        // Reclaim all parts
                        for part in &mut plate.parts.drain(..) {
                            if !part.part.locked {
                                self.unlocked_parts.push(part)
                            } else {
                                self.locked_parts.push(part);
                            }
                        }
                        self.unlocked_parts
                            .sort_by(|x, y| x.insertion_index.cmp(&y.insertion_index));

                        expansion_needed = true;
                        break;
                    }
                }
            }
        }

        let mut solution = Solution::new();
        solution.add_plate(plate);
        Some(solution)
    }

    fn place_single_plate_exp(&mut self) -> Option<Solution> {
        let original_shape = Clone::clone(&self.request.plate_shape);

        for (i, part) in self.unlocked_parts.iter_mut().enumerate() {
            part.insertion_index = i;
        }

        let res = Clone::clone(&self.smallest_observed_plate);

        let bottom_left = (
            self.request.center_x - original_shape.width() / (2.0 * self.request.precision),
            self.request.center_y - original_shape.height() / (2.0 * self.request.precision),
        );

        let mut hash_map: HashMap<usize, Attempts<Solution>> = HashMap::new();
        let mut search = {
            let original_shape = &original_shape;
            let res = &res;
            let hash_map = &mut hash_map;
            |search_index: usize| {
                match hash_map.get(&search_index) {
                    Some(Attempts::Failure | Attempts::Solved(_)) => {}
                    Some(Attempts::ToCompute) | None => {
                        let solution =
                            find_solution(search_index, original_shape, res, self, bottom_left);
                        let res = match solution {
                            None => Attempts::Failure,
                            Some(sol) => Attempts::Solved(sol),
                        };
                        hash_map.insert(search_index, res);
                    }
                };

                match hash_map.get(&search_index) {
                    Some(Attempts::Failure) => false,
                    Some(Attempts::Solved(_)) => true,
                    _ => panic!("Case should have been handled"),
                }
            }
        };

        let smaller = if let Some(upper) = res {
            binary_search(1, upper - 1, &mut search)
        } else {
            exponential_search_simple(N + 1 + 50, &mut search, Some(5))
        };

        if let Some(mut index) = smaller {
            return hash_map.remove(&mut index).unwrap().into();
        }

        None
    }

    fn place_multi_plate(&mut self) -> Option<Solution> {
        let mut solution = Solution::new();

        let plate_shape = Clone::clone(&self.request.plate_shape);
        let plate = Plate::make_plate_with_placed_parts(
            plate_shape.as_ref(),
            self.request.precision,
            &mut Vec::clone(&self.locked_parts),
            self.request.center_x,
            self.request.center_y,
        )?;
        solution.add_plate(plate);

        let mut unlocked_parts = vec![];
        std::mem::swap(&mut unlocked_parts, &mut self.unlocked_parts);
        for part in unlocked_parts {
            let mut i = 0;
            let mut current_part = part;
            while i < solution.count_plates() {
                let res =
                    self.place_unlocked_part(solution.get_plate_mut(i).unwrap(), current_part);
                match res {
                    None => {
                        break;
                    }
                    Some(part) => {
                        if i + 1 == solution.count_plates() {
                            let shape = Clone::clone(&self.request.plate_shape);

                            // Multi plates and ownership of locked parts
                            let next_plate = Plate::make_plate_with_placed_parts(
                                shape.as_ref(),
                                self.request.precision,
                                &mut Vec::clone(&self.locked_parts),
                                self.request.center_x,
                                self.request.center_y,
                            )
                            .unwrap();
                            solution.add_plate(next_plate);
                        }
                        current_part = part;
                    }
                }
                i += 1;
            }
        }

        self.unlocked_parts.clear();
        Some(solution)
    }

    pub(crate) fn place(&mut self) -> Option<Solution> {
        if self.request.single_plate_mode {
            match self.request.algorithm.bed_expansion_mode {
                BedExpansionMode::Linear => self.place_single_plate_linear(),
                BedExpansionMode::Exponential => self.place_single_plate_exp(),
            }
        } else {
            self.place_multi_plate()
        }
    }
}

mod helpers;
mod rect;
pub(crate) mod score;
mod search;
mod strategies;
