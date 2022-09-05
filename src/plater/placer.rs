use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::pin::Pin;
use std::rc::Rc;
use crate::plater::placed_part::PlacedPart;
use crate::plater::request::Request;
use rand::rngs::StdRng;
use rand::thread_rng;
use rand::seq::SliceRandom;
use crate::plater::part::Part;
use crate::plater::placer::GravityMode::{GravityEQ, GravityXY, GravityYX};
use crate::plater::plate::Plate;
use crate::plater::plate_shape::PlateShape;
use crate::plater::solution::Solution;

#[derive(Clone)]
pub(crate) enum SortMode {
    // SortSurfaceDec sorts parts in descending order of surface area.
    SortSurfaceDec,
    // SortSurfaceInc sorts parts in ascending order of surface area.
    SortSurfaceInc,
    // SortShuffle sorts parts in random order.
    SortShuffle
}

impl From<SortMode> for usize {
    fn from(x: SortMode) -> Self {
        match x {
            SortMode::SortSurfaceDec => {0}
            SortMode::SortSurfaceInc => {1}
            SortMode::SortShuffle => {2}
        }
    }
}

pub(crate) enum GravityMode {
    // GravityYX gives Y score a weighting of 10 times the X score.
    GravityYX,
    // GravityXY gives X score a weighting of 10 times the Y score.
    GravityXY,
    // GravityEQ gives X and Y scores equal weighting.
    GravityEQ
}

pub(crate) const GRAVITY_MODE_LIST: [GravityMode; 3] = [GravityYX, GravityXY, GravityEQ];

impl From<GravityMode> for usize {
    fn from(x:GravityMode) -> Self {
        match x {
            GravityMode::GravityYX => {0},
            GravityMode::GravityXY => {1},
            GravityMode::GravityEQ => {2},
        }
    }
}

type PlateId = usize;

pub struct Placer {
    rotate_offset: i32,
    rotate_direction: i32, // 0 = CCW, 1 = CW, TODO: make an enum
    cache: HashMap<PlateId, HashMap<String, bool>>,

    // scoring weights
    x_coef: f64,
    y_coef: f64,
    // input data
    locked_parts: Vec<Rc<RefCell<PlacedPart>>>,
    unlocked_parts: Vec<Rc<RefCell<PlacedPart>>>,
    request: Request
}

impl Placer {
    pub(crate) fn new(request: Request) -> Self {
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

        for (_, part) in &p.request.parts {
            let placed_part =
                Rc::new(
                RefCell::new(
                    PlacedPart::new_placed_part(
                        Rc::clone(part))));
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

    fn sort_parts(&mut self, sort_mode: SortMode) {
        match sort_mode {
            SortMode::SortSurfaceDec => {
                self.unlocked_parts.sort_by(|x, y| {
                    let s1 = (**x).borrow().get_surface();
                    let s2 = (**y).borrow().get_surface();
                    f64::partial_cmp(&s1, &s2).unwrap()
                })
            }
            SortMode::SortSurfaceInc => {
                self.unlocked_parts.sort_by(|x, y| {
                    let s1 = (**x).borrow().get_surface();
                    let s2 = (**y).borrow().get_surface();
                    f64::partial_cmp(&s2, &s1).unwrap()
                });
            }
            SortMode::SortShuffle => {
                let mut rng = thread_rng();
                self.unlocked_parts.shuffle(&mut rng)
            }
        }
    }

    fn set_gravity_mode(&mut self, gravity_mode: GravityMode) {
        let (new_x_coef, new_y_coef) = match gravity_mode {
            GravityMode::GravityYX => { (1.0, 10.0) }
            GravityMode::GravityXY => { (10.0, 1.0) }
            GravityMode::GravityEQ => { (1.0, 1.0) }
        };

        self.y_coef = new_x_coef;
        self.y_coef = new_y_coef;
    }

    fn set_rotation_direction(&mut self, direction: i32) {
        self.rotate_direction = direction;
    }

    fn set_rotate_offset(&mut self, offset: i32) {
        self.rotate_offset = offset;
    }

    fn make_plate(&mut self, shape: &dyn PlateShape) -> Plate {
        let mut plate = Plate::new(shape, self.request.precision);
        for part in &self.locked_parts {
            plate.place(Rc::clone(part));
        }
        plate
    }

    // Internal borrow mut
    fn place_unlocked_part(&mut self, plate: &mut Plate, part: &Rc<RefCell<PlacedPart>>) -> bool {
        let cache_name;

            let mut borrowed_part = RefCell::borrow_mut(part);
            // TODO: optimize string clone
            cache_name = String::from(borrowed_part.get_id());



        if self.cache.get(&plate.plate_id).is_none() {
            self.cache.insert(plate.plate_id, HashMap::new());
        }

        let k = self.cache.get(&plate.plate_id).unwrap().get(cache_name.as_str());
        // If already seen, don't recompute
        if let Some(_) = k {
            return false;
        }

        let mut better_x = 0.0;
        let mut better_y = 0.0;
        let mut better_score = 0.0;
        let mut better_r = 0;
        let mut found = false;

        let mut rs = f64::ceil(PI * 2.0/self.request.delta_r) as usize;

        // Conditionally reverse iteration direction
        let iter = if self.rotate_direction != 0 {
            itertools::Either::Left((0..rs).rev())
        } else {
            itertools::Either::Right(0..rs)
        };

        for r in iter {
            let vr = (r + self.rotate_offset as usize) % rs;
            borrowed_part.set_rotation(vr as i32);

            let delta = self.request.delta;
            let mut x = 0.0;
            while x < plate.width {
                let mut y = 0.0;
                while y < plate.height {
                    let gx = borrowed_part.get_gx() + x;
                    let gy = borrowed_part.get_gy() + y;

                    let score = gy * self.y_coef + gx * self.x_coef;

                    if !found || score < better_score {
                        borrowed_part.set_offset(x, y);
                        if plate.can_place(&borrowed_part) {
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
                borrowed_part.set_rotation(better_r as i32);
                borrowed_part.set_offset(better_x, better_y);
                plate.place(Rc::clone(part));
                true
            } else {
                self
                    .cache
                    .get_mut(&plate.plate_id)
                    .unwrap()
                    .insert(String::from(cache_name), true);
                false
            }
        }
        // TODO: verify correctness
        false
    }

    fn place_single_plate(&mut self) -> Solution {

        // TODO: TRY TO OPTIMIZE AWAY THE RC CLONE // PASS HEIGHT, WIDTH INSTEAD OF CLONING
        let mut shape = Rc::clone(&self.request.plate_shape);
        let mut plate = self.make_plate(shape.as_ref());

        self.locked_parts.clear();
        let mut all_placed = false;

        // TODO: Optimization, make locked and unlocked parts Rc<Vec<_>>
        let rc_cloned_unlocked_parts =
            (&self.unlocked_parts)
            .into_iter()
            .map(Rc::clone)
            .collect::<Vec<_>>();


        while !all_placed {
            all_placed = true;
            self.reset_cache();

            for part in &rc_cloned_unlocked_parts {
                if !self.place_unlocked_part(&mut plate, part) {
                    all_placed = false;
                    break;
                }
            }
            if !all_placed {
                let EXPAND_MM = 100.0;
                shape = shape.expand(EXPAND_MM);
                plate = self.make_plate(shape.as_ref());
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
            let plate_shape = Rc::clone(&self.request.plate_shape);
            let mut plate = self.make_plate(plate_shape.as_ref());
            solution.add_plate(plate);
        }
        // TODO: Optimization, make locked and unlocked parts Rc<Vec<_>>
        let rc_cloned_unlocked_parts =
            (&self.unlocked_parts)
                .into_iter()
                .map(Rc::clone)
                .collect::<Vec<_>>();

        for part in &rc_cloned_unlocked_parts {
            let mut placed = false;
            let mut i = 0;
            while i < solution.count_plates() && !placed {
                let plate = solution.get_plate_mut(i).unwrap();
                if self.place_unlocked_part(plate, part) {
                    placed = true;
                } else if i + 1 == solution.count_plates() {
                    let shape = Rc::clone(&self.request.plate_shape);
                    let plate = self.make_plate(shape.as_ref());
                    solution.add_plate(plate)
                }
                i += 1;
            }
        }

        self.unlocked_parts.clear();
        solution
    }

    fn place(&mut self) -> Solution {
        if self.request.single_plate_mode {
            self.place_single_plate()
        } else {
            self.place_multi_plate()
        }
    }
}
