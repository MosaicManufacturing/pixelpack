use crate::plater::point::Point;
use std::f64::consts::PI;

pub fn get_side(pt: &Point, n: &Point, s: &Point) -> bool {
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

// func formatPointForASCII(x, y, z float64) string {
// 	if x == -0 {
// 		x = 0
// 	}
// 	if y == -0 {
// 		y = 0
// 	}
// 	if z == -0 {
// 		z = 0
// 	}
// 	return fmt.Sprintf("%.6g %.6g %.6g", x, y, z)
// }
//
// func formatASCIINormal(normal Point3D) string {
// 	return formatPointForASCII(normal.X, normal.Y, normal.Z)
// }
//
// func formatASCIIVertex(v Point3D, resolution float64) string {
// 	return formatPointForASCII(v.X/resolution, v.Y/resolution, v.Z/resolution)
// }
