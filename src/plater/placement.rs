use crate::plater::point::Point;

pub(crate) struct Placement {
    pub(crate) id: String,
    pub(crate) center: Point,
    pub(crate) rotation: f64,
}

impl Clone for Placement {
    fn clone(&self) -> Self {
        Placement {
            id: self.id.to_owned(),
            center: Point::clone(&self.center),
            rotation: self.rotation,
        }
    }
}
