use std::sync::atomic::{AtomicUsize, Ordering};

use log::info;

use crate::plater::bitmap::Bitmap;
use crate::plater::placed_part::PlacedPart;
use crate::plater::placement::Placement;
use crate::plater::plate_shape::PlateShape;

static COUNTER: AtomicUsize = AtomicUsize::new(1);

fn generate_unique_plate_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone)]
pub struct Plate<'a> {
    pub(crate) plate_id: usize,
    pub(crate) width: f64,
    pub(crate) height: f64,
    precision: f64,
    pub(crate) parts: Vec<PlacedPart<'a>>,
    bitmap: Bitmap,
    center_x: f64,
    center_y: f64
}

impl<'a> Plate<'a> {
    pub(crate) fn new<S: PlateShape>(shape: &S, precision: f64, center_x: f64, center_y: f64) -> Self {
        let width = shape.width();
        let height = shape.height();

        Plate {
            plate_id: generate_unique_plate_id(),
            precision,
            width,
            height,
            parts: vec![],
            bitmap: Bitmap::new((width / precision) as i32, (height / precision) as i32),
            center_x,
            center_y
        }
    }

    pub(crate) fn make_from<S: PlateShape>(mut self, shape: &S, precision: f64) -> Self {
        let width = shape.width();
        let height = shape.height();

        self.width = width;
        self.height = height;
        self.bitmap = Bitmap::new((width / precision) as i32, (height / precision) as i32);

        let mut new_parts = Vec::with_capacity(self.parts.len());
        std::mem::swap(&mut new_parts, &mut self.parts);

        for part in new_parts {
            self.place(part);
        }

        self
    }

    pub fn make_plate_with_placed_parts<S: PlateShape>(
        shape: &S,
        precision: f64,
        placed_parts: &mut Vec<PlacedPart<'a>>,
        center_x: f64,
        center_y: f64
    ) -> Option<Self> {
        let mut plate = Self::new(shape, precision, center_x, center_y);

        for part in placed_parts.drain(..) {
            plate.place(part);
        }
        Some(plate)
    }

    pub(crate) fn place(&mut self, placed_part: PlacedPart<'a>) {
        let bitmap = placed_part.get_bitmap();
        // TODO: Scaling factor with precision is probably wrong
        let off_x = placed_part.get_x() / self.precision - (self.center_x - self.width/2.0);
        let off_y = placed_part.get_y() / self.precision - ( self.center_y - self.height/2.0);
        self.bitmap.write(bitmap, off_x as i32, off_y as i32);


        self.parts.push(placed_part);
    }

    pub(crate) fn can_contain(&self, placed_part: &PlacedPart) -> bool {
        let part_bmp = placed_part.get_bitmap();

        let x = placed_part.get_x();
        let y = placed_part.get_y();

        if (x + (part_bmp.width as f64) * self.precision) > self.width
            || (y + (part_bmp.height as f64) * self.precision) > self.height
        {
            return false;
        }

        return true;
    }


    pub(crate) fn can_place(&self, placed_part: &PlacedPart) -> bool {
        let part_bmp = placed_part.get_bitmap();

        // TODO: Scaling factor with precision is probably wrong
        let x = placed_part.get_x() - (self.center_x - self.width/2.0);
        let y = placed_part.get_y() - (self.center_y - self.height/2.0);

        if (x + (part_bmp.width as f64) * self.precision) > self.width
            || (y + (part_bmp.height as f64) * self.precision) > self.height
        {
            return false;
        }

        !part_bmp.overlaps(
            &self.bitmap,
            (x / self.precision) as i32,
            (y / self.precision) as i32,
        )
    }

    pub(crate) fn count_parts(&self) -> usize {
        (&self.parts).len()
    }

    pub fn get_placements(&self) -> Vec<Placement> {
        let mut result = vec![];
        info!("Parts len {}", self.parts.len());
        for part in &self.parts {
            result.push(part.get_placement());
        }

        result
    }

    pub fn get_ppm(&self) -> String {
        self.bitmap.to_ppm()
    }


    pub fn get_size(&self) -> (f64, f64) {
        (self.width, self.height)
    }
}
