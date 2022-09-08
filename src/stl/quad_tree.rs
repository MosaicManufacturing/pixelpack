use crate::plater::rectangle::Rectangle;
use crate::stl::triangle_2d::Triangle2D;

pub struct QuadTree {
    triangles: Vec<Triangle2D>,
    depth: usize,
    r: Rectangle,
    quad1: Option<Box<QuadTree>>,
    quad2: Option<Box<QuadTree>>,
    quad3: Option<Box<QuadTree>>,
    quad4: Option<Box<QuadTree>>,
    black: bool,
}

impl QuadTree {
    fn new(x1: f64, y1: f64, x2: f64, y2: f64, depth: usize) -> Self {
        let r = Rectangle::new(x1, y1, x2, y2);
        let mut q = QuadTree {
            triangles: vec![],
            depth,
            r,
            quad1: None,
            quad2: None,
            quad3: None,
            quad4: None,
            black: false,
        };

        if depth > 0 {
            let xm = (x1 + x2) / 2.0;
            let ym = (y1 + y2) / 2.0;
            q.quad1 = Some(Box::new(QuadTree::new(x1, y1, xm, ym, depth - 1)));
            q.quad2 = Some(Box::new(QuadTree::new(xm, y1, x2, ym, depth - 1)));
            q.quad3 = Some(Box::new(QuadTree::new(x1, ym, xm, y2, depth - 1)));
            q.quad4 = Some(Box::new(QuadTree::new(xm, ym, x2, y2, depth - 1)));
        }

        q
    }

    fn add(&mut self, triangle: Triangle2D) {
        if self.depth == 0 {
            self.triangles.push(triangle);
            return;
        }

        if self.black {
            return;
        }

        if triangle.contains_rect(&self.r) {
            self.black = true;
            self.quad1 = None;
            self.quad2 = None;
            self.quad3 = None;
            self.quad4 = None;
            return;
        }

        // Maybe don't clone and use Rc instead
        if triangle.t_box.overlaps(&self.r) {
            if let Some(x) = self.quad1.as_mut() {
                x.add(Clone::clone(&triangle));
            }

            if let Some(x) = self.quad2.as_mut() {
                x.add(Clone::clone(&triangle));
            }

            if let Some(x) = self.quad3.as_mut() {
                x.add(Clone::clone(&triangle));
            }

            if let Some(x) = self.quad4.as_mut() {
                x.add(triangle);
            }
        }
    }
}
