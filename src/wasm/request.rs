use std::collections::HashMap;
use crate::plater;
use crate::plater::bitmap::Bitmap;
use crate::plater::plate_shape::{PlateCircle, PlateRectangle, Shape};
use crate::plater::rectangle::Rectangle;
use crate::plater::request::default_sort_modes;
use crate::stl::util::deg_to_rad;

struct RequestOptions {
    width: i32,
    height: i32,
    diameter: i32,
    precision: f64,
    spacing: f64,
    delta: f64,
    rotation_interval: f64,
    multiple_sort: bool,
    random_iterations: i32
}

struct ModelOptions {
    id: String,
    locked: bool,
    width: i32,
    height: i32,
    center_x: f64,
    center_y: f64,
    spacing: f64,
    rotation_interval: i32
}

struct ModelResult {
    offset_x: f64,
    offset_y: f64,
    rotation: f64
}

fn get_plate_shape(opts: &RequestOptions, resolution: f64) -> Shape {
    if opts.diameter > 0 {
        Shape::new_circle(opts.diameter as f64, resolution)
    } else {
        Shape::new_rectangle(opts.width as f64, opts.height as f64, resolution)
    }
}

fn handle_request(opts: RequestOptions, models: Vec<ModelOptions>, bitmaps: Vec<Vec<u8>>) {
    // Use default
    let resolution = 1000.0;

    let mut model_opts_map = HashMap::new();


    let plate_shape =  get_plate_shape(&opts, resolution);
    let mut request = plater::request::Request::new(&plate_shape, resolution);

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
        model_opts_map.insert(model.id.to_string(), model);
        let bmp = Bitmap::new_bitmap_with_data(opts.width, opts.height, bitmaps[i].as_slice()).unwrap();

        let delta_r = if model.rotation_interval > 0 {
            deg_to_rad(model.rotation_interval as f64)
        } else {
            request.delta_r
        };

        let spacing = if model.spacing > 0.0 {
            model.spacing * resolution
        } else {
            request.spacing
        };

        let (part, loaded) = plater::part::Part::new(model.id.to_owned(), bmp
                                           ,model.center_x, model.center_y, request.precision, delta_r, spacing,
            opts.width as f64, opts.height as f64, model.locked);

        if loaded == 0 {
            unreachable!()
        }

        request.add_part(part).unwrap();
    }



    let result= request.process(|sol| {
        let mut result = HashMap::new();
        for plate in sol.get_plates() {
            for placement in plate.get_placements() {
                let id = placement.id.to_owned();
                let model_opts = model_opts_map.get(&id).unwrap();
                result.insert(placement.id.to_owned(), ModelResult {
                    offset_x: placement.center.x - model_opts.center_x,
                    offset_y: placement.center.y - model_opts.center_y,
                    rotation: placement.rotation
                });
            }
        }

        result
    });

    todo!()




}