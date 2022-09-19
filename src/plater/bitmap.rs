use std::cmp::{max, min};

use crate::plater::util;

pub struct Bitmap {
    // Image dimensions
    pub(crate) width: i32,
    pub(crate) height: i32,

    // center of the sprite
    pub(crate) center_x: f64,
    pub(crate) center_y: f64,

    // sum of the X, Y, and number of pixels
    pub(crate) s_x: i64,
    pub(crate) s_y: i64,
    pub(crate) pixels: i32,

    data: Vec<u8>,
}

impl Clone for Bitmap {
    fn clone(&self) -> Self {
        Bitmap {
            width: self.width,
            height: self.height,
            center_x: self.center_x,
            center_y: self.center_y,
            data: Vec::clone(&self.data),
            s_x: self.s_x,
            s_y: self.s_y,
            pixels: self.pixels,
        }
    }
}

impl Bitmap {
    pub fn initialize_data(&mut self, other: &Self) {
        self.data.copy_from_slice(other.data.as_slice());
    }


    pub(crate) fn new(width: i32, height: i32) -> Self {
        Bitmap {
            width,
            height,
            center_x: (width as f64) / 2.0,
            center_y: (height as f64) / 2.0,
            data: vec![0; (width * height) as usize],
            s_x: 0,
            s_y: 0,
            pixels: 0,
        }
    }

    // TODO: replace option with result
    fn new_bitmap_with_data(width: i32, height: i32, pixels: Vec<u8>) -> Option<Self> {
        if pixels.len() != (width * height) as usize {
            return None;
        }

        let bitmap = Bitmap {
            width,
            height,
            center_x: (width as f64) / 2.0,
            center_y: (height as f64) / 2.0,
            data: pixels,
            s_x: 0,
            s_y: 0,
            pixels: 0,
        };
        Some(bitmap)
    }

    fn index(&self, x: i32, y: i32) -> usize {
        (self.width * y + x) as usize
    }

    fn at(&self, x: i32, y: i32) -> u8 {
        self.data[self.index(x, y)]
    }

    pub(crate) fn to_ppm(&self) -> String {
        let EOL = '\n';
        let mut ppm = String::with_capacity(self.data.len());

        ppm.push_str("P2");
        ppm.push(EOL);
        ppm.push_str("# Generated by Plater - https://github.com/RobotsWar/Plater");
        ppm.push(EOL);
        ppm.push_str(format!("{} {}", self.width, self.height).as_str());
        ppm.push(EOL);
        ppm.push_str("6");
        ppm.push(EOL);

        let last_row_index = (self.width - 1) as usize;
        let casted_width = self.width as usize;
        for (index, byte) in (&self.data).iter().enumerate() {
            let color = match *byte {
                0 => '6',
                1 => '4',
                _ => '0',
            };
            ppm.push(color);
            let sep = if index % casted_width == last_row_index {
                EOL
            } else {
                ' '
            };
            ppm.push(sep)
        }
        ppm
    }

    fn get_point(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            0
        } else {
            self.at(x, y)
        }
    }

    pub(crate) fn set_point(&mut self, x: i32, y: i32, value: u8) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }

        if self.at(x, y) == value {
            return;
        }

        let index = self.index(x, y);
        self.data[index] = value;

        if value == 0 {
            self.s_x -= x as i64;
            self.s_y -= y as i64;
            self.pixels -= 1;
        } else {
            self.s_x += x as i64;
            self.s_y += y as i64;
            self.pixels += 1;
        }
    }

    pub(crate) fn dilate(&mut self, distance: i32) {
        let width = self.width as usize;
        let height = self.height as usize;

        let mut old_version = Bitmap::clone(self);

        for _ in 0..(distance as usize) {
            // This is equivalent to cloning self, but reusing an allocation
            old_version.initialize_data(self);
            for y in 0..height {
                for x in 0..width {
                    if old_version.at(x as i32, y as i32) == 0 {
                        let mut score = 0;
                        for dx in -1..1 {
                            for dy in -1..1 {
                                if dx == 0 && dy == 0 {
                                    continue;
                                }
                                if old_version.get_point((x as i32) + dx, (y as i32) + dy) != 0 {
                                    score += 1;
                                };
                            }
                        }

                        if score >= 1 {
                            self.set_point(x as i32, y as i32, 1);
                        }
                    }
                }
            }
        }


        // for _ in 0..distance {
        //     for index in 0..old.data.len() {
        //         let x = index % casted_width;
        //         let y = (index - x) / casted_height;
        //
        //         if old.at(x as i32, y as i32) == 0 {
        //             let mut score = 0;
        //             for dx in -1..1 {
        //                 for dy in -1..1 {
        //                     if dx == 0 && dy == 0 {
        //                         continue;
        //                     }
        //                     if old.get_point((x as i32) + dx, (y as i32) + dy) != 0 {
        //                         score += 1;
        //                     };
        //                 }
        //             }
        //
        //             if score >= 1 {
        //                 self.set_point(x as i32, y as i32, 1);
        //             }
        //         }
        //     }
        // }
    }

    // TODO: switch x and y cache
    pub(crate) fn overlaps(&self, other: &Bitmap, off_x: i32, off_y: i32) -> bool {
        for x in 0..self.width {
            for y in 0..self.height {
                if self.at(x, y) != 0 && other.get_point(x + off_x, y + off_y) != 0 {
                    return true;
                }
            }
        }

        false
    }

    // TODO: switch x and y cache
    pub(crate) fn write(&mut self, other: &Bitmap, off_x: i32, off_y: i32) {
        for x in 0..other.width {
            for y in 0..other.height {
                let pixel = other.at(x, y);
                if pixel != 0 {
                    self.set_point(x + off_x, y + off_y, pixel);
                }
            }
        }
    }

    pub(crate) fn rotate(&self, mut r: f64) -> Self {
        r = -r;

        let w = self.width as f64;
        let h = self.height as f64;

        let (a_x, a_y) = util::apply_rotation((w, h), r);
        let (b_x, b_y) = util::apply_rotation((0.0, h), r);
        let (c_x, c_y) = util::apply_rotation((w, 0.0), r);

        let x_min = min(min(0, a_x), min(b_x, c_x));
        let x_max = max(max(0, a_x), max(b_x, c_x));

        let y_min = min(min(0, a_y), min(b_y, c_y));
        let y_max = max(max(0, a_y), max(b_y, c_y));

        let width = x_max - x_min;
        let height = y_max - y_min;

        let old_center_x = self.center_x;
        let old_center_y = self.center_y;
        let center_x = (width / 2) as f64;
        let center_y = (height / 2) as f64;

        let mut rotated = Bitmap::new(width, height);

        // Removed casts for c_x & c_y
        for x in 0..width {
            for y in 0..height {
                let c_x = f64::round((x as f64) - center_x);
                let c_y = f64::round((y as f64) - center_y);
                let (X, Y) = util::apply_rotation_f64((c_x, c_y), r);
                rotated.set_point(
                    x,
                    y,
                    self.get_point(
                        f64::round(X + old_center_x) as i32,
                        f64::round(Y + old_center_y) as i32,
                    ),
                );
            }
        }
        rotated
    }

    pub(crate) fn trim(&self) -> Self {
        let mut found = false;
        let (mut min_x, mut min_y) = (0, 0);
        let (mut max_x, mut max_y) = (0, 0);

        // // swap x, y order
        for x in 0..self.width {
            for y in 0..self.height {
                if self.at(x, y) != 0 {
                    if !found {
                        found = true;
                        min_x = x;
                        max_x = x;
                        min_y = y;
                        max_y = y;
                    } else {
                        if x < min_x {
                            min_x = x
                        }
                        if y < min_y {
                            min_y = y
                        }
                        if x > max_x {
                            max_x = x
                        }
                        if y > max_y {
                            max_y = y
                        }
                    }
                }
            }
        }

        let delta_x = max_x - min_x;
        let delta_y = max_y - min_y;
        let mut trimmed = Bitmap::new(delta_x, delta_y);
        trimmed.center_x = self.center_x - min_x as f64;
        trimmed.center_y = self.center_y - min_y as f64;

        // switch x, y order
        for x in 0..delta_x {
            for y in 0..delta_y {
                trimmed.set_point(x, y, self.get_point(x + min_x, y + min_y));
            }
        }

        trimmed
    }

    fn grow(&self, dx: i32, dy: i32) -> Self {
        let new_width = self.width + (2 * dx);
        let new_height = self.height + (2 * dy);
        let mut expanded = Bitmap::new(new_width, new_height);

        expanded.center_x = self.center_x + dx as f64;
        expanded.center_y = self.center_y + dy as f64;

        // Switch x y order
        for x in 0..self.width {
            for y in 0..self.height {
                expanded.set_point(x + dx, y + dy, self.get_point(x, y));
            }
        }

        expanded
    }
}
