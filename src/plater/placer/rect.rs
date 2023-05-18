use core::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rect {
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) center_x: f64,
    pub(crate) center_y: f64,
}

impl Rect {
    pub(crate) fn combine(&self, other: &Self) -> Self {
        let top_height = f64::max(
            self.height / 2.0 + self.center_y,
            other.height / 2.0 + other.center_y,
        );
        let bottom_height = f64::min(
            -self.height / 2.0 + self.center_y,
            -other.height / 2.0 + other.center_y,
        );

        let left_width = f64::min(
            -self.width / 2.0 + self.center_x,
            -other.width / 2.0 + other.center_x,
        );
        let right_width = f64::max(
            self.width / 2.0 + self.center_x,
            other.width / 2.0 + other.center_x,
        );

        Rect {
            width: right_width - left_width,
            height: top_height - bottom_height,
            center_x: (right_width + left_width) / 2.0,
            center_y: (top_height + bottom_height) / 2.0,
        }
    }

    fn get_points(&self) -> [(f64, f64); 4] {
        let w2 = self.width / 2.0;
        let h2 = self.height / 2.0;
        [
            (self.center_x + w2, self.center_y + h2),
            (self.center_x - w2, self.center_y + h2),
            (self.center_x + w2, self.center_y - h2),
            (self.center_x - w2, self.center_y - h2),
        ]
    }
}

impl PartialOrd for Rect {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        f64::partial_cmp(&(self.width * self.height), &(other.width * other.height))
    }
}
