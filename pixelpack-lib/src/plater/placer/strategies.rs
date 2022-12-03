use crate::plater::placer;

use std::collections::HashMap;
use std::f64::consts::PI;
use log::info;
use crate::plater::placed_part::PlacedPart;
use crate::plater::placer::Rect;
use crate::plater::plate::Plate;
use crate::plater::plate_shape::PlateShape;
use crate::plater::request::{ConfigOrder, Strategy};
use crate::plater::spiral::spiral_iterator;


use super::Placer;

impl<'a, Shape: PlateShape> Placer<'a, Shape> {
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
        let rs = f64::ceil(PI * 2.0 / self.request.delta_r) as usize;

        //
        // let combined_iter;
        //
        // {
        //     let mut a = None;
        //     let mut b = None;
        //
        //     match self.request.algorithm.point_enumeration_mode {
        //         PointEnumerationMode::Row => {
        //           poin
        //         },
        //         PointEnumerationMode::Spiral => todo!()
        //     }
        // }
        //


        let res =  match self.request.algorithm.strategy {
            Strategy::PixelPack => Placer::pixel_place(self, rs, plate, &mut part),
            Strategy::SpiralPlace => Placer::spiral_place(self, rs, plate, &mut part)
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


    fn pixel_place<'b>(&mut self, rs: usize,plate: &mut Plate<'b>,
                        part: &mut PlacedPart<'b>) -> Option<(f64, f64, usize)> {

        let mut better_x = 0.0;
        let mut better_y = 0.0;
        let mut better_score = 0.0;
        let mut better_r = 0;
        let mut found = false;

        // Conditionally reverse iteration direction
        let make_rot_iter = || if self.rotate_direction != 0 {
            itertools::Either::Left((0..rs).rev())
        } else {
            itertools::Either::Right(0..rs)
        };

        let make_iter = || (0..)
            .map(|x| (x as f64) * self.request.delta)
            .take_while(|x| *x < plate.width)
            .map(|x| {
                (0..)
                    .map(|y| (y as f64) * self.request.delta)
                    .take_while(|y| *y < plate.height)
                    .map(move |y| (x, y))
            })
            .flatten()
            .map(|(x, y)|{
                (x + self.request.center_x - plate.width/2.0, y + self.request.center_y - plate.height/2.0)
        });



        for (x, y) in make_iter(){
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
                    if plate.can_place(&part)  {
                        // println!("Found {}", part.get_id());
                        found = true;
                        // info!("Placing");
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

    fn spiral_place<'b>(&mut self, rs: usize,plate: &mut Plate<'b>,
                        part: &mut PlacedPart<'b>) -> Option<(f64, f64, usize)> {

        let mut better_x = 0.0;
        let mut better_y = 0.0;
        let mut better_score = 0.0;
        let mut better_r = 0;
        let mut found = false;


        // Conditionally reverse iteration direction
        let make_rot_iter = || if self.rotate_direction != 0 {
            itertools::Either::Left((0..rs).rev())
        } else {
            itertools::Either::Right(0..rs)
        };

        let initial_box = self.current_bounding_box.clone();

        let spiral =
            spiral_iterator(self.request.delta, plate.width, plate.height)
                .map(|(x,y) | {
                    (x + self.request.center_x - plate.width/2.0, y + self.request.center_y - plate.height/2.0)
                });

        for (x, y) in spiral {
            part.set_offset(x, y);
            for r in make_rot_iter() {
                let vr = (r + self.rotate_offset as usize) % rs;
                part.set_rotation(vr as i32);
                let mut cur_rect = None;
                let bmp = part.get_bitmap();

                let score = {
                    let w2 = bmp.width;
                    let h2 = bmp.height;
                    let (c2_x, c2_y) = (bmp.center_x, bmp.center_y);

                    let cur = Rect {
                        width: w2 as f64,
                        height: h2 as f64,
                        center_x: c2_x + x/self.request.precision,
                        center_y: c2_y + y/self.request.precision,
                    };

                    let merged = if let Some(r) = &initial_box {
                        r.combine(&cur)
                    } else {
                        cur.clone()
                    };


                    let area = merged.height * merged.width;
                    cur_rect = Some(merged);
                    area
                };


                if !found || score < better_score {
                    if plate.can_place(&part)  {
                        found = true;
                        better_x = x;
                        better_y = y;
                        better_r = vr;
                        better_score = score;
                        self.current_bounding_box = cur_rect
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



}

