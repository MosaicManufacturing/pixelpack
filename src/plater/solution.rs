use crate::plater::plate::Plate;

struct Solution {
    plates: Vec<Plate>
}

impl Solution {
    fn new() -> Self {
        Solution {
            plates: vec![]
        }
    }

    // Score represents the score associated with this solution.
// A lower score represents a more optimal solution.
    fn score(&self, s: &Solution) -> f64 {
    return (s.count_plates()) as f64 + (1.0 - 1.0 / (1 + s.get_last_plate().count_parts()) as f64   )
    }

    fn count_plates(&self) -> usize {
        self.plates.len()
    }

    fn get_plate(&self, n: usize) -> Option<&Plate> {
        self.plates.get(n)
    }

    fn get_last_plate(&self) -> &Plate {
        self.get_plate(self.plates.len()).unwrap()
    }

    fn add_plate(&mut self, plate: Plate) {
        self.plates.push(plate);
    }

}