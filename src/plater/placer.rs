use std::collections::HashMap;
use std::f64::consts::PI;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::plater::placed_part::PlacedPart;
use crate::plater::placer::GravityMode::{GravityEQ, GravityXY, GravityYX};
use crate::plater::plate::Plate;
use crate::plater::plate_shape::PlateShape;
use crate::plater::request::Request;
use crate::plater::solution::Solution;

#[derive(Clone, Copy)]
pub(crate) enum SortMode {
    // SortSurfaceDec sorts parts in descending order of surface area.
    SortSurfaceDec,
    // SortSurfaceInc sorts parts in ascending order of surface area.
    SortSurfaceInc,
    // SortShuffle sorts parts in random order.
    SortShuffle,
}

impl From<SortMode> for usize {
    fn from(x: SortMode) -> Self {
        match x {
            SortMode::SortSurfaceDec => { 0 }
            SortMode::SortSurfaceInc => { 1 }
            SortMode::SortShuffle => { 2 }
        }
    }
}

pub(crate) enum GravityMode {
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
            GravityMode::GravityYX => { 0 }
            GravityMode::GravityXY => { 1 }
            GravityMode::GravityEQ => { 2 }
        }
    }
}

type PlateId = usize;

pub struct Placer<'a, Shape: PlateShape> {
    rotate_offset: i32,
    rotate_direction: i32,
    // 0 = CCW, 1 = CW, TODO: make an enum
    cache: HashMap<PlateId, HashMap<String, bool>>,

    // scoring weights
    x_coef: f64,
    y_coef: f64,
    // input data
    locked_parts: Vec<PlacedPart<'a>>,
    unlocked_parts: Vec<PlacedPart<'a>>,
    request: &'a Request<'a, Shape>,
}

impl<'a, Shape: PlateShape> Placer<'a, Shape> {
    pub(crate) fn new(request: &'a Request<'a, Shape>) -> Self {
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

        for value in request.parts.values() {
            let part = *value;
            let placed_part =
                PlacedPart::new_placed_part(part);
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
            SortMode::SortSurfaceDec => {
                self.unlocked_parts.sort_by(|x, y| {
                    let s1 = x.get_surface();
                    let s2 = y.get_surface();
                    f64::partial_cmp(&s1, &s2).unwrap()
                })
            }
            SortMode::SortSurfaceInc => {
                self.unlocked_parts.sort_by(|x, y| {
                    let s1 = x.get_surface();
                    let s2 = y.get_surface();
                    f64::partial_cmp(&s2, &s1).unwrap()
                });
            }
            SortMode::SortShuffle => {
                let mut rng = thread_rng();
                self.unlocked_parts.shuffle(&mut rng)
            }
        }
    }

    pub(crate) fn set_gravity_mode(&mut self, gravity_mode: GravityMode) {
        let (new_x_coef, new_y_coef) = match gravity_mode {
            GravityMode::GravityYX => { (1.0, 10.0) }
            GravityMode::GravityXY => { (10.0, 1.0) }
            GravityMode::GravityEQ => { (1.0, 1.0) }
        };

        self.y_coef = new_x_coef;
        self.y_coef = new_y_coef;
    }

    pub(crate) fn set_rotate_direction(&mut self, direction: i32) {
        self.rotate_direction = direction;
    }

    pub(crate) fn set_rotate_offset(&mut self, offset: i32) {
        self.rotate_offset = offset;
    }

    fn make_plate(&mut self, shape: &Shape) -> Plate<'a> {
        let mut plate = Plate::new(shape, self.request.precision);

        let n = self.locked_parts.len();
        let xs = &mut self.locked_parts.drain(0..n);
        for part in xs {
            plate.place(part);
        }
        plate
    }

    // Internal borrow mut
    fn place_unlocked_part<'b>(&mut self, plate: &mut Plate<'b>, mut part: PlacedPart<'b>) -> Result<bool, PlacedPart<'b>> {
        let cache_name = String::from(part.get_id());

        if self.cache.get(&plate.plate_id).is_none() {
            self.cache.insert(plate.plate_id, HashMap::new());
        }

        let k = self.cache.get(&plate.plate_id).unwrap().get(cache_name.as_str());
        // If already seen, don't recompute
        if k.is_some() {
            return Ok(false);
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

        for r in iter {
            let vr = (r + self.rotate_offset as usize) % rs;
            part.set_rotation(vr as i32);

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
                Ok(true)
            } else {
                self
                    .cache
                    .get_mut(&plate.plate_id)
                    .unwrap()
                    .insert(cache_name, true);
                Err(part)
            };
        }
        // TODO: verify correctness
        Err(part)
    }

    fn place_single_plate(&mut self) -> Solution<'a> {

        // TODO: TRY TO OPTIMIZE AWAY THE RC CLONE // PASS HEIGHT, WIDTH INSTEAD OF CLONING
        let mut shape = Clone::clone(self.request.plate_shape);
        let mut plate = self.make_plate(&shape);

        self.locked_parts.clear();
        let mut all_placed = false;
        let mut unlocked_parts = vec![];

        std::mem::swap(&mut self.unlocked_parts, &mut unlocked_parts);
        while !all_placed {
            all_placed = true;
            self.reset_cache();

            let mut reclaimed_unlocked_parts = vec![];
            let n = (&self.unlocked_parts).len();
            for part in unlocked_parts.drain(0..n) {
                match self.place_unlocked_part(&mut plate, part) {
                    Ok(flag) => {
                        if !flag {
                            all_placed = false;
                            break;
                        }
                    }
                    Err(part) => {
                        reclaimed_unlocked_parts.push(part)
                    }
                }
            }
            if !all_placed {
                let EXPAND_MM = 100.0;
                shape = shape.expand(EXPAND_MM);

                let n = (&plate.parts).len();
                for part in &mut plate.parts.drain(0..n) {
                    reclaimed_unlocked_parts.push(part)
                }
                // If we reach here, we have drained all elements out of unlocked_parts so it is EMPTY

                // reclaimed_unlocked_parts contains all parts that were returned from self.place_unlocked_part and
                // we reclaimed all consumed parts that were in plate.parts

                // So, parts_to_handle contains all Parts that were originally in self.unlocked_parts
                std::mem::swap(&mut unlocked_parts, &mut reclaimed_unlocked_parts);
                plate = self.make_plate(&shape);
            }
        }

        self.unlocked_parts.clear();
        let mut solution = Solution::new();
        solution.add_plate(plate);

        solution
    }

    fn place_multi_plate(&mut self) -> Solution {
        let mut solution = Solution::new();

        {
            let plate_shape = Clone::clone(self.request.plate_shape);
            let plate = self.make_plate(&plate_shape);
            solution.add_plate(plate);
        }

        let mut unlocked_parts = vec![];
        std::mem::swap(&mut unlocked_parts, &mut self.unlocked_parts);

        for part in unlocked_parts {
            let mut i = 0;
            let mut current_part = part;
            while solution.count_plates() < i {
                let res = self.place_unlocked_part(solution.get_plate_mut(i).unwrap(), current_part);
                match res {
                    Ok(_) => {
                        break;
                    }
                    Err(part) => {
                        if i + 1 == solution.count_plates() {
                            let shape = Clone::clone(self.request.plate_shape);
                            let next_plate = self.make_plate(&shape);
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
