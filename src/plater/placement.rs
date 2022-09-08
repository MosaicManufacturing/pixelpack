use crate::plater::point::Point;

pub struct Placement {
    pub(crate) id: String,
    pub(crate) center: Point,
    pub(crate) rotation: f64,
}

// TODO: replace clone with Rc<Placement> as optimization
impl Clone for Placement {
    fn clone(&self) -> Self {
        Placement {
            id: self.id.to_owned(),
            center: Point::clone(&self.center),
            rotation: self.rotation,
        }
    }
}
