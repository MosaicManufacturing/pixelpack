use crate::stl::point_3d::Point3D;
use crate::stl::quad_tree::QuadTree;
use crate::stl::triangle_2d::Triangle2D;
use crate::stl::volume::Volume;

pub struct Model {
    volumes: Vec<Volume>,
    tree: Option<Box<QuadTree>>,
    triangles: Vec<Triangle2D>,
}

impl Model {
    fn new() -> Self {
        Model {
            volumes: vec![],
            tree: None,
            triangles: vec![],
        }
    }

    fn min(&self) -> Point3D {
        (&self.volumes)
            .iter()
            .map(Volume::min)
            .reduce(|x, y| Point3D::min(&x, &y))
            .or_else(|| Some(Point3D::new(0.0, 0.0, 0.0)))
            .unwrap()
    }

    fn max(&self) -> Point3D {
        (&self.volumes)
            .iter()
            .map(Volume::max)
            .reduce(|x, y| Point3D::max(&x, &y))
            .or_else(|| Some(Point3D::new(0.0, 0.0, 0.0)))
            .unwrap()
    }
}