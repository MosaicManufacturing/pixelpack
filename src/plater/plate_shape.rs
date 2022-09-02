use crate::plater::bitmap::Bitmap;

pub trait PlateShape {
    fn width(&self) -> f64;
    fn height(&self) -> f64;
    fn string(&self) -> String;
    fn mask_bitmap(&self, bitmap: &mut Bitmap, precision: f64);
    fn expand(&self, size: f64) -> Box<dyn PlateShape>;
}

// PlateRectangle represents a rectangular build plate.
struct PlateRectangle {
    resolution: f64,
    width: f64,
    height: f64
}

impl PlateRectangle {
    fn new(width: f64, height: f64, resolution: f64) -> Self {
        PlateRectangle {
            resolution,
            width: width * resolution,
            height: height * resolution
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

    fn mask_bitmap(&self,_bitmap: &mut Bitmap ,  precision: f64) {
        // no-op for rectangular piece
    }

    fn expand(&self, size: f64) -> Box<dyn PlateShape>{
        Box::new(PlateRectangle::new(self.width + size,
                            self.height + size, self.resolution))
    }
}


struct PlateCircle {
    resolution: f64,
    diameter: f64,
}

impl PlateCircle {
    fn new(diameter: f64, resolution: f64) -> Self {
        PlateCircle {
            resolution,
            diameter: diameter * resolution
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

        for x in 0..bitmap.width {
            for y in 0..bitmap.height {
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
        Box::new(PlateCircle::new(self.diameter + size, self.resolution))
    }
}
