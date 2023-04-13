use std::cmp::Ordering;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::vec;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::plater::placed_part::PlacedPart;
use crate::plater::placer::Alt::{Fst, Snd};
use crate::plater::placer::Attempts::{Failure, Solved, ToCompute};
use crate::plater::placer::GravityMode::{GravityEQ, GravityXY, GravityYX};
use crate::plater::plate::Plate;
use crate::plater::plate_shape::PlateShape;
use crate::plater::request::{BedExpansionMode, Request};
use crate::plater::solution::Solution;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    width: f64,
    height: f64,
    center_x: f64,
    center_y: f64,
}


impl Rect {
    fn combine(&self, other: &Self) -> Self {
        let top_height = f64::max(self.height / 2.0 + self.center_y, other.height / 2.0 + other.center_y);
        let bottom_height = f64::min(-self.height / 2.0 + self.center_y, -other.height / 2.0 + other.center_y);

        let left_width = f64::min(-self.width / 2.0 + self.center_x, -other.width / 2.0 + other.center_x);
        let right_width = f64::max(self.width / 2.0 + self.center_x, other.width / 2.0 + other.center_x);

        Rect {
            width: right_width - left_width,
            height: top_height - bottom_height,
            center_x: (right_width + left_width) / 2.0,
            center_y: (top_height + bottom_height) / 2.0,
        }
    }

    fn get_points(&self) -> [(f64, f64); 4] {
        let w2 = self.width / 2.0;
        let h2 = self.height / 2.0;
        [
            (self.center_x + w2, self.center_y + h2),
            (self.center_x - w2, self.center_y + h2),
            (self.center_x + w2, self.center_y - h2),
            (self.center_x - w2, self.center_y - h2)
        ]
    }
}


impl PartialOrd for Rect {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        f64::partial_cmp(&(self.width * self.height), &(other.width * other.height))
    }
}

#[derive(Clone, Copy)]
pub enum SortMode {
    // SortSurfaceDec sorts parts in descending order of surface area.
    SurfaceDec,
    // SortSurfaceInc sorts parts in ascending order of surface area.
    SurfaceInc,
    // SortShuffle sorts parts in random order.
    Shuffle,
    WidthDec,
    HeightDec,
}

impl From<SortMode> for usize {
    fn from(x: SortMode) -> Self {
        match x {
            SortMode::SurfaceDec => 0,
            SortMode::SurfaceInc => 1,
            SortMode::Shuffle => 2,
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
        };

        for part in request.parts.values() {
            let (off_x, off_y) = (part.center_x - part.width/2.0, part.center_y - part.height/2.0);
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
            SortMode::Shuffle => {
                let mut rng = thread_rng();
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

            // TODO: this will not handle locked parts correctly as locked parts were drained out
            if !all_parts_can_be_attempted(&self.unlocked_parts, shape.as_ref()) {
                expansion_needed = true;
                continue;
            }

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
                            .sort_by(|x, y| {
                                x.insertion_index.cmp(&y.insertion_index)
                            });

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

        let expand_mm = 5.0;
        // 32, 128
        let n = 1024;

        let res = Clone::clone(&self.smallest_observed_plate);

        let bottom_left = (self.request.center_x - original_shape.width() / (2.0 * self.request.precision)
                           , self.request.center_y - original_shape.height() / (2.0 * self.request.precision));

        let mut f = |i| {
            if let Some(x) = res {
                if i >= x {
                    return None;
                }
            }

            let mut should_align_to_bed = false;
            self.current_bounding_box = None;

            let mut shape = if i < n {
                original_shape.contract((n - i) as f64 * expand_mm)?
            } else if i == n {
                original_shape.clone()
            } else {
                should_align_to_bed = true;
                original_shape.expand(original_shape.width() / self.request.precision)
            };

            let center = if i <= n {
                (self.request.center_x, self.request.center_y)
            } else {
                (bottom_left.0 + shape.width() / (2.0 * self.request.precision), bottom_left.1 + shape.height() / (2.0 * self.request.precision))
            };

            let mut unlocked_parts = Vec::clone(&self.unlocked_parts);
            let mut plate = Plate::make_plate_with_placed_parts(
                shape.as_ref(),
                self.request.precision,
                &mut Vec::clone(&self.locked_parts),
                center.0,
                center.1,
            )?;


            if i <= n && !all_parts_can_be_attempted(&unlocked_parts, shape.as_ref()) {
                return None;
                // Add special handling if some parts will never fit
            } else if i > n && !all_parts_can_eventually_be_attempted(&unlocked_parts, shape.as_ref()) {
                return None;
            }


            // Determine current bounding box using pixel data from bitmap

            // if !self.locked_parts.is_empty() {
            //     plate
            // }

            while let Some(cur_part) = unlocked_parts.pop() {
                match self.place_unlocked_part(&mut plate, cur_part) {
                    None => {}
                    Some(part) => {
                        if i <= n {
                            return None;
                        }
                        should_align_to_bed = true;
                        unlocked_parts.push(part);
                        shape = shape.expand(original_shape.width() / original_shape.resolution());
                        plate = Plate::make_from_shape(&mut plate, shape.as_ref(), bottom_left)
                    }
                }
            }

            // Centering models is only correct if there are no locked parts
            if self.locked_parts.is_empty() {
                if should_align_to_bed {
                    let width = self.request.plate_shape.width();
                    let height = self.request.plate_shape.height();
                    plate.align(width, height);
                } else {
                    plate.center();
                }
            }

            let mut solution = Solution::new();
            solution.add_plate(plate);
            solution.best_so_far = Some(i);
            Some(solution)
        };

        let smaller = exponential_search(n + 1, &mut f).map(|x| x.0);
        if smaller.is_some() {
            return smaller;
        }

        for i in (n + 1)..(n + 50) {
            let res = f(i);
            if res.is_some() {
                return res;
            }
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
                            ).unwrap();
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
                BedExpansionMode::Exponential => self.place_single_plate_exp()
            }
        } else {
            self.place_multi_plate()
        }
    }
}

#[derive(Clone, Debug)]
pub enum Attempts<T> {
    ToCompute,
    Solved(T),
    Failure,
}


pub(crate) fn exponential_search<T: Clone + Debug>(limit: usize, mut run: impl FnMut(usize) -> Option<T>) -> Option<(T, usize)> {
    let mut first_found_solution = None;

    let mut i = 1;
    let mut lower = i;

    while i < limit {
        let res = run(i);
        if res.is_some() {
            first_found_solution = res;
            break;
        }

        if i * 2 >= limit {
            break;
        }

        lower = i;
        i *= 2;
    }


    let mut results = vec![ToCompute; 2 * limit];


    results
        .iter_mut()
        .for_each(|x| *x = ToCompute);

    if results.len() < i + 1 {
        unreachable!()
    }

    let mut j = 1;
    while j < i {
        results[j] = Failure;
        j *= 2;
    }

    if first_found_solution.is_none() {
        return None;
    }


    results[(i) as usize] = Solved(first_found_solution.unwrap());

    let mut lo = lower as usize;
    let mut hi = (i) as usize;

    let mut boundary_index = 1;

    while lo <= hi {
        let gap = hi - lo;
        let mid = lo + gap / 2;
        if let ToCompute = results[mid] {
            results[mid] = match run(mid) {
                None => Failure,
                Some(x) => Solved(x)
            }
        }

        match results[mid] {
            Solved(_) => {
                if mid == 1 {
                    boundary_index = mid as i32;
                    break;
                }

                if let ToCompute = results[mid - 1] {
                    results[mid - 1] = match run(mid - 1) {
                        None => Failure,
                        Some(x) => Solved(x)
                    }
                }

                if let Failure = results[mid - 1] {
                    boundary_index = mid as i32;
                    break;
                }

                hi = mid - 1;
            }
            Failure => {
                lo = mid + 1;
            }
            ToCompute => unreachable!()
        }
    }

    let mut ans = ToCompute;
    std::mem::swap(&mut ans, &mut results[boundary_index as usize]);
    match ans {
        Solved(x) => Some((x, boundary_index as usize)),
        _ => None
    }
}


// If for every model, there exists some rotation that fits try it
fn all_parts_can_be_attempted(parts: &Vec<PlacedPart>, plate_shape: &dyn PlateShape) -> bool {
    parts
        .iter()
        .map(|part| part
            .part
            .bitmaps
            .iter()
            .map(|x| {
                x.width as f64 <= plate_shape.width()
                    && x.height as f64 <= plate_shape.height()
            }).any(|x| x))
        .all(|x| x)
}

// If for every model, there exists some rotation that fits try it
fn all_parts_can_eventually_be_attempted(parts: &Vec<PlacedPart>, plate_shape: &dyn PlateShape) -> bool {
    parts
        .iter()
        .map(|part| part
            .part
            .bitmaps
            .iter()
            .map(|x| {
                x.height as f64 <= plate_shape.height()
            }).any(|x| x))
        .all(|x| x)
}


enum CombinedIterator<A: Copy, B: Iterator> {
    XFixed { x: A, it: B },
    YFixed { y: A, it: B },
}

enum Alt<A, B> {
    Fst(A, B),
    Snd(B, A),
}

impl<A> Into<(A, A)> for Alt<A, A> {
    fn into(self) -> (A, A) {
        match self {
            Fst(x, y) => (x, y),
            Snd(x, y) => (x, y)
        }
    }
}

impl<A: Copy, B: Iterator> Iterator for CombinedIterator<A, B> {
    type Item = Alt<A, B::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CombinedIterator::XFixed { x, it } => {
                match it.next() {
                    None => None,
                    Some(y) => Some(Fst(*x, y))
                }
            }
            CombinedIterator::YFixed { y, it } => {
                match it.next() {
                    None => None,
                    Some(x) => Some(Snd(x, *y))
                }
            }
        }
    }
}


mod strategies;
mod score;