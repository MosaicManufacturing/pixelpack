use std::collections::HashMap;
use std::f64::consts::PI;

use crate::plater::placed_part::PlacedPart;
use crate::plater::placer::rect::Rect;
use crate::plater::placer::score::Position::{Inside, Outside};
use crate::plater::placer::score::Prefer;
use crate::plater::placer::score::Preference::Second;
use crate::plater::placer::score::{
    DefaultScoreWrapper, FloatWrapper, Score, ScoreWrapper, ScoreWrapperA, ScoreWrapperB,
    ScoreWrapperC, ScoreWrapperD,
};
use crate::plater::plate::Plate;
use crate::plater::plate_shape::PlateShape;
use crate::plater::request::Strategy;
use crate::plater::spiral::spiral_iterator;

use super::Placer;

impl<'a> Placer<'a> {
    pub fn place_unlocked_part<'b>(
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
        let rs = f64::ceil(PI * 2.0 / part.part.delta_r) as usize;

        let res =
            match self.request.algorithm.strategy {
                Strategy::PixelPack => Placer::pixel_place(self, rs, plate, &mut part),
                Strategy::SpiralPlace => Placer::spiral_place::<DefaultScoreWrapper>(
                    self,
                    rs,
                    &mut plate.clone(),
                    &mut part,
                )
                .or_else(|| {
                    Placer::spiral_place::<ScoreWrapperA>(self, rs, &mut plate.clone(), &mut part)
                })
                .or_else(|| {
                    Placer::spiral_place::<ScoreWrapperB>(self, rs, &mut plate.clone(), &mut part)
                })
                .or_else(|| {
                    Placer::spiral_place::<ScoreWrapperC>(self, rs, &mut plate.clone(), &mut part)
                })
                .or_else(|| {
                    Placer::spiral_place::<ScoreWrapperD>(self, rs, &mut plate.clone(), &mut part)
                }),
            };

        if let Some((better_x, better_y, better_r)) = res {
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
        }
    }

    fn pixel_place<'b>(
        &mut self,
        rs: usize,
        plate: &mut Plate<'b>,
        part: &mut PlacedPart<'b>,
    ) -> Option<(f64, f64, usize)> {
        let mut better_x = 0.0;
        let mut better_y = 0.0;
        let mut better_score = 0.0;
        let mut better_r = 0;
        let mut found = false;

        // Conditionally reverse iteration direction
        let make_rot_iter = || {
            if self.rotate_direction != 0 {
                itertools::Either::Left((0..rs).rev())
            } else {
                itertools::Either::Right(0..rs)
            }
        };

        let make_iter = || {
            (0..)
                .map(|x| (x as f64) * self.request.delta)
                .take_while(|x| *x < plate.width)
                .map(|x| {
                    (0..)
                        .map(|y| (y as f64) * self.request.delta)
                        .take_while(|y| *y < plate.height)
                        .map(move |y| (x, y))
                })
                .flatten()
                .map(|(x, y)| {
                    (
                        x + self.request.center_x - plate.width / 2.0,
                        y + self.request.center_y - plate.height / 2.0,
                    )
                })
        };

        for (x, y) in make_iter() {
            part.set_offset(x, y);
            for r in make_rot_iter() {
                let vr = (r + self.rotate_offset as usize) % rs;
                part.set_rotation(vr as i32);
                let score = {
                    let gx = part.get_gx() + x;
                    let gy = part.get_gy() + y;
                    gy * self.y_coef + gx * self.x_coef
                };

                if !found || score < better_score {
                    if plate.can_place(&part) {
                        found = true;
                        better_x = x;
                        better_y = y;
                        better_r = vr;
                        better_score = score;
                        // break 'outer;
                    }
                }
            }
        }

        if !found {
            None
        } else {
            Some((better_x, better_y, better_r))
        }
    }

    fn spiral_place<'b, T: ScoreWrapper>(
        &mut self,
        rs: usize,
        plate: &mut Plate<'b>,
        part: &mut PlacedPart<'b>,
    ) -> Option<(f64, f64, usize)> {
        let mut better_x = 0.0;
        let mut better_y = 0.0;
        let mut better_r = 0;
        let mut found = false;

        let mut better_score = T::from(Score {
            position: Outside,
            moment_of_inertial: FloatWrapper(f64::INFINITY),
            x_pos: FloatWrapper(f64::INFINITY),
            y_pos: FloatWrapper(f64::INFINITY),
        });

        // Conditionally reverse iteration direction
        let make_rot_iter = || {
            if self.rotate_direction != 0 {
                itertools::Either::Left((0..rs).rev())
            } else {
                itertools::Either::Right(0..rs)
            }
        };

        let initial_box = self.current_bounding_box.clone();

        let spiral = spiral_iterator(
            self.request.delta,
            plate.width,
            plate.height,
            self.request.plate_shape.width(),
            self.request.plate_shape.height(),
        )
        .map(|(x, y)| {
            (
                x + plate.center_x - plate.width / 2.0,
                y + plate.center_y - plate.height / 2.0,
            )
        });

        let cond = self.request.plate_shape.width() + (plate.center_x - plate.width / 2.0);

        for (x, y) in spiral {
            part.set_offset(x, y);
            for r in make_rot_iter() {
                let vr = (r + self.rotate_offset as usize) % rs;
                part.set_rotation(vr as i32);
                let cur_rect;
                let bmp = part.get_bitmap();

                let score = {
                    let w2 = bmp.width;
                    let h2 = bmp.height;
                    let (c2_x, c2_y) = (bmp.center_x, bmp.center_y);

                    let cur = Rect {
                        width: w2 as f64,
                        height: h2 as f64,
                        center_x: c2_x + x / self.request.precision,
                        center_y: c2_y + y / self.request.precision,
                    };

                    let merged = if let Some(r) = &initial_box {
                        r.combine(&cur)
                    } else {
                        cur.clone()
                    };

                    let position = if x > cond { Outside } else { Inside };

                    let moment_of_inertia =
                        f64::powf(merged.height, 2.0) + f64::powf(merged.width, 2.0);
                    cur_rect = Some(merged);

                    T::from(Score {
                        position,
                        moment_of_inertial: FloatWrapper(moment_of_inertia),
                        x_pos: FloatWrapper(x),
                        y_pos: FloatWrapper(y),
                    })
                };

                if !found || better_score.compare_prefer(score) == Second {
                    if plate.can_place(&part) {
                        found = true;
                        better_x = x;
                        better_y = y;
                        better_r = vr;
                        better_score = score;
                        self.current_bounding_box = cur_rect;
                    }
                }
            }
        }

        if !found {
            None
        } else {
            Some((better_x, better_y, better_r))
        }
    }
}
