use crate::plater::bitmap::Bitmap;
use crate::plater::placed_part::PlacedPart;
use crate::plater::placement::Placement;
use crate::plater::plate_shape::PlateShape;

pub struct Plate {
    width: f64,
    height: f64,
    precision: f64,
    parts: Vec<PlacedPart>,
    bitmap: Bitmap
}

impl Plate {
    fn new(shape: &mut dyn PlateShape, precision: f64) -> Self {
        let width = shape.width();
        let height = shape.height();

        Plate {
            precision,
            width,
            height,
            parts: vec![],
            bitmap: Bitmap::new(((width / precision) as i32), (height / precision) as i32)
        }
    }

    fn place(&mut self, placed_part: PlacedPart) {
        let bitmap = placed_part.get_bitmap();
        let off_x = placed_part.get_x()/self.precision;
        let off_y = placed_part.get_y()/self.precision;
        self.bitmap.write(bitmap, off_x as i32, off_y as i32);
        self.parts.push(placed_part);
    }

    fn can_place(&self, placed_part: &PlacedPart) -> bool {
        let part_bmp = placed_part.get_bitmap();

        let x = placed_part.get_x();
        let y = placed_part.get_y();

        if (x+(part_bmp.width as f64)*self.precision) > self.width ||
            (y+(part_bmp.height as f64)*self.precision) > self.height {
            return false
        }

        !part_bmp.overlaps(&self.bitmap, ((x / self.precision) as i32), ((y / self.precision) as i32))
    }

    pub(crate) fn count_parts(&self) -> usize {
        (&self.parts).len()
    }

    fn get_placements(&self) -> Vec<Placement> {
        let mut result = vec![];
        for x in &self.parts {
            result.push(x.get_placement());
        }

        result
    }

    fn get_ppm(&self) -> String {
        self.bitmap.to_ppm()
    }




}