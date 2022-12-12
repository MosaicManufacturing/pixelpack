use std::collections::HashMap;

use log::info;
use serde::{Deserialize, Serialize};

use pixelpack::plater;
use pixelpack::plater::bitmap::Bitmap;
use pixelpack::plater::plate_shape::{PlateShape, Shape};
use pixelpack::plater::request::{Algorithm, default_sort_modes};
use pixelpack::plater::solution::Solution;
use pixelpack::stl::util::{deg_to_rad, rad_to_deg};

#[derive(Deserialize, Debug)]
pub struct WasmArgs {
    pub options: RequestOptions,
    pub model_options: Vec<ModelOptions>,
    pub offsets: Vec<u32>,
}

#[derive(Deserialize, Debug)]
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
pub struct ModelResult {
    center_x: f64,
    center_y: f64,
    rotation: f64,
}

#[derive(Serialize, Debug)]
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
) -> Option<PlacingResult> {
    // Each model has a bunch of orientations with different dimensions

    // width and height are scaled by resolution param

    // let (width, height) = models
    //     .iter()
    //     .map(|x| (x.width, x.height))
    //     .reduce(|x, y| (i32::max(x.0, y.0), i32::max(x.1, y.1)))
    //     .map(|(x, y)|  (x as f64/opts.resolution, y as f64/opts.resolution))
    //     .map(|(x, y)| (f64::max(x, opts.width as f64), f64::max(y, opts.height as f64)))?;
    //
    // opts.width = 10;
    // opts.height = 10;

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

    info!("DIMS {} {}", plate_width, plate_height);

    // TODO: There might be a missing scaling factor for the center
    let mut request = plater::request::Request::new(plate_shape, resolution, alg, opts.bed_center_x, opts.bed_center_y);

    if opts.precision > 0.0 {
        request.precision = opts.precision * resolution;
    }

    if opts.spacing > 0.0 {
        request.spacing = opts.spacing * resolution;
    }

    if opts.delta > 0.0 {
        request.delta = opts.delta * resolution;
    }

    request.delta_r = deg_to_rad(opts.rotation_interval);
    request.sort_modes = default_sort_modes();

    for (i, model) in models.iter().enumerate() {
        info!("Adding model {} {}", model.id, bitmaps[i].len());
        model_opts_map.insert(model.id.to_string(), model);

        let mut bmp =
            Bitmap::new_bitmap_with_data(model.width, model.height, bitmaps[i]).unwrap();


        bmp.dilate((request.spacing/request.precision) as i32);

        let delta_r = if model.rotation_interval > 0.0 {
            deg_to_rad(model.rotation_interval)
        } else {
            request.delta_r
        };

        let spacing = if model.spacing > 0.0 {
            model.spacing * resolution
        } else {
            request.spacing
        };

        info!("Creating part");
        let part = plater::part::Part::new(
            model.id.to_owned(),
            bmp,
            model.center_x,
            model.center_y,
            request.precision,
            delta_r,
            spacing,
            plate_width,
            plate_height,
            model.locked,
        );

        info!("Part loaded");

        request.add_part(part).unwrap();
    }

    info!("Loaded all parts");

    let on_solution = |sol: &Solution| {
        let mut result = HashMap::new();

        let plate = &sol.get_plates()[0];


        info!("{}", plate.get_ppm());

        for placement in plate.get_placements() {
            info!("{:#?}", placement);
            let id = placement.id.to_owned();
            let model_opts = model_opts_map.get(&id).unwrap();
            // Center x and center y are funky when in locked mode
            if !model_opts.locked {
                result.insert(
                    placement.id.to_owned(),
                    ModelResult {
                        // TODO: All of this should be made private
                        center_x: placement.center.x,
                        center_y: placement.center.y,
                        rotation: rad_to_deg(placement.rotation),
                    },
                );
            }
        }

        let (plate_width, plate_height) = plate.get_size();
        PlacingResult {
            models: result,
            plate_width,
            plate_height
        }
    };


    // let mut all_results = vec![];
    // for (i, tolerance) in [(10.0, 40), (5.0, 30), (2.5,20), (1.25,15), (0.75, 10)] {
    //     info!("I {}, tolerance {}", i, tolerance);
    //     request.delta = i as f64 * resolution;
    //     let result = request.process(on_solution);
    //     if result.plate_width <= plate_width {
    //         info!("{:#?}", result);
    //         return Some(result);
    //     } else {
    //         if result.plate_width - plate_width > tolerance as f64 {
    //             info!("Fail {} {} tolerance {}", result.plate_width, plate_width, tolerance);
    //             all_results.push(result);
    //             break;
    //         }
    //         all_results.push(result);
    //     }
    //
    // }
    //
    // all_results.sort_by(|x, y| f64::total_cmp(&y.plate_width, &x.plate_width));
    // let result = all_results.pop();
    // info!("{:#?}", result);
    //
    // result

    Some(request.process(on_solution))

}
