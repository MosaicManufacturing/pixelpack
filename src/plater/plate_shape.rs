use crate::plater::bitmap::Bitmap;
use crate::plater::plate_shape::Shape::{Circle, Rectangle};

pub trait PlateShape: Send + Sync {
    fn resolution(&self) -> f64;
    fn width(&self) -> f64;
    fn height(&self) -> f64;
    fn string(&self) -> String;
    fn mask_bitmap(&self, bitmap: &mut Bitmap, precision: f64);
    fn expand(&self, size: f64) -> Box<dyn PlateShape>;
    fn dyn_clone(&self) -> Box<dyn PlateShape>;
    fn intersect_square(&self, size: f64) -> Option<Box<dyn PlateShape>>;
    fn contract(&self, size: f64) -> Option<Box<dyn PlateShape>>;
}

impl Clone for Box<dyn PlateShape> {
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}

// PlateRectangle represents a rectangular build plate.
#[derive(Clone)]
pub struct PlateRectangle {
    resolution: f64,
    width: f64,
    height: f64,
}

pub enum Shape {
    Rectangle(PlateRectangle),
    Circle(PlateCircle),
}

impl Shape {
    pub fn new_circle(diameter: f64, resolution: f64) -> Self {
        Circle(PlateCircle::new(diameter, resolution))
    }

    pub fn new_rectangle(width: f64, height: f64, resolution: f64) -> Self {
        Rectangle(PlateRectangle::new(width, height, resolution))
    }

    pub fn width(&self) -> f64 {
        match self {
            Rectangle(r) => r.width(),
            Circle(c) => c.width()
        }
    }

    pub fn height(&self) -> f64 {
        match self {
            Rectangle(r) => r.height(),
            Circle(c) => c.height()
        }
    }
}

impl PlateRectangle {
    pub fn new(width: f64, height: f64, resolution: f64) -> Self {
        PlateRectangle {
            resolution,
            width: width * resolution,
            height: height * resolution,
        }
    }
}

impl PlateShape for PlateRectangle {
    fn resolution(&self) -> f64 {
        self.resolution
    }

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

    fn expand(&self, size: f64) -> Box<dyn PlateShape> {
        Box::new(PlateRectangle::new(
            self.width / self.resolution + size,
            self.height / self.resolution,
            self.resolution,
        ))
    }

    fn dyn_clone(&self) -> Box<dyn PlateShape> {
        Box::new(PlateRectangle {
            resolution: self.resolution,
            width: self.width,
            height: self.height,
        })
    }

    fn intersect_square(&self, size: f64) -> Option<Box<dyn PlateShape>> {
        if size <= 0.0 {
            return None;
        }

        let width = self.width / self.resolution;
        let height = self.height / self.resolution;


        Some(Box::new(PlateRectangle::new(f64::min(size, width), f64::min(size, height), self.resolution)))
    }

    fn contract(&self, size: f64) -> Option<Box<dyn PlateShape>> {
        if size <= 0.0 {
            return None;
        }

        let width = self.width - size * self.resolution;

        if width <= 0.0 {
            return None;
        }

        let height = self.height * (width / self.width);

        let rectangle = PlateRectangle::new(width / self.resolution, height / self.resolution, self.resolution);
        Some(Box::new(rectangle))
    }
}

#[derive(Clone)]
pub struct PlateCircle {
    resolution: f64,
    diameter: f64,
}

impl PlateCircle {
    pub fn new(diameter: f64, resolution: f64) -> Self {
        PlateCircle {
            resolution,
            diameter: diameter * resolution,
        }
    }
}

impl PlateShape for PlateCircle {
    fn resolution(&self) -> f64 {
        self.resolution
    }

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
    fn expand(&self, size: f64) -> Box<dyn PlateShape> {
        Box::new(PlateCircle::new(self.diameter / self.resolution + size, self.resolution))
    }

    fn dyn_clone(&self) -> Box<dyn PlateShape> {
        Box::new(PlateCircle {
            resolution: self.resolution,
            diameter: self.diameter,
        })
    }

    fn intersect_square(&self, size: f64) -> Option<Box<dyn PlateShape>> {
        if size <= 0.0 {
            return None;
        }

        let diameter = self.diameter / self.resolution - size;
        if diameter <= 0.0 {
            None
        } else {
            Some(Box::new(PlateCircle::new(diameter, self.resolution)))
        }
    }

    // Also mask bitmap
    fn contract(&self, size: f64) -> Option<Box<dyn PlateShape>> {
        if size <= 0.0 {
            return None;
        }

        let width = self.width() - size * self.resolution;

        if width <= 0.0 {
            return None;
        }

        let circle = PlateCircle::new(width / self.resolution, self.resolution);
        Some(Box::new(circle))
    }
}
