use std::collections::HashMap;

use crate::plater;
use crate::plater::plate_shape::PlateShape;
use crate::plater::request::{Algorithm, BedExpansionMode, ConfigOrder, PointEnumerationMode, Strategy, ThreadingMode};
use crate::plater::solution::Solution;
use crate::stl::model::Model;
use crate::stl::orientation::Orientation;
use crate::stl::part::load_model;

pub struct Request {
    pub request: plater::request::Request<plater::plate_shape::Shape>,
    resolution: f64,
    models: HashMap<String, Model>,
}

impl Request {
    pub fn process<T>(&self, on_solution_found: impl Fn(&Solution) -> T) -> T {
        self.request.process(on_solution_found)
    }

    pub fn new(plate_shape: plater::plate_shape::Shape, resolution: f64) -> Self {
        let alg = Algorithm {
            threading_mode: ThreadingMode::MultiThreaded,
            strategy: Strategy::SpiralPlace,
            order_config: ConfigOrder::PointFirst,
            point_enumeration_mode: PointEnumerationMode::Row,
            bed_expansion_mode: BedExpansionMode::Exponential,
        };

        let (width, height) = (plate_shape.width() / resolution, plate_shape.height() / resolution);

        // There might be a missing scaling factor for the center
        let request = plater::request::Request::new(plate_shape, resolution, alg, width, height);
        Request {
            request,
            resolution,
            models: Default::default(),
        }
    }

    pub fn add_model(
        &mut self,
        filename: String,
        orientation: Orientation,
        locked: bool,
    ) -> Option<()> {
        if filename.is_empty() {
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

        let (part, model) = load_model(
            filename,
            id.to_owned(),
            self.resolution,
            self.request.precision,
            self.request.delta_r,
            self.request.spacing,
            orientation,
            self.request.plate_shape.width(),
            self.request.plate_shape.height(),
            locked,
        )?;

        self.models.insert(id.to_owned(), model);
        self.request.parts.insert(id, part);

        Some(())
    }

    fn create_model(&self, p: &plater::plate::Plate) -> Option<Model> {
        let placements = p.get_placements();
        let x = placements
            .iter()
            .map(|placement| {
                let id = placement.get_id();
                let model = self.models.get(id.as_str()).unwrap();
                let center = placement.get_center();
                model
                    .clone()
                    .center_consume()
                    .rotate_z_consume(placement.get_rotation())
                    .translate_consume(center.x, center.y, 0.0)
            })
            .reduce(|mut x, mut y| {
                let mut m = Model::new();
                m.volumes.append(&mut x.volumes);
                m.volumes.append(&mut y.volumes);
                m
            });

        x
    }

    pub fn write_stl(
        &self,
        p: &plater::plate::Plate,
        filename: String,
    ) -> Option<std::io::Result<()>> {
        let model = self.create_model(p)?;
        Some(model.save_to_file_binary(filename, self.resolution))
    }
}