use std::fmt::Debug;

use crate::plater::placer::search::Attempts::{Failure, Solved, ToCompute};

#[derive(Clone, Debug)]
pub enum Attempts<T> {
    ToCompute,
    Solved(T),
    Failure,
}

// TODO: move computation driving to the closure, the search function shouldn't care about how this happens
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

    let mut lo = lower as usize;
    let mut hi;

    if let Some(x) = first_found_solution {
        results[(i) as usize] = Solved(x);
        hi = (i) as usize;
    } else {
        hi = limit;
    }

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

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use itertools::Itertools;

    use crate::plater::placer::search::exponential_search;

    fn find_min_val_gt_cut_in_range(range: Range<i32>, cut: i32) -> Option<i32> {
        let xs = &(range.clone()).collect_vec();
        let result = exponential_search((&range.max().unwrap() + 1) as usize, |i| {
            let res = xs.get(i);
            if let Some(x) = res {
                if *x > cut {
                    Some(*x)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .map(|x| (x.0));
        result
    }

    #[test]
    fn test() {
        for i in 0..(10000 - 1) {
            println!("{}", i);
            assert_eq!(find_min_val_gt_cut_in_range(0..10000, i), Some(i + 1))
        }
    }

    #[test]
    fn test1() {
        assert_eq!(find_min_val_gt_cut_in_range(0..1024, 2), Some(3));
        assert_eq!(find_min_val_gt_cut_in_range(0..1024, 10), Some(11));
        assert_eq!(find_min_val_gt_cut_in_range(0..1024, 63), Some(64));
        assert_eq!(find_min_val_gt_cut_in_range(0..1024, 100), Some(101));
        assert_eq!(find_min_val_gt_cut_in_range(0..1024, 128), Some(129));
        assert_eq!(find_min_val_gt_cut_in_range(0..1024, 510), Some(511));
        assert_eq!(find_min_val_gt_cut_in_range(0..1024, 600), Some(601));
    }
}
