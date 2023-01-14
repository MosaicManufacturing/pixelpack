extern crate core;

use std::time::Instant;

use clap::Parser;
use log::info;
use simple_logger::SimpleLogger;





mod request;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, value_parser)]
    number: usize,
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let args = request::CliOpts::parse();
    let xs = (0..args.threads)
        .into_iter()
        .flat_map(|_| ["Gimbal_snowflake_small_and_flat.STL".into()])
        .collect();
    request::run(&args, xs).unwrap();
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
//     let model = load_file(args.source, 1.0)?;
//
//
//     let triangles = model
//         .volumes
//         .iter()
//         .map(|x| x.faces.len())
//         .sum::<usize>();
//
//     write_file(&model, args.dest, 1.0)?;
//     Ok(())
// }
