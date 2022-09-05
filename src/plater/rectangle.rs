struct Rectangle {
    // X1 is the minimum X coordinate of the box.
    x1: f64,
    // Y1 is the minimum Y coordinate of the box.
    y1: f64,
    // X2 is the maximum X coordinate of the box.
    x2: f64,
    // Y2 is the maximum Y coordinate of the box.
    y2: f64,
}

impl Rectangle {
    fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Rectangle { x1, x2, y1, y2 }
    }

    fn overlaps(&self, other: &Self) -> bool {
        (self.x1 <= other.x2 && self.x2 >= other.x1) && (self.y1 <= other.y2 && self.y2 >= other.y1)
    }

    fn contains(&self, x: f64, y: f64) -> bool {
        (self.x1 <= x && x <= self.x2) && (self.y1 <= y && y <= self.y2)
    }
}
