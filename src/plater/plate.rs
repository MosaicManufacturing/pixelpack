use std::sync::atomic::{AtomicUsize, Ordering};

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
    pub(crate) center_x: f64,
    pub(crate) center_y: f64,
}

impl<'a> Plate<'a> {
    pub(crate) fn align(&mut self, original_width: f64, original_height: f64) {
        let (bottom_space, top_space, left_space, right_space) = self.bitmap.get_bound();
        self.center();

        let centered_width = self.width - left_space - right_space;
        let tr_x = (original_width - centered_width) / 2.0;

        for part in &mut self.parts {
            let (x, y) = (part.get_x(), part.get_y());
            part.set_offset(x - tr_x, y);
        }
    }

    pub(crate) fn center(&mut self) {
        let (bottom_space, top_space, left_space, right_space) = self.bitmap.get_bound();
        let (width, height) = self.bitmap.get_dims();

        let new_ref = (left_space, bottom_space);
        let (new_width, new_height) = (width as f64 - left_space - right_space, height as f64 - bottom_space - top_space);
        let new_center = (new_ref.0 + new_width / 2.0, new_ref.1 + new_height / 2.0);

        let (tr_x, tr_y) = ((width as f64) / 2.0 - new_center.0, (height as f64) / 2.0 - new_center.1);

        for part in &mut self.parts {
            let (x, y) = (part.get_x(), part.get_y());
            part.set_offset(x + tr_x, y + tr_y);
        }
    }

    pub(crate) fn new(shape: &dyn PlateShape, precision: f64, center_x: f64, center_y: f64) -> Self {
        let width = shape.width();
        let height = shape.height();

        let mut bitmap = Bitmap::new((width / precision) as i32, (height / precision) as i32);
        shape.mask_bitmap(&mut bitmap, precision);

        Plate {
            plate_id: generate_unique_plate_id(),
            precision,
            width,
            height,
            parts: vec![],
            bitmap,
            center_x,
            center_y,
        }
    }

    pub(crate) fn make_from_shape(&mut self, shape: &dyn PlateShape) -> Self {
        let mut next_plate = Plate::new(shape
                                        , self.precision
                                        , self.center_x
                                        , self.center_y);

        let mut new_parts = Vec::with_capacity(self.parts.len());
        std::mem::swap(&mut new_parts, &mut self.parts);

        // Going to have to copy these bitmaps directly pixel by pixel
        for mut part in new_parts {
            let (x, y) = (part.get_x(), part.get_y());
            part.set_offset(x, y);

            next_plate.place(part);
        }
        next_plate
    }

    pub fn make_plate_with_placed_parts(
        shape: &dyn PlateShape,
        precision: f64,
        placed_parts: &mut Vec<PlacedPart<'a>>,
        center_x: f64,
        center_y: f64,
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
        let off_x = (placed_part.get_x() - (self.center_x - self.width / 2.0)) / self.precision;
        let off_y = (placed_part.get_y() - (self.center_y - self.height / 2.0)) / self.precision;
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
        let x = placed_part.get_x() - (self.center_x - self.width / 2.0);
        let y = placed_part.get_y() - (self.center_y - self.height / 2.0);

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
