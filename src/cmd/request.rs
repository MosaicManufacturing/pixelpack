use std::path::PathBuf;
use clap::{ArgEnum, Parser, PossibleValue, value_parser};
use crate::{plater, stl};
use crate::plater::placer::SortMode;
use crate::plater::plate::Plate;
use crate::plater::plate_shape::{PlateShape, Shape};
use crate::plater::solution::Solution;

#[derive(Parser, Debug)]
pub struct CliOpts {
    #[clap(short, long, value_parser, default_value_t = false)]
    verbose: bool,
    #[clap(short, long, value_parser, default_value_t = 150)]
    width: i32,
    #[clap(short, long, value_parser, default_value_t = 150)]
    height: i32,
    #[clap(short, long, value_parser, default_value_t = 0)]
    diameter: i32,
    #[clap(short, long, value_parser, default_value_t = 0.5)]
    precision: f64,
    #[clap(short, long, value_parser, default_value_t = 1.5)]
    spacing: f64,
    #[clap(short, long, value_parser, default_value_t = 1.5)]
    delta: f64,
    #[clap(short, long, value_parser, default_value_t = 90)]
    rotation_interval: i32,




    #[clap(short, long, value_parser)]
    multiple_sort: bool,

    #[clap(long, value_parser, default_value_t = 0)]
    random_iterations: i32,


    #[clap(short, long, value_parser, default_value = "plate_%03d")]
    output_pattern: String,
    #[clap(long, value_parser, default_value_t = false)]
    ppm: bool,
    #[clap(short, long, value_parser, default_value_t = 1)]
    threads: i32,
    #[clap(short, long, value_parser, default_value_t = false)]
    csv: bool,
    #[clap(short, long, multiple_values = true)]
    paths: Vec<PathBuf>,
}

fn get_plate_shape(opts: &CliOpts, resolution: f64) -> Shape {
    if opts.diameter > 0 {
        return Shape::new_circle(opts.diameter as f64, resolution)
    }

    return Shape::new_rectangle(opts.width as f64, opts.height as f64, resolution)
}

fn get_sort_modes(multiple_sort: bool, random_iterations: i32) -> Vec<plater::placer::SortMode> {
    if multiple_sort {
        let last_sort = 1 + random_iterations;
        let mut modes = vec![];

        for i in 0..last_sort {
            let x = match i {
                0 => SortMode::SortSurfaceDec,
                1 => SortMode::SortSurfaceInc,
                2 => SortMode::SortShuffle,

                // TODO: figure this out
                _ => todo!()
            };

            modes.push(x);
        }
        modes
    } else {
        vec![SortMode::SortSurfaceDec]
    }
}

pub fn run(opts: &CliOpts, filenames: Vec<String>) -> Option<()> {
    println!("{:#?}", opts);
    let resolution = plater::request::DEFAULT_RESOLUTION;
    let plate_shape = get_plate_shape(opts, resolution);

    let mut request = stl::request::Request::new(&plate_shape, resolution);

    request.request.spacing = opts.spacing * resolution;
    request.request.delta = opts.delta * resolution;
    request.request.delta_r = stl::util::deg_to_rad(opts.rotation_interval as f64);
    request.request.precision = opts.precision * resolution;
    request.request.sort_modes = get_sort_modes(opts.multiple_sort, opts.random_iterations);

    if opts.threads > 0 {
        request.request.max_threads = opts.threads as usize;
    }

    for filename in filenames {
        println!("Adding file {}", filename);
        request.add_model(filename, stl::orientation::Orientation::Bottom, false).unwrap();
    }

    let write_solution = |sol: &Solution| -> Option<()> {
        let count = sol.count_plates();

        println!("solution {}", count);

        for i in 0..count {
            let plate = sol.get_plate(i).unwrap();
            println!("Got plate");
            let out_file = format!("{}_{}.stl", opts.output_pattern, i);
            request.write_stl(plate, out_file)?.ok()?;
        }

        Some(())
    };

    request.process(write_solution)
}