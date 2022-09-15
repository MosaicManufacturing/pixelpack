use std::collections::HashMap;
use std::ops::DerefMut;
use std::pin::Pin;
use crate::{plater, stl};


// type Request struct {
//     Request    *plater.Request
//     resolution float64
//     parts      map[string]*Part
// }

struct Request<'a> {
    request: plater::request::Request<'a, plater::plate_shape::Shape>,
    resolution: f64,
    parts: Vec<stl::part::Part>
    // parts: HashMap<>
}


// impl<'a> Request<'a>  {
//
//     fn new() -> Self {
//         let x = Request {}
//     }
//
//     fn help(&mut self) {
//         let xs = &mut self.parts;
//         let x = xs.deref_mut();
//     }
// }


// func NewRequest(plateShape plater.PlateShape, resolution float64) *Request {
// 	r := new(Request)
// 	r.Request = plater.NewRequest(plateShape, resolution)
// 	r.resolution = resolution
// 	r.parts = make(map[string]*Part)
// 	return r
// }