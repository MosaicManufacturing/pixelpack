use std::fmt::Debug;

use crate::plater::placer::search::Attempts::{Failure, Solved, ToCompute};

#[derive(Clone, Debug)]
pub enum Attempts<T> {
    ToCompute,
    Solved(T),
    Failure,
}

pub(crate) fn exponential_search<T: Clone + Debug>(
    limit: usize,
    mut run: impl FnMut(usize) -> Option<T>,
) -> Option<(T, usize)> {
    let mut first_found_solution = None;

    let mut i = 1;
    let mut lower = i;

    while i < limit {
        let res = run(i);
        if res.is_some() {
            first_found_solution = res;
            break;
        }

        if i * 2 >= limit {
            break;
        }

        lower = i;
        i *= 2;
    }

    let mut results = vec![ToCompute; 2 * limit];

    results.iter_mut().for_each(|x| *x = ToCompute);

    if results.len() < i + 1 {
        unreachable!()
    }

    let mut j = 1;
    while j < i {
        results[j] = Failure;
        j *= 2;
    }

    if first_found_solution.is_none() {
        return None;
    }

    results[(i) as usize] = Solved(first_found_solution.unwrap());

    let mut lo = lower as usize;
    let mut hi = (i) as usize;

    let mut boundary_index = 1;

    while lo <= hi {
        let gap = hi - lo;
        let mid = lo + gap / 2;
        if let ToCompute = results[mid] {
            results[mid] = match run(mid) {
                None => Failure,
                Some(x) => Solved(x),
            }
        }

        match results[mid] {
            Solved(_) => {
                if mid == 1 {
                    boundary_index = mid as i32;
                    break;
                }

                if let ToCompute = results[mid - 1] {
                    results[mid - 1] = match run(mid - 1) {
                        None => Failure,
                        Some(x) => Solved(x),
                    }
                }

                if let Failure = results[mid - 1] {
                    boundary_index = mid as i32;
                    break;
                }

                hi = mid - 1;
            }
            Failure => {
                lo = mid + 1;
            }
            ToCompute => unreachable!(),
        }
    }

    let mut ans = ToCompute;
    std::mem::swap(&mut ans, &mut results[boundary_index as usize]);
    match ans {
        Solved(x) => Some((x, boundary_index as usize)),
        _ => None,
    }
}
