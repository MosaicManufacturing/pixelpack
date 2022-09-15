use std::collections::HashMap;
use crate::plater;


// type Request struct {
//     Request    *plater.Request
//     resolution float64
//     parts      map[string]*Part
// }

struct Request<'a> {
    request: plater::request::Request<'a, plater::plate_shape::Shape>,
    resolution: f64,
    // parts: HashMap<>

}


// func NewRequest(plateShape plater.PlateShape, resolution float64) *Request {
// 	r := new(Request)
// 	r.Request = plater.NewRequest(plateShape, resolution)
// 	r.resolution = resolution
// 	r.parts = make(map[string]*Part)
// 	return r
// }