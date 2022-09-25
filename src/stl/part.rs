use crate::{Model, plater};
use crate::stl::orientation::Orientation;

pub(crate) struct Part {
    // move ownership of plater::part::Part out of stl part to plater/part (replace hashmap with vec, ids with indexs), store an id
    part: plater::part::Part,
    filename: String,
    model: Model
}


pub(crate) fn load_model(filename: String, id: String,
                  resolution: f64, precision: f64,
                  delta_r: f64, spacing: f64,
                  orientation: Orientation,
                    plate_width: f64, plate_height: f64, locked: bool) -> Option<(plater::part::Part, Model, i32)> {

        let mut model = Model::load_stl_file_binary(filename, resolution).ok()?;

        let next_model = model.put_face_on_plate(orientation);
        // TODO: Is this correct?, shouldn't we pixelize the rotated model
        let bitmap = model.pixelize(precision,  spacing);

        let min = next_model.min();
        let max = next_model.max();

        let center_x = (min.x + max.x)/2.0;
        let center_y = (min.y + max.y)/2.0;

        let res = plater::part::Part::new(id, bitmap, center_x, center_y,
        precision, delta_r, spacing, plate_width, plate_height, locked);

        Some((res.0, next_model, res.1 as i32))
    }


