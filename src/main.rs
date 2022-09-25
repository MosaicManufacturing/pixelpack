extern crate core;

use std::f32::consts::E;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use clap::{ArgEnum, Parser, PossibleValue, value_parser};
use clap::builder::ValueParserFactory;

use crate::Format::{ASCII, Binary};
use crate::stl::model::Model;

mod plater;
mod stl;
mod cmd;

#[derive(Copy, Clone)]
enum Format {
    Binary,
    ASCII
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, value_parser)]
    source: PathBuf,

    #[clap(short, long, value_parser)]
    input_format: String,

    #[clap(short, long, value_parser)]
    dest: PathBuf,

    #[clap(short, long, value_parser)]
    output_format: String,
}

fn parse_format(s: &str) -> Option<Format> {
    let s= s.to_ascii_lowercase();
    match s.as_str() {
        "ascii" => Some(ASCII),
        "binary" => Some(Binary),
        _ => None
    }
}

fn main() {
    //
    // rayon::ThreadPoolBuilder::new().num_threads(1).build_global().unwrap();
    //

    let args = cmd::request::CliOpts::parse();
    let xs = (0..50)
        .into_iter()
        .flat_map(|_| ["Gimbal_snowflake_small_and_flat.STL".into()])
        .collect();
    println!("Going to start run");


    let t1 = Instant::now();
    let x = cmd::request::run(&args, xs).unwrap();

    println!("{} ms", t1.elapsed().as_millis());
}

// fn main() -> std::io::Result<()> {
//     let args: Args = Args::parse();
//
//     let input_format = parse_format(args.input_format.as_str())
//         .expect("Invalid input format");
//
//     let output_format = parse_format(args.output_format.as_str())
//         .expect("Invalid output format");
//
//
//     let load_file = match input_format {
//         Binary => Model::load_stl_file_binary::<PathBuf>,
//         ASCII => Model::load_stl_file_ascii::<PathBuf>,
//     };
//
//     let write_file = match output_format {
//         Binary => Model::save_to_file_binary::<PathBuf>,
//         ASCII => Model::save_to_file_ascii::<PathBuf>,
//     };
//
//     println!("Going to load model");
//     let model = load_file(args.source, 1.0)?;
//
//
//     println!("Loaded model");
//     let triangles = model
//         .volumes
//         .iter()
//         .map(|x| x.faces.len())
//         .sum::<usize>();
//
//     println!("{}", triangles);
//     write_file(&model, args.dest, 1.0)?;
//     Ok(())
// }
