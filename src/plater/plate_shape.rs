use crate::plater::bitmap::Bitmap;
use crate::plater::plate_shape::Shape::{Circle, Rectangle};

pub trait PlateShape: Send + Sync {
    fn resolution(&self) -> f64;
    fn width(&self) -> f64;
    fn height(&self) -> f64;
    fn string(&self) -> String;
    fn make_masked_bitmap(&self, precision: f64) -> Bitmap;
    fn extend_right(&self, size: f64) -> Box<dyn PlateShape>;
    fn dyn_clone(&self) -> Box<dyn PlateShape>;
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
        Circle(PlateCircle::new(diameter, resolution, 1.0))
    }

    pub fn new_rectangle(width: f64, height: f64, resolution: f64) -> Self {
        Rectangle(PlateRectangle::new(width, height, resolution))
    }

    pub fn width(&self) -> f64 {
        match self {
            Rectangle(r) => r.width(),
            Circle(c) => c.width(),
        }
    }

    pub fn height(&self) -> f64 {
        match self {
            Rectangle(r) => r.height(),
            Circle(c) => c.height(),
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

    fn make_masked_bitmap(&self, precision: f64) -> Bitmap {
        let width = self.width();
        let height = self.height();

        Bitmap::new((width / precision) as i32, (height / precision) as i32)
    }

    fn extend_right(&self, size: f64) -> Box<dyn PlateShape> {
        Box::new(PlateRectangle {
            resolution: self.resolution,
            width: self.width * size,
            height: self.height,
        })
    }

    fn dyn_clone(&self) -> Box<dyn PlateShape> {
        Box::new(PlateRectangle {
            resolution: self.resolution,
            width: self.width,
            height: self.height,
        })
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

        let rectangle = PlateRectangle::new(
            width / self.resolution,
            height / self.resolution,
            self.resolution,
        );
        Some(Box::new(rectangle))
    }
}

#[derive(Clone)]
pub struct PlateCircle {
    resolution: f64,
    diameter: f64,
    plate_expansion_factor: f64,
}

impl PlateCircle {
    pub fn new(diameter: f64, resolution: f64, plate_expansion_factor: f64) -> Self {
        PlateCircle {
            resolution,
            diameter: diameter * resolution,
            plate_expansion_factor,
        }
    }
}

fn make_standard_circle_bitmap(shape: &PlateCircle, precision: f64) -> Bitmap {
    let width = shape.width();
    let height = shape.height();

    let mut bitmap = Bitmap::new((width * shape.plate_expansion_factor / precision) as i32, (height / precision) as i32);
    // fill all pixels outside plate radius so parts cannot be placed there
    let radius = shape.diameter / 2.0;

    for y in 0..bitmap.height {
        for x in 0..bitmap.width {
            let dx = (x as f64 - bitmap.center_x) * precision;
            let dy = (y as f64 - bitmap.center_y) * precision;

            if f64::sqrt(dx * dx + dy * dy) > radius {
                bitmap.set_point(x, y, 2)
            }
        }
    }

    bitmap
}
impl PlateShape for PlateCircle {
    fn resolution(&self) -> f64 {
        self.resolution
    }

    fn width(&self) -> f64 {
        self.diameter * self.plate_expansion_factor
    }

    fn height(&self) -> f64 {
        self.diameter
    }

    fn string(&self) -> String {
        format!("{} micron (circle)", self.diameter)
    }

    fn make_masked_bitmap(&self, precision: f64) -> Bitmap {
        if self.plate_expansion_factor <= 1.0 {
            return make_standard_circle_bitmap(self, precision);
        }

        let regular = make_standard_circle_bitmap(&PlateCircle { diameter: self.diameter, resolution: self.resolution, plate_expansion_factor: 1.0 }, precision);

        let width = self.width();
        let height = self.height();

        let mut bitmap = Bitmap::new((width * self.plate_expansion_factor / precision) as i32, (height / precision) as i32);


        // Super-impose the normal-sized circle onto the expanded bitmap
        for y in 0..regular.height {
            for x in 0..regular.width {
                bitmap.set_point(x, y, regular.get_point(x, y))
            }
        }

        bitmap
    }

    // We return a rectangle when expanding a circle
    fn extend_right(&self, size: f64) -> Box<dyn PlateShape> {
        Box::new(PlateCircle {
            resolution: self.resolution,
            diameter: self.diameter,
            plate_expansion_factor: self.plate_expansion_factor * size,
        })
    }

    fn dyn_clone(&self) -> Box<dyn PlateShape> {
        Box::new(PlateCircle {
            resolution: self.resolution,
            diameter: self.diameter,
            plate_expansion_factor: self.plate_expansion_factor,
        })
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

        let circle = PlateCircle::new(width / self.resolution, self.resolution, self.plate_expansion_factor);
        Some(Box::new(circle))
    }
}
