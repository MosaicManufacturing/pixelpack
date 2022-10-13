use crate::stl::point_3d::Point3D;

pub(crate) struct Face {
    pub(crate) v: [Point3D; 3],
}

impl Clone for Face {
    // Clone is a deep clone as points are immutable
    fn clone(&self) -> Self {
        let [x, y, z] = self.v;
        Face::new(x, y, z)
    }
}

impl Face {
    pub(crate) fn new(v0: Point3D, v1: Point3D, v2: Point3D) -> Self {
        Face { v: [v0, v1, v2] }
    }

    pub(crate) fn get_normal(&self) -> Point3D {
        let [a, b, c] = &self.v;
        let ab = Point3D::new(b.x - a.x, b.y - a.y, b.z - a.z);
        let ac = Point3D::new(c.x - a.x, c.y - a.y, c.z - a.z);
        let cross_product = Point3D::cross_product(&ab, &ac);
        cross_product.normalize()
    }
}
