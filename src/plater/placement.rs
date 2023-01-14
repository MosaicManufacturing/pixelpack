use crate::plater::point::Point;

#[derive(Debug)]
pub struct Placement {
    id: String,
    center: Point,
    rotation: f64,
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

impl Placement {
    pub fn new(id: String, center: Point, rotation: f64) -> Self {
        Placement {
            id,
            center,
            rotation,
        }
    }
    pub fn get_id(&self) -> String {
        self.id.to_string()
    }

    pub fn get_center(&self) -> Point {
        self.center.clone()
    }

    pub fn get_rotation(&self) -> f64 {
        self.rotation
    }
}
