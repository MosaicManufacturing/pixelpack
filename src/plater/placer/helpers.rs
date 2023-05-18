use crate::plater::placer::{
    all_parts_can_be_attempted, all_parts_can_eventually_be_attempted, Placer, N,
};
use crate::plater::plate::Plate;
use crate::plater::plate_shape::PlateShape;
use crate::plater::solution::Solution;

const EXPAND_MM: f64 = 5.0; // cutoff index

pub(crate) fn find_solution<'a, 'b>(
    search_index: usize,
    original_shape: &Box<dyn PlateShape>,
    maybe_lowest_encountered_search_index: &Option<usize>,
    placer: &'a mut Placer<'b>,
    bottom_left: (f64, f64),
) -> Option<Solution<'b>> {
    if let Some(lowest_encountered_search_index) = maybe_lowest_encountered_search_index {
        // Don't bother searching if cannot do better than the best solution encountered so far
        if search_index >= *lowest_encountered_search_index {
            return None;
        }
    }

    let mut should_align_to_bed = false;
    placer.current_bounding_box = None;

    let mut shape = if search_index < N {
        original_shape.contract((N - search_index) as f64 * EXPAND_MM)?
    } else if search_index == N {
        original_shape.clone()
    } else {
        should_align_to_bed = true;
        original_shape.expand(original_shape.width() / placer.request.precision)
    };

    let center = if search_index <= N {
        (placer.request.center_x, placer.request.center_y)
    } else {
        (
            bottom_left.0 + shape.width() / (2.0 * placer.request.precision),
            bottom_left.1 + shape.height() / (2.0 * placer.request.precision),
        )
    };

    let mut unlocked_parts = Vec::clone(&placer.unlocked_parts);
    let mut plate = Plate::make_plate_with_placed_parts(
        shape.as_ref(),
        placer.request.precision,
        &mut Vec::clone(&placer.locked_parts),
        center.0,
        center.1,
    )?;

    if search_index <= N && !all_parts_can_be_attempted(&unlocked_parts, shape.as_ref()) {
        return None;
        // Add special handling if some parts will never fit
    } else if search_index > N
        && !all_parts_can_eventually_be_attempted(&unlocked_parts, shape.as_ref())
    {
        return None;
    }

    // Determine current bounding box using pixel data from bitmap

    // if !self.locked_parts.is_empty() {
    //     plate
    // }

    while let Some(cur_part) = unlocked_parts.pop() {
        match placer.place_unlocked_part(&mut plate, cur_part) {
            None => {}
            Some(part) => {
                // return None;
                if search_index <= N {
                    return None;
                }
                should_align_to_bed = true;
                unlocked_parts.push(part);
                shape = shape.expand(original_shape.width() / original_shape.resolution());
                plate = Plate::make_from_shape(&mut plate, shape.as_ref(), bottom_left)
            }
        }
    }

    // Centering models is only correct if there are no locked parts
    if placer.locked_parts.is_empty() {
        if should_align_to_bed {
            let width = placer.request.plate_shape.width();
            let height = placer.request.plate_shape.height();
            plate.align(width, height);
        } else {
            plate.center();
        }
    }

    let mut solution = Solution::new();
    solution.add_plate(plate);
    solution.best_so_far = Some(search_index);
    Some(solution)
}
