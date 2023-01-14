use std::f64::consts::PI;

use crate::plater::point::Point;

pub(crate) fn get_side(pt: &Point, n: &Point, s: &Point) -> bool {
    let scalar_n = n.x * pt.x + n.y * pt.y;
    if scalar_n == 0.0 {
        s.x * pt.x + s.y * pt.y > 0.0
    } else {
        scalar_n < 0.0
    }
}

pub fn deg_to_rad(x: f64) -> f64 {
    PI * x / 180.0
}

pub fn rad_to_deg(x: f64) -> f64 {
    180.0 * x / PI
}
