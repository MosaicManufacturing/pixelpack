use crate::stl::face::Face;
use crate::stl::point_3d::Point3D;

pub struct Volume {
    pub(crate) faces: Vec<Face>,
}

impl Clone for Volume {
    fn clone(&self) -> Self {
        Volume {
            faces: (&self.faces).iter().map(Clone::clone).collect(),
        }
    }
}

impl Volume {
    pub(crate) fn new() -> Self {
        Volume { faces: vec![] }
    }

    pub(crate) fn add_face(&mut self, f: Face) {
        self.faces.push(f);
    }

    fn reduce_faces_with(&self, f: impl Fn(Point3D, Point3D) -> Point3D) -> Point3D {
        (&self.faces)
            .iter()
            .flat_map(|face| &face.v)
            .map(Clone::clone)
            .reduce(f)
            .or_else(|| Some(Point3D::new(0.0, 0.0, 0.0)))
            .unwrap()
    }

    pub(crate) fn min(&self) -> Point3D {
        self.reduce_faces_with(|x, y| Point3D::min(&x, &y))
    }

    pub(crate) fn max(&self) -> Point3D {
        self.reduce_faces_with(|x, y| Point3D::max(&x, &y))
    }
}
