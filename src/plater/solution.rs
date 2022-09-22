use crate::plater::placed_part::PlacedPart;
use crate::plater::plate::Plate;

pub struct Solution<'a> {
    plates: Vec<Plate<'a>>,
}

impl<'a> Solution<'a> {
    pub(crate) fn new() -> Self {
        Solution { plates: vec![] }
    }


    pub(crate) fn reclaim_placed_parts(self) -> Vec<PlacedPart<'a>> {
        let mut result = vec![];
        for plate in self.plates {
            for x in plate.parts {
                result.push(x);
            }
        }

        result
    }

    // Score represents the score associated with this solution.
    // A lower score represents a more optimal solution.
    pub(crate) fn score(&self) -> f64 {
        return (self.count_plates()) as f64
            + (1.0 - 1.0 / (1 + self.get_last_plate().count_parts()) as f64);
    }

    pub(crate) fn count_plates(&self) -> usize {
        self.plates.len()
    }

    pub(crate) fn get_plate(&self, n: usize) -> Option<&Plate> {
        self.plates.get(n)
    }

    pub(crate) fn get_plate_mut<'b>(&'b mut self, n: usize) -> Option<&'b mut Plate<'a>> {
        self.plates.get_mut(n)
    }

    fn get_last_plate(&self) -> &Plate {
        self.get_plate(self.plates.len() - 1).unwrap()
    }

    pub(crate) fn add_plate(&mut self, plate: Plate<'a>) {
        self.plates.push(plate);
    }
}
