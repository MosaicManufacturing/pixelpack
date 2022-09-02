pub struct Point {
    x: f64,
    y: f64
}

impl Point {
    pub(crate) fn new(x: f64, y: f64) -> Self {
        Point {x, y}
    }
}

impl Clone for Point {
    fn clone(&self) -> Self {
        Point::new(self.x, self.y)
    }
}