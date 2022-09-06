use crate::stl::point_3d::Point3D;

pub struct Face {
    pub(crate) v: [Point3D; 3]
}

impl Clone for Face {
    // Clone is a deep clone as points are immutable
    fn clone(&self) -> Self {
        let [x, y, z] = self.v;
        Face::new(x, y, z)
    }
}

impl Face {
    fn new(v0: Point3D, v1: Point3D, v2: Point3D) -> Self {
        Face {v: [v0, v1, v2]}
    }

    fn get_normal(&self) -> Point3D {
        let [a, b, c] = &self.v;
        let ab = Point3D::new(b.x - a.x,b.y - a.y, b.z - a.z);
        let ac = Point3D::new(c.x - a.x,c.y - a.y, c.z - a.z);
        let cross_product = Point3D::cross_product(&ab, &ac);
        cross_product.normalize()
    }
}



// // Clone returns a deeply-copied Face.
// func (f Face) Clone() Face {
// 	return NewFace(f.V[0], f.V[1], f.V[2])
// }
//
// // GetNormal returns the normal vector of the Face.
// func (f Face) GetNormal() Point3D {
// 	a, b, c := f.V[0], f.V[1], f.V[2]
// 	ab := NewPoint3D(b.X-a.X, b.Y-a.Y, b.Z-a.Z)
// 	ac := NewPoint3D(c.X-a.X, c.Y-a.Y, c.Z-a.Z)
// 	return CrossProduct(ab, ac).Normalize()
// }