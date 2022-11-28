use crate::plater::bitmap::Bitmap;
use crate::plater::part::Part;
use crate::plater::placement::Placement;
use crate::plater::point::Point;

#[derive(Clone)]
pub struct PlacedPart<'a> {
    pub(crate) part: &'a Part,
    x: f64,
    y: f64,
    rotation: i32,
    pub(crate) insertion_index: usize,
}

impl<'a> PlacedPart<'a> {
    pub(crate) fn new_placed_part(part: &Part) -> PlacedPart {
        PlacedPart {
            part,
            x: 0.0,
            y: 0.0,
            rotation: 0,
            insertion_index: 0,
        }
    }

    // get_id returns the ID of the underlying Part.
    pub(crate) fn get_id(&self) -> &str {
        self.part.get_id()
    }

    pub(crate) fn set_offset(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }

    pub(crate) fn set_rotation(&mut self, r: i32) {
        self.rotation = r;
    }

    fn get_part(&self) -> &Part {
        self.part
    }

    pub(crate) fn get_x(&self) -> f64 {
        self.x
    }

    pub(crate) fn get_y(&self) -> f64 {
        self.y
    }

    pub(crate) fn get_bitmap(&self) -> &Bitmap {
        self.part.get_bitmap(self.rotation as usize)
    }

    fn get_center_x(&self) -> f64 {
        self.x + self.part.precision * self.get_bitmap().center_x
    }

    fn get_center_y(&self) -> f64 {
        self.y + self.part.precision * self.get_bitmap().center_y
    }

    // get_rotation returns the rotation about the Z axis at the Part's center
    // point as placed, in radians.
    fn get_rotation(&self) -> f64 {
        (self.rotation as f64) * self.part.delta_r
    }

    pub(crate) fn get_surface(&self) -> f64 {
        self.part.get_surface()
    }

    fn get_g_dist(&self) -> f64 {
        let mut has_score = false;
        let mut score = 0.0;

        for bmp in &self.part.bitmaps {
            // TODO: this really needs to filter out plates that don't fit, PlacedPart, Plate
                let g_x = (bmp.s_x as f64) / (bmp.pixels as f64);
                let g_y = (bmp.s_y as f64) / (bmp.pixels as f64);
                let s = g_x * g_x + g_y * g_y;
                if !has_score || s < score {
                    score = s;
                    has_score = true;
                }
            }


        score
    }

    pub(crate) fn get_gx(&self) -> f64 {
        let bmp = self.get_bitmap();
        (bmp.s_x as f64 / bmp.pixels as f64) * self.part.precision
    }

    pub(crate) fn get_gy(&self) -> f64 {
        let bmp = self.get_bitmap();
        (bmp.s_y as f64 / bmp.pixels as f64) * self.part.precision
    }

    pub(crate) fn get_placement(&self) -> Placement {
        Placement {
            id: self.get_id().to_owned(),
            center: Point::new(self.get_center_x(), self.get_center_y()),
            rotation: self.get_rotation(),
        }
    }
}
