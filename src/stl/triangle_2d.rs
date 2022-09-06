// struct Triangle2D {
//     box: i32,
//     // a   plater.Point
//     // b   plater.Point
//     // c   plater.Point
//     // ab  plater.Point
//     // bc  plater.Point
//     // ca  plater.Point
//     // nAB plater.Point
//     // nBC plater.Point
//     // nCA plater.Point
// }

use crate::plater::rectangle::Rectangle;
use crate::plater::point::Point;

struct Triangle2D {
    t_box: Rectangle,
    a: Point,
    b: Point,
    c: Point,
    ab: Point,
    bc: Point,
    ca: Point,
    nAB: Point,
    nBC: Point,
    nCA: Point
}