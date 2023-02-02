use crate::plater::placer::score::Position::{Inside, Outside};
use crate::plater::placer::score::Preference::{First, NoPreference, Second};

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum Preference {
    First,
    Second,
    NoPreference,
}

impl Preference {
    fn defer_to<F>(self, f: F) -> Preference
        where
            F: Fn() -> Preference
    {
        match self {
            NoPreference => f(),
            pref => pref
        }
    }
}

pub(crate) trait Prefer: Copy {
    fn compare_prefer(self, other: Self) -> Preference;
}

impl Prefer for Position {
    fn compare_prefer(self, other: Self) -> Preference {
        match (self, other) {
            (Inside, Inside) => NoPreference,
            (Outside, Inside) => Second,
            (Inside, Outside) => First,
            (Outside, Outside) => NoPreference,
        }
    }
}

impl Prefer for FloatWrapper {
    fn compare_prefer(self, other: Self) -> Preference {
        let FloatWrapper(n) = self;
        let FloatWrapper(m) = other;

        if f64::abs(n - m) < 0.1 {
            NoPreference
        } else if n < m {
            First
        } else {
            Second
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct FloatWrapper(pub(crate) f64);

#[derive(Copy, Clone)]
pub(crate) enum Position {
    Inside,
    Outside,
}

#[derive(Copy, Clone)]
pub(crate) struct Score {
    pub(crate) position: Position,
    pub(crate) moment_of_inertial: FloatWrapper,
    pub(crate) x_pos: FloatWrapper,
    pub(crate) y_pos: FloatWrapper,
}

impl Prefer for Score {
    fn compare_prefer(self, other: Self) -> Preference {
        let cmp_position = || self.position.compare_prefer(other.position);
        let cmp_inertia = || self.moment_of_inertial.compare_prefer(other.moment_of_inertial);
        let cmp_x = || self.x_pos.compare_prefer(other.x_pos);
        let cmp_y = || self.y_pos.compare_prefer(other.y_pos);

        NoPreference
            .defer_to(cmp_position)
            .defer_to(cmp_inertia)
            .defer_to(cmp_x)
            .defer_to(cmp_y)
    }
}