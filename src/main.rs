extern crate core;

use std::path::PathBuf;

use clap::Parser;

mod plater;
mod stl;

#[derive(Parser, Debug)]
struct Args {
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
    delta: f64,
    #[clap(short, long, value_parser, default_value_t = 90)]
    rotation_interval: i32,
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

fn main() {
    let args: Args = Args::parse();

    println!("Hello, world!");
    println!("{:#?}", args);
}
