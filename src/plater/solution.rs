use crate::plater::plate::Plate;

pub struct Solution {
    plates: Vec<Plate>,
}

impl Solution {
    pub(crate) fn new() -> Self {
        Solution { plates: vec![] }
    }

    // Score represents the score associated with this solution.
    // A lower score represents a more optimal solution.
    fn score(&self, s: &Solution) -> f64 {
        return (s.count_plates()) as f64
            + (1.0 - 1.0 / (1 + s.get_last_plate().count_parts()) as f64);
    }

    pub(crate) fn count_plates(&self) -> usize {
        self.plates.len()
    }

    pub(crate) fn get_plate(&self, n: usize) -> Option<&Plate> {
        self.plates.get(n)
    }

    pub(crate) fn get_plate_mut(&mut self, n: usize) -> Option<&mut Plate> {
        self.plates.get_mut(n)
    }

    fn get_last_plate(&self) -> &Plate {
        self.get_plate(self.plates.len()).unwrap()
    }

    pub(crate) fn add_plate(&mut self, plate: Plate) {
        self.plates.push(plate);
    }
}
