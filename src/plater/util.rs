use crate::plater::util;

pub(crate) fn apply_rotation_f64(point: (f64, f64), angle: f64) -> (f64, f64) {
    let x = point.0 * f64::cos(angle) - point.1 * f64::sin(angle);
    let y = point.0 * f64::sin(angle) + point.1 * f64::cos(angle);
    (x, y)
}

pub(crate) fn apply_rotation(point: (f64, f64), angle: f64) -> (i32, i32) {
    let (x, y) = util::apply_rotation_f64(point, angle);
    (f64::ceil(x) as i32, f64::ceil(y) as i32)
}
