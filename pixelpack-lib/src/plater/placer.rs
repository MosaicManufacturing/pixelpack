use std::cmp::Ordering;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::vec;
use log::{info, log};
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::plater::placed_part::PlacedPart;
use crate::plater::placer::Alt::{Fst, Snd};
use crate::plater::placer::Attempts::{Failure, Solved, ToCompute};
use crate::plater::placer::GravityMode::{GravityEQ, GravityXY, GravityYX};
use crate::plater::plate::Plate;
use crate::plater::plate_shape::PlateShape;
use crate::plater::request::{BedExpansionMode, ConfigOrder, PointEnumerationMode, Request, Strategy};
use crate::plater::solution::Solution;
use crate::plater::spiral::{RepeatIter, spiral_iterator};

#[cfg(test)]
mod tests {
    use log::info;
    use crate::plater::placer::Rect;

    #[test]
    fn exploration() {

        let fst = Rect {
            width: 2.0,
            height: 2.0,
            center_x: 0.0,
            center_y: 0.0
        };
        let snd = Rect {
            width: 2.0,
            height: 2.0,
            center_x: 1.0,
            center_y: 0.0
        };
        let trd = Rect {
            width: 3.0,
            height: 2.0,
            center_x: 1.5,
            center_y: 0.0
        };


        let res = fst.combine(&snd);
        assert_eq!(res, trd);
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    width: f64,
    height: f64,
    center_x: f64,
    center_y: f64
}


impl Rect {
    fn combine(&self, other: &Self)  -> Self {
        let top_height = f64::max(self.height/2.0 + self.center_y, other.height/2.0 + other.center_y);
        let bottom_height = f64::min(-self.height/2.0 + self.center_y, -other.height/2.0 + other.center_y);

        let left_width = f64::min(-self.width/2.0 + self.center_x, -other.width/2.0 + other.center_x);
        let right_width = f64::max(self.width/2.0 + self.center_x, other.width/2.0 + other.center_x);

        Rect {
            width: right_width - left_width,
            height: top_height - bottom_height,
            center_x: (right_width + left_width)/2.0,
            center_y: (top_height + bottom_height)/2.0
        }
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
            GravityMode::GravityYX => 0,
            GravityMode::GravityXY => 1,
            GravityMode::GravityEQ => 2,
        }
    }
}

type PlateId = usize;

pub(crate) struct Placer<'a, S: PlateShape> {
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
    pub(crate) request: &'a Request<S>,
    // center_x, center_y, width, height
    current_bounding_box: Option<Rect>,
    pub smallest_observed_plate: Option<usize>
}

impl<'a, Shape: PlateShape> Placer<'a, Shape> {
    pub(crate) fn new(request: &'a Request<Shape>) -> Self {
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
            smallest_observed_plate: None
        };

        for part in request.parts.values() {
            let (x, y) = (part.center_x, part.center_y);
            let mut placed_part = PlacedPart::new_placed_part(part);

            if part.locked {
                placed_part.set_offset(x, y);
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
            SortMode::WidthDec =>  {
                self.unlocked_parts.sort_by(|x, y| {
                    let s1 = x.part.get_bitmap(0).width;
                    let s2 = y.part.get_bitmap(0).width;
                    i32::partial_cmp(&s1, &s2).unwrap()
                });
            },
            SortMode::HeightDec => {
                self.unlocked_parts.sort_by(|x, y| {
                    let s1 = x.part.get_bitmap(0).width;
                    let s2 = y.part.get_bitmap(0).width;
                    i32::partial_cmp(&s2, &s1).unwrap()
                });
            },
        }
    }

    pub(crate) fn set_gravity_mode(&mut self, gravity_mode: GravityMode) {
        let (new_x_coef, new_y_coef) = match gravity_mode {
            GravityMode::GravityYX => (1.0, 10.0),
            GravityMode::GravityXY => (10.0, 1.0),
            GravityMode::GravityEQ => (1.0, 1.0),
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



    fn place_single_plate_linear(&mut self) -> Solution<'a> {
        let mut shape = Clone::clone(&self.request.plate_shape);
        // TODO
        let mut plate = Plate::make_plate_with_placed_parts(
            &shape,
            self.request.precision,
            &mut Vec::clone(&self.locked_parts),
        ).unwrap();


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
                    &shape,
                    self.request.precision,
                    &mut Vec::clone(&self.locked_parts),
                ).unwrap();
                expansion_needed = false;
            }

            // TODO: this will not handle locked parts correctly as locked parts were drained out
            if  !all_parts_can_be_attempted(&self.unlocked_parts, &shape) {
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
        solution
    }

    fn place_single_plate_exp(&mut self) -> Option<Solution<'a>> {
        let mut shape = Clone::clone(&self.request.plate_shape);

        for (i, part) in self.unlocked_parts.iter_mut().enumerate() {
            part.insertion_index = i;
        }

        let mut expansion_needed = false;
        let expand_mm = 5.0;



        let m = f64::min(shape.width(), shape.height());
        // 32, 128
        let n = 128;
        let limit = 1024;

        let res= Clone::clone(&self.smallest_observed_plate);

        let f=  |i| {
            if let Some(x) = res {
                if i >= x {
                    return None;
                }
            }


            // if i  128 {
            //     return None;
            // }
            //
            //
            // let shape = if i == 128 {
            //     shape.clone()
            // } else {
            //
            //     shape.expand( f64::powf(i as f64, 1.0) as f64 * expand_mm)
            // };


            // TODO, if one of the models is locked, you have to align the new plate with the origin of th eprint bed

            // TODO: positioning of locked models not fully correct if plate is smaller
            // info!("Updated k, {} iteration", i);


            if i < n {
                return None;
            }
            //
            // if i < n && false {
            //     shape.intersect_square(m + (i as f64 - n as f64 + 1.0) * expand_mm, 10.0)?
            // } else

            let shape =  if i == n {
                shape.clone()
            } else {
                shape.expand( f64::powf((i - n) as f64, 1.0) as f64 * expand_mm)
            };

            // info!("For {}, width: {}, height: {}", i, shape.width(), shape.height());
            let mut unlocked_parts = Vec::clone(&self.unlocked_parts);
            let mut plate = Plate::make_plate_with_placed_parts(
                &shape,
                self.request.precision,
                &mut Vec::clone(&self.locked_parts),
            )?;


            for part in &self.locked_parts {
                info!("Before attempting place, {:#?} {:#?}", part.get_x(), part.get_y());
            }

            if !all_parts_can_be_attempted(&unlocked_parts, &shape) {
                // info!("Updated k attempt, Failed {} iteration", i);
                return None;
            }

            while let Some(cur_part) = unlocked_parts.pop() {
                let name = cur_part.part.id.to_owned();
                match self.place_unlocked_part(&mut plate, cur_part) {
                    None => {
                    }
                    Some(_) => {
                        // info!("Updated k place, {} iteration", i);
                        return None;
                    }
                }
            }

            // self.smallest_observed_plate = Some(n);
            info!("{} iteration complete", i);
            let mut solution = Solution::new();
            solution.add_plate(plate);
            solution.best_so_far = Some(i);
            Some(solution)
        };


        let mut buffer = vec![ToCompute; 2 * limit + 2];


      exponential_search(&mut buffer, limit + 1, f)
    }

    fn place_multi_plate(&mut self) -> Solution {
        let mut solution = Solution::new();

        let plate_shape = Clone::clone(&self.request.plate_shape);
        let plate = Plate::make_plate_with_placed_parts(
            &plate_shape,
            self.request.precision,
            &mut Vec::clone(&self.locked_parts),
        ).unwrap();
        solution.add_plate(plate);

        let mut unlocked_parts = vec![];
        std::mem::swap(&mut unlocked_parts, &mut self.unlocked_parts);

        info!("Unlocked parts len {}", unlocked_parts.len());

        info!("Multi part");

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
                                &shape,
                                self.request.precision,
                                &mut Vec::clone(&self.locked_parts),
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
        solution
    }

    pub(crate) fn place(&mut self) -> Option<Solution> {
        if self.request.single_plate_mode {
            match self.request.algorithm.bed_expansion_mode {
                BedExpansionMode::Linear => Some(self.place_single_plate_linear()),
                BedExpansionMode::Exponential => self.place_single_plate_exp()
            }
        } else {
            Some(self.place_multi_plate())
        }
    }
}

#[derive(Clone, Debug)]
enum Attempts<T> {
    ToCompute,
    Solved(T),
    Failure
}

fn exponential_search<T: Clone + Debug>(results: &mut Vec<Attempts<T>>, limit: usize, mut run: impl FnMut(usize) -> Option<T>) -> Option<T> {
    let mut first_found_solution = None;

    let mut i = 1;

    while i < limit {
        // info!("Loop {}", i);
        let res = run(i);
        if res.is_some() {
            first_found_solution = res;
            break;
        }

        if i * 2 >= limit {
            break;
        }

        i *= 2;
    }


    //
    // let mut results: Vec<Attempts<T>> = vec![ToCompute; i + 1 as usize];

    // results.clear();

    results
        .iter_mut()
        .for_each(|x| *x = ToCompute);

    if results.len() < i + 1 {
        info!("Failed {} {}", results.len(), i + 1);
        unreachable!()
    }

    let mut j = 1;
    while j < i {
        results[j] = Failure;
        j *= 2;
    }

    if first_found_solution.is_none() {
        //info!("tag fail {:#?}", results);
        return None;
    }


    results[(i) as usize] = Solved(first_found_solution.unwrap());

    let mut lo = 0 as usize;
    let mut hi = (i) as usize;

    let mut boundary_index = 1;

    while lo <= hi {
        let mut mid = (lo + hi)/2;

        info!("LO: {}, HI: {}, MID: {}", lo, hi, mid);

        if let ToCompute = results[mid] {
            results[mid] = match run(mid) {
                None => Failure,
                Some(x) => Solved(x)
            }
        }

        match results[mid] {
            Solved(_) => {

                if mid == 0 {
                    boundary_index = mid as i32;
                    break;
                }

                if let ToCompute = results[mid-1] {
                    results[mid-1] = match run(mid-1) {
                        None => Failure,
                        Some(x) => Solved(x)
                    }
                }

                if let Failure = results[mid-1] {
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

    info!("Boundary index {}", boundary_index);


    // info!("tag {:#?}", results);

    let mut ans = ToCompute;
    std::mem::swap(&mut ans, &mut results[boundary_index as usize]);
    match ans {
        Solved(x) => Some(x),
        _ => None
    }
}


// If for every model, there exists some rotation that fits try it
fn all_parts_can_be_attempted<S: PlateShape>(parts: &Vec<PlacedPart>, plate_shape: &S) -> bool {
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



enum CombinedIterator<A: Copy, B:Iterator> {
    XFixed {x: A, it: B},
    YFixed {y: A, it: B}

}

#[derive(Copy, Clone)]
struct FloatIterator {
    start: f64,
    end: f64,
    dx: f64
}

impl Iterator for FloatIterator{
    type Item = f64;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let res = Some(self.start);
            self.start += self.dx;
            res
        } else {
            None
        }
    }
}

enum Alt<A, B> {
    Fst(A, B),
    Snd(B, A)
}

impl<A> Into<(A, A)> for Alt<A, A> {
    fn into(self) -> (A, A) {
        match self {
            Fst(x, y) => (x, y),
            Snd(x, y) => (x, y)
        }
    }
}

impl<A: Copy, B:Iterator> Iterator for CombinedIterator<A, B> {
    type Item =  Alt<A, B::Item>;

    fn next(&mut self) -> Option<Self::Item> {

        match self{
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


fn test(dx: f64, width: f64, height: f64) -> impl Iterator<Item=(f64, f64)> {
    let x = FloatIterator {
        start: 0.0,
        end: width,
        dx
    };

    let y = FloatIterator {
        start: 0.0,
        end: height,
        dx
    };

    x
        .into_iter()
        .flat_map(move |x| CombinedIterator::XFixed {x, it: y })
        .map(|x: Alt<f64, f64>| x.into())

}

mod strategies;