use crate::plater;
use crate::plater::point::Point;
use crate::plater::rectangle::Rectangle;
use crate::stl::util::get_side;

#[derive(Clone)]
pub struct Triangle2D {
    pub(crate) t_box: Rectangle,
    a: Point,
    b: Point,
    c: Point,
    ab: Point,
    bc: Point,
    ca: Point,
    nAB: Point,
    nBC: Point,
    nCA: Point,
}

impl Triangle2D {
    fn triangle_from_points(a: Point, b: Point, c: Point) -> Self {
        let ab = Point::sub(&b, &a);
        let bc = Point::sub(&c, &b);
        let ca = Point::sub(&a, &c);

        let nAB = ab.segment_normal();
        let nBC = bc.segment_normal();
        let nCA = ca.segment_normal();

        let t_box = Rectangle::new(
            f64::max(a.x, f64::max(b.x, c.x)),
            f64::max(a.y, f64::max(b.y, c.y)),
            f64::max(a.x, f64::max(b.x, c.x)),
            f64::max(a.y, f64::max(b.y, c.y)),
        );

        Triangle2D {
            a,
            b,
            c,
            ab,
            bc,
            ca,
            nAB,
            nBC,
            nCA,
            t_box,
        }
    }

    pub(crate) fn contains(&self, x: f64, y: f64) -> bool {
        let a = plater::point::Point::new(x - self.a.x, y - self.a.y);
        let b = plater::point::Point::new(x - self.b.x, y - self.b.y);
        let c = plater::point::Point::new(x - self.c.x, y - self.c.y);

        let side_a = get_side(&a, &self.nAB, &self.ab);
        let side_b = get_side(&b, &self.nBC, &self.bc);
        let side_c = get_side(&c, &self.nCA, &self.ca);

        side_a && side_b && side_c
    }


    pub(crate) fn contains_rect(&self, rect: &plater::rectangle::Rectangle) -> bool {
        self.contains(rect.x1, rect.y1) &&
            self.contains(rect.x1, rect.y2) &&
            self.contains(rect.x2, rect.y1) &&
            self.contains(rect.x2, rect.y2)
    }
}