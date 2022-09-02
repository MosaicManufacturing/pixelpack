use std::collections::HashMap;
use crate::plater::part::Part;
use crate::plater::plate_shape::PlateShape;

// DEFAULT_RESOLUTION is the default bitmap resolution, in pixels per mm.
const DEFAULT_RESOLUTION: f64 = 1000.0;


struct Request {
    // plate_shape represents the size and shape of the build plate.
    plate_shape: Box<dyn PlateShape>,
    // single_plate_mode uses a single, expandable plate
    single_plate_mode: bool,
    // sort_modes is a list of sort modes to attempt when placing.
    sort_modes: Vec<i32>,
    // max_threads is the maximum number of goroutines to use when placing.
    // Set this to 0 or a negative value for no limit.
    max_threads: usize,
    precision: f64, // precision
    spacing:    f64, // part spacing

    // brute-force deltas
    delta:  f64,
    delta_r: f64,

    // Parts to place
    parts: HashMap<String, Part>,
    resolution: f64 // internal resolution (pixels per mm)
}