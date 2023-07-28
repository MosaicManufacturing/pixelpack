use std::fmt::Debug;

use crate::plater::placer::search::Attempts::Solved;

#[derive(Clone, Debug)]
pub(crate) enum Attempts<T> {
    ToCompute,
    Solved(T),
    Failure,
}

impl<T> Into<Option<T>> for Attempts<T> {
    fn into(self) -> Option<T> {
        match self {
            Solved(x) => Some(x),
            _ => None,
        }
    }
}

pub(crate) fn exponential_search_simple(
    limit: usize,
    mut run: impl FnMut(usize) -> bool,
    search_limit: Option<usize>,
) -> Option<usize> {
    let mut first_found_solution = None;

    let mut i = 1;
    let mut lower = i;

    while i < limit {
        if run(i) {
            first_found_solution = Some(i);
            break;
        }

        if i * 2 >= limit {
            break;
        }

        lower = i;
        i *= 2;
    }

    let lo = lower as usize;
    let hi;

    if let Some(x) = first_found_solution {
        hi = x;
    } else {
        hi = limit;
    }

    binary_search_with_bound(lo, hi, run, search_limit)
}

pub(crate) fn binary_search(lo: usize, hi: usize, run: impl FnMut(usize) -> bool) -> Option<usize> {
    binary_search_with_bound(lo, hi, run, None)
}

// Binary search, but relax the constrain for a non boundary solution, but any solution
pub(crate) fn binary_search_with_bound(
    mut lo: usize,
    mut hi: usize,
    mut run: impl FnMut(usize) -> bool,
    limit: Option<usize>,
) -> Option<usize> {
    let mut iter_count = 0;
    let mut solution = None;
    while lo <= hi {
        iter_count += 1;

        if let Some(lim) = limit {
            if iter_count > lim {
                if solution.is_some() {
                    return solution;
                }
            }
        }

        let gap = hi - lo;
        let mid = lo + gap / 2;

        if run(mid) {
            solution = Some(mid);
            if mid == 1 || !run(mid - 1) {
                return Some(mid);
            }
            hi = mid - 1;
        } else {
            lo = mid + 1;
        }
    }

    None
}

#[cfg(test)]
mod tests1 {
    use crate::plater::placer::search::exponential_search_simple;

    #[test]
    fn test() {
        let f = |x: usize| x >= 10;
        assert_eq!(exponential_search_simple(1000, f, None), Some(10));
    }

    #[test]
    fn test2() {
        let f = |x: usize| x >= 2;
        assert_eq!(exponential_search_simple(1000, f, None), Some(2));
    }

    #[test]
    fn test3() {
        let f = |x: usize| x > 512;
        assert_eq!(exponential_search_simple(1000, f, None), Some(513));
    }

    #[test]
    fn test4() {
        let f = |x: usize| false;
        assert_eq!(exponential_search_simple(1000, f, None), None);
    }
}
