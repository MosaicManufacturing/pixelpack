use std::sync::atomic::{AtomicUsize, Ordering};

use crate::plater::bitmap::Bitmap;
use crate::plater::placed_part::PlacedPart;
use crate::plater::placement::Placement;
use crate::plater::plate_shape::PlateShape;

static COUNTER: AtomicUsize = AtomicUsize::new(1);

fn generate_unique_plate_id() -> usize { COUNTER.fetch_add(1, Ordering::Relaxed) }

pub struct Plate<'a> {
    pub(crate) plate_id: usize,
    pub(crate) width: f64,
    pub(crate) height: f64,
    precision: f64,
    pub(crate) parts: Vec<PlacedPart<'a>>,
    bitmap: Bitmap,
}

impl<'a> Plate<'a> {
    pub(crate) fn new<Shape: PlateShape>(shape: &Shape, precision: f64) -> Self {
        let width = shape.width();
        let height = shape.height();

        Plate {
            plate_id: generate_unique_plate_id(),
            precision,
            width,
            height,
            parts: vec![],
            bitmap: Bitmap::new(((width / precision) as i32), (height / precision) as i32),
        }
    }

    pub(crate) fn place(&mut self, placed_part: PlacedPart<'a>) {
        {
            // let borrowed_placed_part = (*placed_part).borrow_mut();
            let bitmap = placed_part.get_bitmap().unwrap();
            let off_x = placed_part.get_x() / self.precision;
            let off_y = placed_part.get_y() / self.precision;
            self.bitmap.write(bitmap, off_x as i32, off_y as i32);
        }
        self.parts.push(placed_part);
    }

    pub(crate) fn can_place(&self, placed_part: &PlacedPart) -> bool {
        let part_bmp = placed_part.get_bitmap().unwrap();

        let x = placed_part.get_x();
        let y = placed_part.get_y();

        if (x + (part_bmp.width as f64) * self.precision) > self.width
            || (y + (part_bmp.height as f64) * self.precision) > self.height
        {
            return false;
        }

        !part_bmp.overlaps(
            &self.bitmap,
            ((x / self.precision) as i32),
            ((y / self.precision) as i32),
        )
    }

    pub(crate) fn count_parts(&self) -> usize {
        (&self.parts).len()
    }

    fn get_placements(&self) -> Vec<Placement> {
        let mut result = vec![];
        for part in &self.parts {
            result.push(part.get_placement());
        }

        result
    }

    fn get_ppm(&self) -> String {
        self.bitmap.to_ppm()
    }
}
