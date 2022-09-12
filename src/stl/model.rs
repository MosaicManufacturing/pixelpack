use crate::plater;
use crate::plater::point::Point;
use crate::plater::bitmap::Bitmap;
use crate::stl::orientation::Orientation;
use crate::stl::point_3d::Point3D;
use crate::stl::quad_tree::QuadTree;
use crate::stl::triangle_2d::Triangle2D;
use crate::stl::util::deg_to_rad;
use crate::stl::volume::Volume;

pub struct Model {
    volumes: Vec<Volume>,
    tree: Option<Box<QuadTree>>,
    // triangles: Vec<Triangle2D>, No need for this right now
}

impl Clone for Model {
    fn clone(&self) -> Self {
        Model {
            volumes: self.volumes.iter().map(|x| x.clone()).collect(),
            tree: None,
        }
    }
}

impl Model {
    fn new() -> Self {
        Model {
            volumes: vec![],
            tree: None,
            // triangles: vec![],
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

    fn initialize_quad_tree(&mut self) {
        let min_p = self.min();
        let max_p = self.max();

        let mut tree = QuadTree::new(min_p.x, min_p.y, max_p.x, max_p.y, 6);

        (&self.volumes)
            .iter()
            .flat_map(|x| &x.faces)
            .map(|face| &face.v)
            .map(|[p1, p2, p3]| {
               Triangle2D::triangle_from_points(Point::new(p1.x, p1.y), Point::new(p2.x, p2.y), Point::new(p3.x, p3.y))
            })
            .for_each(|x| tree.add(x));

        self.tree = Some(Box::new(tree));
    }

    fn contains(&mut self, x: f64, y: f64) -> bool {
        if self.tree.is_none() {
            self.initialize_quad_tree();
        }

        self
            .tree
            .as_ref()
            .unwrap()
            .test(x, y)
    }

    fn pixelize(&mut self, precision: f64, dilation: f64) -> Bitmap {
        let min_p = self.min();
        let max_p = self.max();

        let x_min = min_p.x - dilation;
        let y_min = min_p.y - dilation;

        let x_max = max_p.x + dilation;
        let y_max = max_p.y + dilation;

        let width = ((x_max - x_min)/precision) as i32;
        let height = ((y_max - y_min)/precision) as i32;

        let mut bitmap = Bitmap::new(width, height);

        for x in 0..width  {
            for y in 0..height {
                let X = (x+1) as f64 * precision - dilation + min_p.x;
                let Y = (y+1) as f64 * precision - dilation + min_p.y;

                let value =
                    if min_p.x < X &&
                        X < max_p.x &&
                        min_p.y < Y &&
                        Y < max_p.y  &&
                        self.contains(X, Y){
                    2
                } else {
                    0
                };

                bitmap.set_point(x, y, value);
            }
        }

        bitmap.dilate((dilation/precision) as i32);
        bitmap
    }

    fn clone_model_with_point_transform(&self, transform_point: impl Fn(&mut Point3D)) -> Self {
        let mut cloned = self.clone();
        cloned.volumes
            .iter_mut()
            .flat_map(|x| &mut x.faces)
            .flat_map(|face| &mut face.v)
            .for_each(transform_point);
        cloned
    }

    fn rotate_z(&self, r: f64) -> Self {
        self.clone_model_with_point_transform(|Point3D {x, y, ..}|{
            let (x_, y_) = plater::util::apply_rotation_f64((*x, *y), r);
            *x = x_;
            *y = y_;
        })
    }

    fn rotate_y(&self, r: f64) -> Self {
        self.clone_model_with_point_transform(|Point3D {x, z, ..}|{
            let (x_, z_) = plater::util::apply_rotation_f64((*x, *z), r);
            *x = x_;
            *z = z_;
        })
    }

    fn rotate_x(&self, r: f64) -> Self {
        self.clone_model_with_point_transform(|Point3D {y, z, ..}|{
            let (y_, z_) = plater::util::apply_rotation_f64((*y, *z), r);
            *y = y_;
            *z = z_;
        })
    }

    fn translate(&self, x1: f64, y1: f64, z1: f64) -> Self {
        self.clone_model_with_point_transform(|Point3D{x, y, z}|{
            *x += x1;
            *y += y1;
            *z += z1;
        })
    }

    fn center(&self) -> Self {
        let min_p = self.min();
        let max_p = self.max();

        let x = (min_p.x + max_p.x)/2.0;
        let y = (min_p.y + max_p.y)/2.0;
        let z = min_p.z;

        self.translate(-x, -y, -z)
    }

    fn put_face_on_plate(&self, orientation: Orientation) -> Self {
        match orientation {
            Orientation::Bottom => {self.clone()}
            Orientation::Top => {self.rotate_x(deg_to_rad(180.0))}
            Orientation::Front => {self.rotate_x(deg_to_rad(90.0))}
            Orientation::Back => {self.rotate_x(deg_to_rad(270.0))}
            Orientation::Left => {self.rotate_y(deg_to_rad(90.0))}
            Orientation::Right => {self.rotate_y(deg_to_rad(-90.0))}
        }
    }
}