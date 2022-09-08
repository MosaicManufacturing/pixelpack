// Maybe revisit to make this more explicit

#[derive(Clone, Copy)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}


impl Point3D {
    pub(crate) fn new(x: f64, y: f64, z: f64) -> Self {
        Point3D { x, y, z }
    }

    fn length(&self) -> f64 {
        f64::sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
    }

    pub(crate) fn normalize(&self) -> Self {
        let length = self.length();
        Point3D { x: self.x / length, y: self.y / length, z: self.z / length }
    }

    fn dot_product(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.x * other.z
    }

    pub(crate) fn cross_product(&self, other: &Self) -> Self {
        let x = self.y * other.z - self.z * other.y;
        let y = self.z * other.x - self.x * other.z;
        let z = self.x * other.y - self.y * other.x;
        Point3D { x, y, z }
    }

    pub fn min(a: &Self, b: &Self) -> Self {
        Point3D {
            x: f64::min(a.x, b.x)
            ,
            y: f64::min(a.y, b.y)
            ,
            z: f64::min(a.z, b.z),
        }
    }

    pub fn max(a: &Self, b: &Self) -> Self {
        Point3D {
            x: f64::max(a.x, b.x)
            ,
            y: f64::max(a.y, b.y)
            ,
            z: f64::max(a.z, b.z),
        }
    }
}