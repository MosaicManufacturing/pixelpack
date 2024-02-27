use std::f64::consts::PI;

use itertools::Itertools;
use log::info;

use crate::plater::bitmap::Bitmap;

pub struct Part {
    pub(crate) locked: bool,
    // if true, part cannot be moved or rotated
    pub(crate) id: String,
    pub(crate) precision: f64,
    pub(crate) delta_r: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
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
        plate_width: f64,
        plate_height: f64,
        locked: bool,
        dilation_spacing: i32,
        top_left_spacing: i32,
    ) -> anyhow::Result<Self> {
        let trimmed_original = bitmap.trim();
        drop(bitmap);

        let mut num_bitmaps = f64::ceil(PI * 2.0 / delta_r) as i32;
        if locked {
            num_bitmaps = 1;
        }

        let (width, height) = trimmed_original.get_dims();

        // Improvement, we currently only use a rotation if it fits within the original plate

        // if for every model there exists a rotation that is contained within,
        // we may attempt to place the model

        info!("Original");
        info!("{}", trimmed_original.to_ppm());

        let undilated_bitmaps = if locked {
            vec![trimmed_original]
        } else {
            (0..num_bitmaps as usize)
                .into_iter()
                .map(|k| trimmed_original.rotate((k as f64) * delta_r))
                .collect()
        };

        let addition_spacing = dilation_spacing + top_left_spacing;

        let bitmaps = undilated_bitmaps
            .into_iter()
            .map(|x| {
                // info!("Pre shrink");
                // info!("{}", x.to_ppm());
                x
            })
            .map(|bmp| bmp.trim())
            .map(|bmp| {
                // info!("Shrunk");
                // info!("{}", bmp.to_ppm());
                let mut bmp = bmp.grow(addition_spacing, addition_spacing);
                //
                // info!("Pre");
                // info!("{}", bmp.to_ppm());

                // TODO: ensure that the bmp has enough space present for the dilation to be applied. If the model goes to the ends of the bitmap, this entire exercise is futile. Add a buffer of dilation_spacign + top_left_spacign, idlate and then downsize
                if dilation_spacing > 0 {
                    bmp.dilate(dilation_spacing);
                }

                if top_left_spacing > 0 {
                    bmp.top_left_dilate(top_left_spacing);
                }
                //
                // info!("Inter");
                // info!("{}", bmp.to_ppm());

                info!("Post");
                let b = bmp.trim();
                info!("{}", b.to_ppm());

                b
            })
            .collect_vec();

        let mut p = Part {
            precision,
            delta_r,
            id: id.to_string(),
            locked,
            bitmaps,
            center_y,
            center_x,
            width: width as f64 + 2.0 * spacing,
            height: height as f64 + 2.0 * spacing,
            surface: 0.0,
        };

        let mut correct = 0;

        for k in 0..num_bitmaps as usize {
            let Bitmap { width, height, .. } = &p.bitmaps[k];

            if *width as f64 * precision < plate_width + 2.0 * spacing
                && *height as f64 * precision < plate_height + 2.0 * spacing
            {
                p.surface += (width * height) as f64;
                correct += 1;
            }
        }

        if correct == 0 {
            anyhow::bail!(
                "None of the rotations of {} fit within Plate width {} height {}",
                id,
                plate_width,
                plate_height
            );
        }

        p.surface /= correct as f64;
        anyhow::Ok(p)
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.id
    }

    #[allow(dead_code)]
    fn get_rotation(&self) -> f64 {
        self.delta_r
    }

    pub(crate) fn get_bitmap(&self, index: usize) -> &Bitmap {
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
