use crate::plater::point::Point;

#[derive(Debug)]
pub struct Placement {
    pub id: String,
    pub center: Point,
    pub rotation: f64,
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
