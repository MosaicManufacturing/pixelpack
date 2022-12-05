use std::cmp::{max, min};
use crate::plater::bitmap::Bitmap;

pub trait PlateShape: Clone + Send + Sync {
    fn width(&self) -> f64;
    fn height(&self) -> f64;
    fn string(&self) -> String;
    fn mask_bitmap(&self, bitmap: &mut Bitmap, precision: f64);
    fn expand(&self, size: f64) -> Self;
    fn intersect_square(&self, size: f64) -> Option<Self>;
}

#[derive(Clone)]
pub enum Shape {
    Rectangle(PlateRectangle),
    Circle(PlateCircle),
}

impl Shape {
    pub fn new_rectangle(width: f64, height: f64, resolution: f64) -> Self {
        Shape::Rectangle(PlateRectangle::new(width, height, resolution))
    }

    pub fn new_circle(diameter: f64, resolution: f64) -> Self {
        Shape::Circle(PlateCircle::new(diameter, resolution))
    }
}

impl PlateShape for Shape {
    fn width(&self) -> f64 {
        match self {
            Shape::Rectangle(r) => PlateShape::width(r),
            Shape::Circle(c) => PlateShape::width(c),
        }
    }

    fn height(&self) -> f64 {
        match self {
            Shape::Rectangle(r) => PlateShape::height(r),
            Shape::Circle(c) => PlateShape::height(c),
        }
    }

    fn string(&self) -> String {
        match self {
            Shape::Rectangle(r) => PlateShape::string(r),
            Shape::Circle(c) => PlateShape::string(c),
        }
    }

    fn mask_bitmap(&self, bitmap: &mut Bitmap, precision: f64) {
        match self {
            Shape::Rectangle(r) => PlateShape::mask_bitmap(r, bitmap, precision),
            Shape::Circle(c) => PlateShape::mask_bitmap(c, bitmap, precision),
        }
    }

    fn expand(&self, size: f64) -> Self {
        match self {
            Shape::Rectangle(r) => Shape::Rectangle(PlateShape::expand(r, size)),
            Shape::Circle(c) => Shape::Circle(PlateShape::expand(c, size)),
        }
    }

    fn intersect_square(&self, size: f64) -> Option<Self> {
        match self {
            Shape::Rectangle(r) => r.intersect_square(size).map(|x| Shape::Rectangle(x)),
            Shape::Circle(c) => c.intersect_square(size).map(|x| Shape::Circle(x)),
        }
    }
}

// PlateRectangle represents a rectangular build plate.
#[derive(Clone)]
pub struct PlateRectangle {
    resolution: f64,
    width: f64,
    height: f64,
}

impl PlateRectangle {
    pub(crate) fn new(width: f64, height: f64, resolution: f64) -> Self {
        PlateRectangle {
            resolution,
            width: width * resolution,
            height: height * resolution,
        }
    }
}

impl PlateShape for PlateRectangle {
    fn width(&self) -> f64 {
        self.width
    }

    fn height(&self) -> f64 {
        self.height
    }

    fn string(&self) -> String {
        format!("{} x {} micron", self.width, self.height)
    }

    fn mask_bitmap(&self, _bitmap: &mut Bitmap, _precision: f64) {
        // no-op for rectangular piece
    }

    fn expand(&self, size: f64) -> Self {
        PlateRectangle::new(
            self.width / self.resolution + size,
            self.height / self.resolution + size,
            self.resolution,
        )
    }

    fn intersect_square(&self, size: f64) -> Option<Self> {

        if size <= 0.0 {
            return None;
        }

        let width = self.width / self.resolution;
        let height = self.height / self.resolution;



        Some(PlateRectangle::new(f64::min(size, width), f64::min(size, height), self.resolution))
    }
}

#[derive(Clone)]
pub struct PlateCircle {
    resolution: f64,
    diameter: f64,
}

impl PlateCircle {
    fn new(diameter: f64, resolution: f64) -> Self {
        PlateCircle {
            resolution,
            diameter: diameter * resolution,
        }
    }
}

impl PlateShape for PlateCircle {
    fn width(&self) -> f64 {
        self.diameter
    }

    fn height(&self) -> f64 {
        self.diameter
    }

    fn string(&self) -> String {
        format!("{} micron (circle)", self.diameter)
    }

    fn mask_bitmap(&self, bitmap: &mut Bitmap, precision: f64) {
        // fill all pixels outside plate radius so parts cannot be placed there
        let radius = self.diameter / 2.0;

        for y in 0..bitmap.height {
            for x in 0..bitmap.width {
                let dx = (x as f64 - bitmap.center_x) * precision;
                let dy = (y as f64 - bitmap.center_y) * precision;

                if f64::sqrt(dx * dx + dy * dy) > radius {
                    bitmap.set_point(x, y, 2)
                }
            }
        }
    }

    // Expand returns a new PlateCircle with the diameter
    // of the receiver increased by size.
    fn expand(&self, size: f64) -> Self {
        PlateCircle::new(self.diameter / self.resolution + size, self.resolution)
    }

    fn intersect_square(&self, size: f64) -> Option<Self> {
        let diameter= self.diameter / self.resolution - size;
        if diameter <= 0.0 {
            None
        } else {
            Some(PlateCircle::new(diameter, self.resolution))
        }
    }
}
