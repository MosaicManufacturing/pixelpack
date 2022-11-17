use std::collections::HashMap;
use std::f64::consts::PI;
use log::{info, log};
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::plater::placed_part::PlacedPart;
use crate::plater::placer::Alt::{Fst, Snd};
use crate::plater::placer::Attempts::{Failure, Solved, ToCompute};
use crate::plater::placer::GravityMode::{GravityEQ, GravityXY, GravityYX};
use crate::plater::plate::Plate;
use crate::plater::plate_shape::PlateShape;
use crate::plater::request::Request;
use crate::plater::solution::Solution;
use crate::plater::spiral::spiral_iterator;

#[derive(Clone, Copy)]
pub enum SortMode {
    // SortSurfaceDec sorts parts in descending order of surface area.
    SurfaceDec,
    // SortSurfaceInc sorts parts in ascending order of surface area.
    SurfaceInc,
    // SortShuffle sorts parts in random order.
    Shuffle,
}

impl From<SortMode> for usize {
    fn from(x: SortMode) -> Self {
        match x {
            SortMode::SurfaceDec => 0,
            SortMode::SurfaceInc => 1,
            SortMode::Shuffle => 2,
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
    request: &'a Request<S>,
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
        };

        for part in request.parts.values() {
            let placed_part = PlacedPart::new_placed_part(part);
            if part.locked {
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

    fn place_unlocked_part<'b>(
        &mut self,
        plate: &mut Plate<'b>,
        mut part: PlacedPart<'b>,
    ) -> Option<PlacedPart<'b>> {
        let cache_name = String::from(part.get_id());

        if self.cache.get(&plate.plate_id).is_none() {
            self.cache.insert(plate.plate_id, HashMap::new());
        }

        let k = self
            .cache
            .get(&plate.plate_id)
            .unwrap()
            .get(cache_name.as_str());
        // If already seen, don't recompute
        if k.is_some() {
            return None;
        }

        let mut better_x = 0.0;
        let mut better_y = 0.0;
        let mut better_score = 0.0;
        let mut better_r = 0;
        let mut found = false;

        let rs = f64::ceil(PI * 2.0 / self.request.delta_r) as usize;

        for r in 0..rs {
            let vr = (r + self.rotate_offset as usize) % rs;
            part.set_rotation(vr as i32);

            let bmp = part.get_bitmap();
            if !(bmp.width as f64 <= plate.width && bmp.height as f64 <= plate.height) {
                continue;
            }

            let delta = self.request.delta;
            let mut x = 0.0;

            for (x, y) in spiral_iterator(delta, plate.width, plate.height) {
                let gx = part.get_gx() + x;
                let gy = part.get_gy() + y;

                let score = gy * self.y_coef + gx * self.x_coef;

                if !found  {
                    part.set_offset(x, y);
                    if plate.can_place(&part) {
                        found = true;
                        better_x = x;
                        better_y = y;
                        better_score = score;
                        better_r = vr;
                        break;
                    }
                }
            }

            return if found {
                part.set_rotation(better_r as i32);
                part.set_offset(better_x, better_y);
                plate.place(part);
                None
            } else {
                self.cache
                    .get_mut(&plate.plate_id)
                    .unwrap()
                    .insert(cache_name, true);
                Some(part)
            };
        }

        Some(part)
    }

    fn _place_unlocked_part<'b>(
        &mut self,
        plate: &mut Plate<'b>,
        mut part: PlacedPart<'b>,
    ) -> Option<PlacedPart<'b>> {
        let cache_name = String::from(part.get_id());

        if self.cache.get(&plate.plate_id).is_none() {
            self.cache.insert(plate.plate_id, HashMap::new());
        }

        let k = self
            .cache
            .get(&plate.plate_id)
            .unwrap()
            .get(cache_name.as_str());
        // If already seen, don't recompute
        if k.is_some() {
            return None;
        }

        let mut better_x = 0.0;
        let mut better_y = 0.0;
        let mut better_score = 0.0;
        let mut better_r = 0;
        let mut found = false;

        let rs = f64::ceil(PI * 2.0 / self.request.delta_r) as usize;

        // Conditionally reverse iteration direction
        let iter = if self.rotate_direction != 0 {
            itertools::Either::Left((0..rs).rev())
        } else {
            itertools::Either::Right(0..rs)
        };


        // let iter = 0..1;
        for r in iter {
            let vr = (r + self.rotate_offset as usize) % rs;
            part.set_rotation(vr as i32);

            let bmp = part.get_bitmap();
            if !(bmp.width as f64 <= plate.width && bmp.height as f64 <= plate.height) {
                continue;
            }

            let delta = self.request.delta;
            let mut x = 0.0;
            while x < plate.width {
                let mut y = 0.0;
                while y < plate.height {
                    let gx = part.get_gx() + x;
                    let gy = part.get_gy() + y;

                    let score = gy * self.y_coef + gx * self.x_coef;

                    if !found || score < better_score {
                        part.set_offset(x, y);
                        if plate.can_place(&part) {
                            found = true;
                            better_x = x;
                            better_y = y;
                            better_score = score;
                            better_r = vr;
                        }
                    }

                    y += delta;
                }
                x += delta
            }

            return if found {
                part.set_rotation(better_r as i32);
                part.set_offset(better_x, better_y);
                plate.place(part);
                None
            } else {
                self.cache
                    .get_mut(&plate.plate_id)
                    .unwrap()
                    .insert(cache_name, true);
                Some(part)
            };
        }

        Some(part)
    }

    fn place_once(&mut self) -> Solution<'a> {
        let mut shape = Clone::clone(&self.request.plate_shape);
        let mut plate = Plate::make_plate_with_placed_parts(
            &shape,
            self.request.precision,
            &mut self.locked_parts,
        );
        let mut unlocked_parts = vec![];

        std::mem::swap(&mut self.unlocked_parts, &mut unlocked_parts);

        self.reset_cache();
        while !unlocked_parts.is_empty() {
            let part = unlocked_parts.pop().unwrap();
            match self.place_unlocked_part(&mut plate, part) {
                None => {}
                Some(part) => {
                    self.cache.clear();
                    unlocked_parts.push(part);
                    let expand_mm = 100.0;
                    shape = shape.expand(expand_mm);
                    plate = plate.make_from(&shape, self.request.precision);
                }
            }
        }

        self.unlocked_parts.clear();
        let mut solution = Solution::new();
        solution.add_plate(plate);
        solution
    }

    fn place_single_plate(&mut self) -> Solution<'a> {
        let mut shape = Clone::clone(&self.request.plate_shape);
        let mut plate = Plate::make_plate_with_placed_parts(
            &shape,
            self.request.precision,
            &mut Vec::clone(&self.locked_parts),
        );


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
                );
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

    fn _place_single_plate(&mut self) -> Solution<'a> {
        let mut shape = Clone::clone(&self.request.plate_shape);
        // let mut plate = Plate::make_plate_with_placed_parts(
        //     &shape,
        //     self.request.precision,
        //     &mut Vec::clone(&self.locked_parts),
        // );


        for (i, part) in self.unlocked_parts.iter_mut().enumerate() {
            part.insertion_index = i;
        }

        let mut expansion_needed = false;
        let expand_mm = 10.0;


        exponential_search(|i| {
            info!("{} iteration", i);
            let shape = shape.expand(f64::sqrt(i as f64) * expand_mm);
            let mut unlocked_parts = Vec::clone(&self.unlocked_parts);
            let mut plate = Plate::make_plate_with_placed_parts(
                &shape,
                self.request.precision,
                &mut Vec::clone(&self.locked_parts),
            );

            if !all_parts_can_be_attempted(&unlocked_parts, &shape) {
                return None;
            }

            while let Some(cur_part) = unlocked_parts.pop() {
                match self.place_unlocked_part(&mut plate, cur_part) {
                    None => {}
                    Some(_) => {
                        return None;
                    }
                }
            }

            info!("{} iteration complete", i);
            let mut solution = Solution::new();
            solution.add_plate(plate);
            Some(solution)
        })

    }

    fn place_multi_plate(&mut self) -> Solution {
        let mut solution = Solution::new();

        let plate_shape = Clone::clone(&self.request.plate_shape);
        let plate = Plate::make_plate_with_placed_parts(
            &plate_shape,
            self.request.precision,
            &mut Vec::clone(&self.locked_parts),
        );
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
                            );
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

    pub(crate) fn place(&mut self) -> Solution {
        if self.request.single_plate_mode {
            self.place_single_plate()
        } else {
            self.place_multi_plate()
        }
    }
}
// def binary_search(arr, x):
// low = 0
// high = len(arr) - 1
// mid = 0
//
// while low <= high:
//
// mid = (high + low) // 2
//
// # If x is greater, ignore left half
// if arr[mid] < x:
// low = mid + 1
//
// # If x is smaller, ignore right half
// elif arr[mid] > x:
// high = mid - 1
//
// # means x is present at mid
// else:
// return mid
//
// # If we reach here, then the element was not present
// return -1
//

#[derive(Clone)]
enum Attempts<T> {
    ToCompute,
    Solved(T),
    Failure
}

fn exponential_search<T: Clone>(mut run: impl FnMut(usize) -> Option<T>) -> T {
    let first_found_solution;

    let mut i = 1;
    loop {
        info!("Loop {}", i);
        let res = run(i);
        if res.is_some() {
            first_found_solution = res.unwrap();
            break;
        }

        i *= 2;
    }

    let mut results: Vec<Attempts<T>> = vec![ToCompute; i as usize];

    let mut j = 1;
    while j < i {
        results[j] = Failure;
        j *= 2;
    }


    results[(i-1) as usize] = Solved(first_found_solution);

    let mut lo = 0 as usize;
    let mut hi = (i - 1) as usize;

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


    let mut ans = ToCompute;
    std::mem::swap(&mut ans, &mut results[boundary_index as usize]);
    match ans {
        Solved(x) => x,
        _ => unreachable!()
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