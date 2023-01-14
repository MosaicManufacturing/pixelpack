use std::collections::HashMap;
use anyhow::{anyhow, bail, Context};

use serde::{Deserialize, Serialize};

use pixelpack::plater;
use pixelpack::plater::bitmap::Bitmap;
use pixelpack::plater::plate_shape::{PlateShape, Shape};
use pixelpack::plater::request::{Algorithm, default_sort_modes};
use pixelpack::plater::solution::Solution;
use pixelpack::stl::util::{deg_to_rad, rad_to_deg};
use crate::TaggedError;
use crate::TaggedError::{Hidden, Reportable};
use typeshare::typeshare;

#[derive(Deserialize, Debug)]
#[typeshare]
pub struct WasmArgs {
    pub options: RequestOptions,
    pub model_options: Vec<ModelOptions>,
    pub offsets: Vec<u32>,
}

#[derive(Deserialize, Debug)]
#[typeshare]
pub struct RequestOptions {
    width: i32,
    height: i32,
    diameter: i32,
    precision: f64,
    resolution: f64,
    spacing: f64,
    delta: f64,
    rotation_interval: f64,
    multiple_sort: bool,
    random_iterations: i32,
    bed_center_x: f64,
    bed_center_y: f64,
}

#[derive(Deserialize, Debug)]
#[typeshare]
pub struct ModelOptions {
    id: String,
    locked: bool,
    width: i32,
    height: i32,
    center_x: f64,
    center_y: f64,
    spacing: f64,
    rotation_interval: f64,
}

#[derive(Serialize, Debug)]
#[typeshare]
pub struct ModelResult {
    center_x: f64,
    center_y: f64,
    rotation: f64,
}

#[derive(Serialize, Debug)]
#[typeshare]
pub struct PlacingResult {
    models: HashMap<String, ModelResult>,
    plate_width: f64,
    plate_height: f64
}

fn get_plate_shape(opts: &RequestOptions, resolution: f64) -> Shape {
    if opts.diameter > 0 {
        Shape::new_circle(opts.diameter as f64, resolution)
    } else {
        Shape::new_rectangle(opts.width as f64, opts.height as f64, resolution)
    }
}

pub fn handle_request(
    opts: RequestOptions,
    models: Vec<ModelOptions>,
    bitmaps: Vec<&[u8]>,
    alg: Algorithm
) -> Result<PlacingResult, TaggedError> {

    // Use default
    let resolution = if opts.resolution > 0.0 {
        opts.resolution
    } else {
        1000.0
    };

    let mut model_opts_map = HashMap::new();

    let plate_shape = get_plate_shape(&opts, resolution);
    let plate_width = plate_shape.width();
    let plate_height = plate_shape.height();
    let mut request = plater::request::Request::new(plate_shape, resolution, alg, opts.bed_center_x, opts.bed_center_y);

    if opts.precision > 0.0 {
        request.set_precision(opts.precision);
    }

    if opts.spacing > 0.0 {
        request.set_spacing(opts.spacing);
    }

    if opts.delta > 0.0 {
        request.set_delta(opts.delta);
    }

    request.set_delta_r(opts.rotation_interval);
    let sort_modes = default_sort_modes();
    request.set_sort_modes(sort_modes);

    if models.len() != bitmaps.len() {
        return Err(Hidden(anyhow!("Models len {} != bitmaps len {}", models.len(), bitmaps.len())));
    }


    for (i, model) in models.iter().enumerate() {
        model_opts_map.insert(model.id.to_string(), model);

        let mut bmp =
            Bitmap::new_bitmap_with_data(model.width, model.height, bitmaps[i])
                .with_context(|| format!("Could not load bitmap[{}] with model {}", i, model.id))
                .map_err(Hidden)?;

        bmp.dilate((request.get_spacing()/request.get_precision()) as i32);

        let delta_r = if model.rotation_interval > 0.0 {
            deg_to_rad(model.rotation_interval)
        } else {
            request.get_delta_r()
        };

        let spacing = if model.spacing > 0.0 {
            model.spacing * resolution
        } else {
            request.get_spacing()
        };

        let part = plater::part::Part::new(
            model.id.to_owned(),
            bmp,
            model.center_x,
            model.center_y,
            request.get_precision(),
            delta_r,
            spacing,
            plate_width,
            plate_height,
            model.locked,
        ).with_context(|| format!("Could not create part for model {}", model.id))
            .map_err(Reportable)?;
        request
            .add_part(part)
            .with_context(|| format!("Could not add part {}", model.id.to_string()))
            .map_err(Hidden)?;
    }

    let on_solution = |sol: &Solution| {
        let mut result = HashMap::new();

        let plate = (&sol)
            .get_plates()
            .get(0)
            .with_context(|| format!("No plates found"))
            .map_err(Reportable)?;

        for placement in plate.get_placements() {
            let id = placement.get_id();
            let model_opts = model_opts_map
                .get(&id)
                .with_context(|| format!("Could not find {} in placement", id))
                .map_err(Hidden)?;
            // Center x and center y are funky when in locked mode
            if !model_opts.locked {
                let center = placement.get_center();
                let (center_x, center_y) = (center.get_x(), center.get_y());
                result.insert(
                    id,
                    ModelResult {
                        center_x,
                        center_y,
                        rotation: rad_to_deg(placement.get_rotation()),
                    },
                );
            }
        }

        let (plate_width, plate_height) = plate.get_size();
        let placing_result = PlacingResult {
            models: result,
            plate_width,
            plate_height
        };
        Ok(placing_result)
    };

    request.process(on_solution)
}
