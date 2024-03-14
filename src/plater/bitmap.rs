use std::cmp::{max, min};
use std::f64::consts::PI;

use crate::plater::util;

const NEIGHBORS: [(i32, i32); 9] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (0, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

fn parse_int(input: f64) -> Option<i32> {
    if (input - input.ceil()).abs() > 0.0001 {
        return None;
    }

    if i32::MIN as f64 <= input && input <= i32::MAX as f64 {
        return Some(input as i32);
    }

    return None;
}

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
    pub(crate) fn get_dims(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    pub(crate) fn initialize_data(&mut self, other: &Self) {
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

    pub fn rotate_90_clockwise(&self) -> Self {
        let width = self.height;
        let height = self.width;

        let center_y = self.center_x;
        let center_x = self.center_y;

        let pixels = self.pixels;

        let s_y = self.s_x;
        let s_x = self.s_y;

        let mut data = Vec::with_capacity((width * height) as usize);

        for x in 0..self.width {
            for y in (0..self.height).rev() {
                data.push(self.at(x, y));
            }
        }

        Bitmap {
            width,
            height,
            center_x,
            center_y,
            s_x,
            s_y,
            pixels,
            data,
        }
    }

    // TODO: replace option with result
    pub fn new_bitmap_with_data(width: i32, height: i32, pixels: &[u8]) -> Option<Self> {
        if pixels.len() != (width * height) as usize {
            return None;
        }

        let bitmap = Bitmap {
            width,
            height,
            center_x: (width as f64) / 2.0,
            center_y: (height as f64) / 2.0,
            data: pixels.to_vec(),
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

    pub fn to_ppm(&self) -> String {
        let eol = '\n';
        let mut ppm = String::with_capacity(self.data.len());

        ppm.push_str("P2");
        ppm.push(eol);
        ppm.push_str("# Generated by Plater - https://github.com/RobotsWar/Plater");
        ppm.push(eol);
        ppm.push_str(format!("{} {}", self.width, self.height).as_str());
        ppm.push(eol);
        ppm.push('6');
        ppm.push(eol);

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
                eol
            } else {
                ' '
            };
            ppm.push(sep)
        }
        ppm
    }

    pub(crate) fn get_point(&self, x: i32, y: i32) -> u8 {
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

    pub fn top_left_dilate(&mut self, distance: i32) {
        let width = self.width as usize;
        let height = self.height as usize;

        let mut old_version = Bitmap::clone(self);

        for _ in 0..(distance as usize) {
            // This is equivalent to cloning self, but reusing an allocation
            old_version.initialize_data(self);
            for y in 0..height {
                for x in 0..width {
                    if old_version.get_point(x as i32, y as i32) == 0 {
                        if old_version.get_point(x as i32 + 1, y as i32) != 0
                            || old_version.get_point(x as i32, y as i32 + 1) != 0
                            || old_version.get_point(x as i32 + 1, y as i32 + 1) != 0
                        {
                            self.set_point(x as i32, y as i32, 1);
                        }
                    }
                }
            }
        }
    }

    pub fn dilate(&mut self, distance: i32) {
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
                        for dx in -1..=1 {
                            for dy in -1..=1 {
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
    }

    // This only copies if other is fully contained in self
    #[allow(dead_code)]
    pub(crate) fn copy_from(&mut self, other: &Self, off_x: i32, off_y: i32) -> Option<()> {
        if !(other.width <= self.width && other.height <= self.height) {
            return None;
        }

        let src = other.data.as_slice();
        let dest = self.data.as_mut_slice();

        for i in 0..(other.height as usize) {
            let src_base_i = (other.width * i as i32) as usize;
            let src_slice = &(src)[src_base_i..src_base_i + other.width as usize];

            let dest_base_i = (off_x + (self.width * (i as i32 + off_y))) as usize;
            let dest_slice = &mut (dest)[dest_base_i..dest_base_i + other.width as usize];

            dest_slice.copy_from_slice(src_slice);
        }

        Some(())
    }

    pub(crate) fn overlaps(&self, other: &Bitmap, off_x: i32, off_y: i32) -> bool {
        let common_width = min(self.width, other.width - off_x) as usize;
        let common_height = min(self.height, other.height - off_y) as usize;

        let model_data = self.data.as_slice();
        let plate_data = other.data.as_slice();

        for i in 0..common_height {
            let model_base_i = self.width as usize * i;
            let model_slice = &(model_data)[model_base_i..model_base_i + common_width];
            let base_i = ((i as i32 + off_y) * other.width + off_x) as usize;
            let plate_slice = &(plate_data)[base_i..base_i + common_width];

            for (q, w) in model_slice.iter().zip(plate_slice.iter()) {
                if *q != 0 && *w != 0 {
                    return true;
                }
            }
        }

        // for y in 0..self.height {
        //     for x in 0..self.width {
        //         if self.at(x, y) != 0 && other.get_point(x + off_x, y + off_y) != 0 {
        //             return true;
        //         }
        //
        //     }
        // }

        false
    }

    pub(crate) fn write(&mut self, other: &Bitmap, off_x: i32, off_y: i32) {
        for y in 0..other.height {
            for x in 0..other.width {
                let pixel = other.at(x, y);
                if pixel != 0 {
                    self.set_point(x + off_x, y + off_y, pixel);
                }
            }
        }
    }

    pub(crate) fn rotate(&self, mut r: f64) -> Self {
        r = -r;

        // Apply special logic If the rotation angle is an integer multiple of pi/2 (90 degrees)
        let pi_over_2_factor = r / (PI / 2.0);
        match parse_int(pi_over_2_factor).map(|n| {
            // normalize values to range of [0, 2*pi)
            let value = n % 4;
            if value < 0 {
                4 + value
            } else {
                value
            }
        }) {
            Some(0) => {
                return self.clone();
            }
            Some(1) => {
                return self
                    .rotate_90_clockwise()
                    .rotate_90_clockwise()
                    .rotate_90_clockwise();
            }
            Some(2) => {
                return self.rotate_90_clockwise().rotate_90_clockwise();
            }
            Some(3) => {
                return self.rotate_90_clockwise();
            }
            _ => {}
        }

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
        let center_x = width as f64 / 2.0;
        let center_y = height as f64 / 2.0;

        let mut rotated = Bitmap::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let c_x = f64::round((x as f64) - center_x);
                let c_y = f64::round((y as f64) - center_y);
                let (x_1, y_1) = util::apply_rotation_f64((c_x, c_y), r);

                let center_x = f64::round(x_1 + old_center_x) as i32;
                let center_y = f64::round(y_1 + old_center_y) as i32;

                let max_of_neighbors = NEIGHBORS
                    .iter()
                    .map(|(off_x, off_y)| self.get_point(center_x + *off_x, center_y + *off_y))
                    .max()
                    .unwrap();

                rotated.set_point(x, y, max_of_neighbors);
            }
        }
        rotated
    }

    pub(crate) fn trim(&self) -> Self {
        let mut found = false;
        let (mut min_x, mut min_y) = (0, 0);
        let (mut max_x, mut max_y) = (0, 0);

        for y in 0..self.height {
            for x in 0..self.width {
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

        let delta_x = max_x - min_x + 1;
        let delta_y = max_y - min_y + 1;
        let mut trimmed = Bitmap::new(delta_x, delta_y);
        trimmed.center_x = self.center_x - min_x as f64;
        trimmed.center_y = self.center_y - min_y as f64;

        for y in 0..delta_y {
            for x in 0..delta_x {
                trimmed.set_point(x, y, self.get_point(x + min_x, y + min_y));
            }
        }

        trimmed
    }

    pub fn grow(&self, dx: i32, dy: i32) -> Self {
        let new_width = self.width + (2 * dx);
        let new_height = self.height + (2 * dy);
        let mut expanded = Bitmap::new(new_width, new_height);

        expanded.center_x = self.center_x + dx as f64;
        expanded.center_y = self.center_y + dy as f64;

        for y in 0..self.height {
            for x in 0..self.width {
                expanded.set_point(x + dx, y + dy, self.get_point(x, y));
            }
        }

        expanded
    }

    pub fn get_bound(&self) -> (f64, f64, f64, f64) {
        let (width, height) = self.get_dims();

        let bottom_space = {
            let mut counter = 0;
            'outer: for j in 0..height {
                for i in 0..width {
                    if self.get_point(i, j) != 0 {
                        break 'outer;
                    }
                }

                counter += 1;
            }

            counter as f64
        };

        let top_space = {
            let mut counter = 0;
            'outer: for j in (0..height).rev() {
                for i in 0..width {
                    if self.get_point(i, j) != 0 {
                        break 'outer;
                    }
                }

                counter += 1;
            }

            counter as f64
        };

        let left_space = {
            let mut counter = 0;
            'outer: for i in 0..width {
                for j in 0..height {
                    if self.get_point(i, j) != 0 {
                        break 'outer;
                    }
                }

                counter += 1;
            }
            counter as f64
        };

        let right_space = {
            let mut counter = 0;
            'outer: for i in (0..width).rev() {
                for j in 0..height {
                    if self.get_point(i, j) != 0 {
                        break 'outer;
                    }
                }

                counter += 1;
            }
            counter as f64
        };

        (bottom_space, top_space, left_space, right_space)
    }
}
