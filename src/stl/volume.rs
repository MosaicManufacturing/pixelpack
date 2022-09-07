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

    fn min(&self) -> Point3D {
        // Memory overhead
        let (x, y, z) = (self.faces)
            .iter()
            .flat_map(|face| &face.v)
            .map(|x| (x.x, x.y, x.z))
            .reduce(|(x, y, z), (a, b, c)| {
                (f64::min(x, a), f64::min(y, b),f64::min(z, c))
            })
            .unwrap_or((0.0, 0.0, 0.0));

        Point3D::new(x, y, z)
    }

    fn max(&self) -> Point3D {
        // Memory overhead
        let (x, y, z) = (self.faces)
            .iter()
            .flat_map(|face| &face.v)
            .map(|x| (x.x, x.y, x.z))
            .reduce(|(x, y, z), (a, b, c)| {
                (f64::max(x, a), f64::max(y, b),f64::max(z, c))
            })
            .unwrap_or((0.0, 0.0, 0.0));

        Point3D::new(x, y, z)
    }


}

