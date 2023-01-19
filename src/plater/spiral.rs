use std::cmp::{max, min};

enum StraightLineIter {
    XIter { cur: InclusiveRange, x: isize },
    YIter { cur: InclusiveRange, y: isize },
    Empty,
}

#[derive(PartialEq, Debug)]
struct InclusiveRange {
    start: isize,
    stop: isize,
    step: isize,
}

impl InclusiveRange {
    fn contains(&self, x: isize) -> bool {
        self.start <= x && x <= self.stop
    }

    fn intersects(&self, other: &Self) -> bool {
        self.start <= other.stop && other.start <= self.stop
    }
}

#[derive(Debug)]
struct Rectangle {
    x_range: InclusiveRange,
    y_range: InclusiveRange,
}


// Just check four points

impl Rectangle {
    fn intersection(&self, other: &StraightLine) -> StraightLine {
        match other {
            // max, min
            StraightLine::XFixed { x, ys } => {
                if self.x_range.contains(*x) && ys.intersects(&self.y_range) {
                    StraightLine::XFixed {
                        x: *x,
                        ys: InclusiveRange {
                            start: max(self.y_range.start, ys.start),
                            stop: min(self.y_range.stop, ys.stop),
                            step: ys.step,
                        },
                    }
                } else {
                    StraightLine::Empty
                }
            }
            StraightLine::YFixed { y, xs } => {
                if self.y_range.contains(*y) && xs.intersects(&self.x_range) {
                    StraightLine::YFixed {
                        y: *y,
                        xs: InclusiveRange {
                            start: max(self.x_range.start, xs.start),
                            stop: min(self.x_range.stop, xs.stop),
                            step: xs.step,
                        },
                    }
                } else {
                    StraightLine::Empty
                }
            }
            StraightLine::Empty => StraightLine::Empty
        }
    }
}

#[derive(PartialEq, Debug)]
enum StraightLine {
    XFixed { x: isize, ys: InclusiveRange },
    YFixed { y: isize, xs: InclusiveRange },
    Empty,
}


type Point = (isize, isize);

impl StraightLine {
    fn new(x: Point, y: Point) -> Self {
        if x.0 == y.0 {
            StraightLine::XFixed {
                x: x.0,
                ys: InclusiveRange {
                    start: min(x.1, y.1),
                    stop: max(x.1, y.1),
                    step: if x.1 < y.1 { 1 } else { -1 },
                },
            }
        } else if x.1 == y.1 {
            StraightLine::YFixed {
                y: x.1,
                xs: InclusiveRange {
                    start: min(x.0, y.0),
                    stop: max(x.0, y.0),
                    step: if x.0 < y.0 { 1 } else { -1 },
                },
            }
        } else {
            unreachable!()
        }
    }

    fn into_iter(self) -> StraightLineIter {
        match self {
            StraightLine::XFixed { x, ys } => StraightLineIter::XIter { cur: ys, x },
            StraightLine::YFixed { y, xs } => StraightLineIter::YIter { cur: xs, y },
            StraightLine::Empty => StraightLineIter::Empty
        }
    }
}

impl Iterator for StraightLineIter {
    type Item = (isize, isize);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            StraightLineIter::XIter { cur, x } => {
                if let Some(y) = cur.next() {
                    Some((*x, y))
                } else {
                    None
                }
            }
            StraightLineIter::YIter { cur, y } => {
                if let Some(x) = cur.next() {
                    Some((x, *y))
                } else {
                    None
                }
            }
            StraightLineIter::Empty => None,
        }
    }
}


impl Iterator for InclusiveRange {
    type Item = isize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.stop {
            return None;
        }

        if self.step > 0 {
            let res = Some(self.start);
            self.start += 1;
            res
        } else {
            let res = Some(self.stop);
            self.stop -= 1;
            res
        }
    }
}

struct WindowIter<A, T: Iterator<Item=A>, const N: usize> {
    iter: T,
}


impl<A, T: Iterator<Item=A>, const N: usize> WindowIter<A, T, N> {
    fn new(iter: T) -> Self {
        WindowIter { iter }
    }
}

impl<A, T: Iterator<Item=A>, const N: usize> Iterator for WindowIter<A, T, N> {
    type Item = [Option<A>; N];

    fn next(&mut self) -> Option<Self::Item> {
        let mut xs: Self::Item = [(); N].map(|_| None);
        for i in 0..N {
            xs[i] = self.iter.next();
        }

        match xs.iter().any(|x| x.is_some()) {
            true => Some(xs),
            _ => None
        }
    }
}

pub struct RepeatIter<T, const N: usize> {
    values: [T; N],
    index: usize,
}

impl<T, const N: usize> RepeatIter<T, N> {
    pub fn new(values: [T; N]) -> Self {
        if values.is_empty() {
            panic!("Provided an empty array");
        }
        RepeatIter { values, index: 0 }
    }
}

impl<T: Copy, const N: usize> Iterator for RepeatIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.values[self.index];
        self.index = (self.index + 1) % N;
        Some(cur)
    }
}


struct Cons<A, T: Iterator<Item=A>> {
    initial: Option<A>,
    it: T,
}

impl<A, T: Iterator<Item=A>> Iterator for Cons<A, T> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        if self.initial.is_none() {
            self.it.next()
        } else {
            self.initial.take()
        }
    }
}

#[derive(Ord, Eq, PartialOrd, PartialEq)]
struct PairWrapper<A, B> ((A, B));

pub(crate) fn spiral_iterator(delta: f64, width: f64, height: f64, original_width: f64, original_height: f64) -> impl Iterator<Item=(f64, f64)> {
    let d_width = f64::floor(width / delta) as isize;
    let d_height = f64::floor(height / delta) as isize;

    let origin = {
        let (w, h) = {
            if original_width < width {
                (original_width, original_height)
            } else {
                (width, height)
            }
        };

        (f64::floor(w / delta) as isize, f64::floor(h / delta) as isize)
    };


    let rect = Rectangle {
        x_range: InclusiveRange {
            start: 0,
            stop: d_width,
            step: 1,
        },
        y_range: InclusiveRange {
            start: 0,
            stop: d_height,
            step: 1,
        },
    };
    let distances = (1..)
        .flat_map(|n| [n, n]);


    let direction_vectors = RepeatIter::new([(1, 0), (0, -1), (-1, 0), (0, 1)]);

    let points = distances
        .zip(direction_vectors)
        .map(|(dist, direction)| (direction.0 * dist, direction.1 * dist))
        .scan(origin, |st, next| {
            let (a, b) = *st;
            *st = (a + next.0, b + next.1);
            Some(*st)
        });

    let points_with_origin = Cons { initial: Some(origin), it: points };

    let duplicated_points = points_with_origin
        .flat_map(|p| [p, p]).skip(1);

    let windowed = WindowIter::new(duplicated_points)
        .map(|[x, y]| [x.unwrap(), y.unwrap()]);

    let spiral_lines = windowed.map(|[p1, p2]|
        StraightLine::new(p1, p2));


    let grouped_lines = WindowIter::new(spiral_lines)
        .map(move |[a, b, c, d]| {
            let xs = [a.unwrap(), b.unwrap(), c.unwrap(), d.unwrap()];
            let ys = xs.into_iter()
                .map(|x| {
                    let res = rect.intersection(&x);
                    res
                })
                .filter(|x| x.ne(&StraightLine::Empty))
                .collect::<Vec<_>>();

            if ys.is_empty() {
                None
            } else {
                Some(ys)
            }
        })
        .take_while(|x| x.is_some())
        .map(|x| x.unwrap())
        .flatten();

    let spiral = itertools::Itertools::dedup(Cons {
        initial: Some(origin),
        it: grouped_lines.flat_map(|x| {
            x.into_iter()
        }),
    });

    spiral.map(move |(x, y)| {
        (x as f64 * delta, y as f64 * delta)
    })
}

