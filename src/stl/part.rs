use crate::{Model, plater};
use crate::stl::orientation::Orientation;

struct Part {
    part: plater::part::Part,
    filename: String,
    model: Model
}

impl Part {
    fn load_model(filename: String, id: String,
                  resolution: f64, precision: f64,
                  delta_r: f64, spacing: f64,
                  orientation: Orientation,
                    plate_width: f64, plate_height: f64, locked: bool) -> Option<(Part, i32)> {

        // let model = Model::lo

        todo!()
        // let part = plater::part::Part::new();
    }
}


// func LoadModel(
// 	filename, id string,
// 	resolution, precision, deltaR, spacing float64,
// 	orientation Orientation,
// 	plateWidth, plateHeight float64,
// 	locked bool,
// ) (*Part, int, error) {
// 	p := new(Part)
// 	p.filename = filename
// 	model, err := LoadSTLFile(filename, resolution)
// 	if err != nil {
// 		return nil, 0, err
// 	}
// 	p.model = model.PutFaceOnPlate(orientation)
// 	bitmap := model.Pixelize(precision, spacing)
//
// 	min := p.model.Min()
// 	max := p.model.Max()
// 	centerX := (min.X + max.X) / 2
// 	centerY := (min.Y + max.Y) / 2
//
// 	part, loaded := plater.NewPart(
// 		id, bitmap,
// 		centerX, centerY,
// 		precision, deltaR, spacing,
// 		plateWidth, plateHeight,
// 		locked,
// 	)
// 	p.part = part
// 	return p, loaded, nil
// }




