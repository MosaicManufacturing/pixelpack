use std::f64::consts::PI;

use crate::plater::bitmap::Bitmap;

pub struct Part {
    pub(crate) locked: bool,
    // if true, part cannot be moved or rotated
    pub(crate) id: String,
    pub(crate) precision: f64,
    pub(crate) delta_r: f64,
    _width: f64,
    _height: f64,
    pub(crate) center_x: f64,
    pub(crate) center_y: f64,
    surface: f64,
    // average bitmap size
    pub(crate) bitmaps: Vec<Bitmap>,
}

impl Part {
    pub fn new(
        id: String,
        bitmap: Bitmap,
        center_x: f64,
        center_y: f64,
        precision: f64,
        delta_r: f64,
        spacing: f64,
        _plate_width: f64,
        _plate_height: f64,
        locked: bool,
    ) -> Self {
        let mut num_bitmaps = f64::ceil(PI * 2.0 / delta_r) as i32;
        if locked {
            num_bitmaps = 1;
        }

        let (width, height) = bitmap.get_dims();

        // Improvement, we currently only use a rotation if it fits within the original plate

        // if for every model there exists a rotation that is contained within,
        // we may attempt to place the model

        let bitmaps = if locked {
            vec![bitmap]
        } else {
            (0..num_bitmaps as usize)
                .into_iter()
                .map(|k|
                    bitmap.rotate((k as f64) * delta_r).trim()
                )
                .collect()
        };

        let mut p = Part {
            precision,
            delta_r,
            id,
            locked,
            bitmaps,
            center_y,
            center_x,
            _width: width as f64 + 2.0 * spacing,
            _height: height as f64 + 2.0 * spacing,
            surface: 0.0,
        };

        let mut correct = 0;

        for k in 0..num_bitmaps as usize {
            let Bitmap { width, height, .. } = &p.bitmaps[k];

            p.surface += (width * height) as f64;
            correct += 1;
            //
            // if *width as f64 * precision < plate_width && *height as f64 * precision < plate_height
            // {
            //     p.surface += (width * height) as f64;
            //     correct += 1;
            // } else {
            //     p.bitmaps[k] = None;
            // }
        }


        p.surface /= correct as f64;
        p
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.id
    }

    #[allow(dead_code)]
    fn get_rotation(&self) -> f64 {
        self.delta_r
    }

    pub(crate) fn get_bitmap(&self, index: usize) -> &Bitmap{
        &self.bitmaps[index]
    }

    pub(crate) fn get_surface(&self) -> f64 {
        self.surface
    }

    #[allow(dead_code)]
    fn get_density(&self, index: usize) -> f64 {
        let bmp = self.get_bitmap(index);
        let (width, height) = bmp.get_dims();

        (bmp.pixels as f64) / (width * height) as f64
    }
}
