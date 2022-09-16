use std::collections::HashMap;
use std::ops::DerefMut;
use std::pin::Pin;
use itertools::Itertools;
use crate::{Model, plater, stl};
use crate::plater::part::Part;
use crate::plater::plate_shape::PlateShape;
use crate::plater::solution::Solution;
use crate::stl::orientation::Orientation;
use crate::stl::part::load_model;


// type Request struct {
//     Request    *plater.Request
//     resolution float64
//     parts      map[string]*Part
// }

pub struct Request<'a> {
    pub(crate) request: plater::request::Request<'a, plater::plate_shape::Shape>,
    resolution: f64,
    models: HashMap<String, Model>,
    // parts: Vec<stl::part::Part>
    // parts: HashMap<>
}


impl<'a> Request<'a>  {

    pub fn process<T>(&self, f: impl Fn(&Solution) -> T) -> T {
        self.request.process(f)
    }

    pub fn new(plateShape: &'a plater::plate_shape::Shape, resolution: f64) -> Self {
        let request = plater::request::Request::new(plateShape, resolution);
        Request { request , resolution, models: Default::default() }
    }

    pub fn add_model(&mut self, filename: String, orientation: Orientation, locked: bool) -> Option<()> {
        if filename == "" {
            return None;
        }

        let mut id = filename.to_owned();

        // TODO: Optimize away n^2 behavior
        for i in 0.. {
            let res = self.request.parts.get(id.as_str());
            if res.is_none() {
                break;
            }
            id = format!("{} {}", filename, i);
        }

        let n = filename.to_owned();
        println!("Going to load {}", &n);

        let (part, model, loaded) = load_model(filename, id.to_owned(), self.resolution, self.request.precision,
                   self.request.delta_r, self.request.spacing, orientation,
        self.request.plate_shape.width(), self.request.plate_shape.height(),
        locked)?;



        if loaded == 0 {
            return None;
        }

        println!("Done loading {}", &n);

        self.models.insert(id.to_owned(), model);
        self.request.parts.insert(id, part);



        Some(())
    }

    fn create_model(&self, p: &plater::plate::Plate) -> Option<Model> {
        let placements = p.get_placements();

        println!("placement length {}", placements.len());

        placements
            .iter()
            .map(|x| {
                let id = x.id.as_str();
                let model = self.models.get(id).unwrap();
                model
                    .center()
                    .rotate_z(x.rotation)
                    .translate(x.center.x, x.center.y, 0.0)
            }).reduce(|mut x, mut y| {
            let mut m = Model::new();
            m.volumes.append(&mut x.volumes);
            m.volumes.append(&mut y.volumes);
            m
        })
    }

    pub(crate) fn write_stl(&self, p: &plater::plate::Plate, filename: String) -> Option<std::io::Result<()>> {
        println!("Going to make model");
        let model = self.create_model(p)?;
        println!("Created model");
        Some(model.save_to_file_binary(filename, self.resolution))
    }




}