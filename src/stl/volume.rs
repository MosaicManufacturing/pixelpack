use crate::stl::face::Face;
use crate::stl::point_3d::Point3D;

pub struct Volume {
    faces: Vec<Face>
}

impl Clone for Volume {
    fn clone(&self) -> Self {
        Volume {
            faces: (&self.faces).iter().map(Clone::clone).collect()
        }
    }
}

impl Volume {
    fn new() -> Self {
        Volume {
            faces: vec![]
        }
    }

    fn add_face(&mut self, f: Face) {
        self.faces.push(f);
    }

    pub fn min(&self) -> Point3D {
         (&self.faces)
            .iter()
            .flat_map(|face| &face.v)
            .map(Clone::clone)
            .reduce(|x, y | Point3D::min(&x, &y))
            .or_else(|| Some(Point3D::new(0.0, 0.0, 0.0)))
            .unwrap()
    }

    pub(crate) fn max(&self) -> Point3D {
        (&self.faces)
            .iter()
            .flat_map(|face| &face.v)
            .map(Clone::clone)
            .reduce(|x, y | Point3D::max(&x, &y))
            .or_else(|| Some(Point3D::new(0.0, 0.0, 0.0)))
            .unwrap()
    }


}

