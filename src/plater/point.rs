#[derive(Clone)]
pub struct Point {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl Point {
    pub(crate) fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    pub(crate) fn sub(a: &Self, b: &Self) -> Self {
        Point { x: a.x - b.x, y: a.y - b.y }
    }

    pub(crate) fn segment_normal(&self) -> Self {
        Point { x: self.y, y: -self.x }
    }
}
