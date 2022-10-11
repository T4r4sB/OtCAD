pub mod points;
pub mod render;
pub mod solver;

use crate::points::*;
use num::traits::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct CLoCC<T: Float> {
    // Equation a*<x,x> + <n,x> + c = 0
    // Constraints: <n,n> - 4*a*c = 1.0; a >= 0.0
    pub a: T,
    pub n: Point<T>,
    pub c: T,
}

impl<T: Float> CLoCC<T> {
    pub fn line(x: Point<T>, y: Point<T>) -> Self {
        let normal = (x - y).rot90().normalize();
        Self {
            a: T::zero(),
            n: normal,
            c: -dot(x, normal),
        }
    }

    pub fn circle(center: Point<T>, radius: T) -> Self {
        let a = (radius + radius).recip();
        let n = center.scale(-a - a);
        let num = n.sqr_length() - T::one();
        let c = if num.abs() > T::from(0.01).unwrap() {
            num / (a + a + a + a)
        } else {
            a * (center.sqr_length() - radius * radius)
        };

        Self { a, n, c }
    }

    pub fn zero() -> Self {
        Self {
            a: T::zero(),
            n: Point::new(T::zero(), T::zero()),
            c: T::zero(),
        }
    }

    pub fn neg(self) -> Self {
        Self {
            a: -self.a,
            n: -self.n,
            c: -self.c,
        }
    }

    pub fn scale(&self, factor: T) -> Self {
        Self {
            a: self.a / factor,
            n: self.n,
            c: self.c * factor,
        }
    }

    pub fn complex_mul(&self, t: Point<T>) -> Self {
        Self {
            a: self.a,
            n: complex_mul(self.n, t),
            c: self.c * t.sqr_length(),
        }
    }

    pub fn translate(&self, delta: Point<T>) -> Self {
        let a = self.a;
        let n = self.n - delta.scale(a + a);
        let num = n.sqr_length() - T::one();
        let c = if num.abs() > T::from(0.01).unwrap() {
            num / (a + a + a + a)
        } else {
            -a * delta.sqr_length() - dot(n, delta) + self.c
        };

        Self { a, n, c }
    }

    pub fn change_radius(&self, delta: T) -> Option<Self> {
        let det = (T::one() + self.a * delta + self.a * delta).recip();
        if !det.is_finite() || det < T::zero() {
            return None;
        }

        let a = self.a * det;
        let n = self.n.scale(det);
        let num = n.sqr_length() - T::one();
        let c = if num.abs() > T::from(0.01).unwrap() {
            num / (a + a + a + a)
        } else {
            (self.c - delta - self.a * delta * delta) * det
        };

        Some(Self { a, n, c })
    }

    pub fn discriminant(&self) -> T {
        self.n.sqr_length() - T::from(4.0).unwrap() * self.a * self.c
    }

    pub fn get_value(&self, x: Point<T>) -> T {
        x.sqr_length() * self.a + dot(x, self.n) + self.c
    }

    pub fn distance(&self, x: Point<T>) -> T {
        self.get_value(x)
            / ((x.scale(self.a) + self.n.scale(T::from(0.5).unwrap())).length()
                + T::from(0.5).unwrap())
    }

    pub fn check_right(&self, x: T) -> bool {
        if self.n.x < T::zero() || self.n.x > self.n.y.abs() {
            -self.n.x + T::one() < x * T::from(2.0).unwrap() * self.a
        } else {
            self.n.y * self.n.y - T::from(4.0).unwrap() * self.a * self.c
                < x * T::from(2.0).unwrap() * self.a * (self.n.x + T::one())
        }
    }

    pub fn check_bottom(&self, y: T) -> bool {
        if self.n.y < T::zero() || self.n.y > self.n.x.abs() {
            -self.n.y + T::one() < y * T::from(2.0).unwrap() * self.a
        } else {
            self.n.x * self.n.x - T::from(4.0).unwrap() * self.a * self.c
                < y * T::from(2.0).unwrap() * self.a * (self.n.y + T::one())
        }
    }

    pub fn check_left(&self, x: T) -> bool {
        if self.n.x > T::zero() || self.n.x < -self.n.y.abs() {
            -self.n.x - T::one() > x * T::from(2.0).unwrap() * self.a
        } else {
            self.n.y * self.n.y - T::from(4.0).unwrap() * self.a * self.c
                < x * T::from(2.0).unwrap() * self.a * (self.n.x - T::one())
        }
    }

    pub fn check_top(&self, y: T) -> bool {
        if self.n.y > T::zero() || self.n.y < -self.n.x.abs() {
            -self.n.y - T::one() > y * T::from(2.0).unwrap() * self.a
        } else {
            self.n.x * self.n.x - T::from(4.0).unwrap() * self.a * self.c
                < y * T::from(2.0).unwrap() * self.a * (self.n.y - T::one())
        }
    }

    pub fn in_rect(&self, corner1: Point<T>, corner2: Point<T>) -> bool {
        self.check_right(T::max(corner1.x, corner2.x))
            && self.check_bottom(T::max(corner1.y, corner2.y))
            && self.check_left(T::min(corner1.x, corner2.x))
            && self.check_top(T::min(corner1.y, corner2.y))
    }

    pub fn sqr_distance_to_center(&self, position: Point<T>, sqr_max: T) -> Option<(T, Point<T>)> {
        //(p-n/2a).length()
        let factor = self.a * T::from(2.0).unwrap();
        let point = position.scale(factor) + self.n;
        let value = point.sqr_length();
        if value < (factor * factor) * sqr_max {
            Some((value / (factor * factor), self.n.scale(-factor.recip())))
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct SoCC<T: Float> {
    pub clocc: CLoCC<T>,
    pub begin: Point<T>,
    pub end: Point<T>,
    pub big: bool, // is arc bigger than 180 degrees
}

impl<T: Float> SoCC<T> {
    pub fn line(x: Point<T>, y: Point<T>) -> Self {
        Self {
            clocc: CLoCC::line(x, y),
            begin: x,
            end: y,
            big: false,
        }
    }

    pub fn begin_direction(&self) -> Point<T> {
        // begin + n/2a in common case
        self.begin.scale(self.clocc.a + self.clocc.a) + self.clocc.n
    }

    pub fn end_direction(&self) -> Point<T> {
        self.end.scale(self.clocc.a + self.clocc.a) + self.clocc.n
    }

    pub fn translate(&self, delta: Point<T>) -> Self {
        Self {
            clocc: self.clocc.translate(delta),
            begin: self.begin + delta,
            end: self.end + delta,
            big: self.big,
        }
    }

    pub fn scale(self, factor: T) -> Self {
        Self {
            clocc: self.clocc.scale(factor),
            begin: self.begin.scale(factor),
            end: self.end.scale(factor),
            big: self.big,
        }
    }

    pub fn inside_sector(&self, x: Point<T>) -> bool {
        if self.big {
            cross(x - self.begin, self.begin_direction()) < T::zero()
                || cross(x - self.end, self.end_direction()) > T::zero()
        } else {
            cross(x - self.begin, self.begin_direction()) < T::zero()
                && cross(x - self.end, self.end_direction()) > T::zero()
        }
    }

    pub fn distance(&self, x: Point<T>) -> T {
        if self.inside_sector(x) {
            self.clocc.distance(x)
        } else {
            T::min((x - self.begin).sqr_length(), (x - self.end).sqr_length()).sqrt()
        }
    }

    pub fn in_rect(&self, corner1: Point<T>, corner2: Point<T>) -> bool {
        let x1 = T::min(corner1.x, corner2.x);
        let y1 = T::min(corner1.y, corner2.y);
        let x2 = T::max(corner1.x, corner2.x);
        let y2 = T::max(corner1.y, corner2.y);
        if self.begin.x < x1
            || self.begin.x > x2
            || self.begin.y < y1
            || self.begin.y > y2
            || self.end.x < x1
            || self.end.x > x2
            || self.end.y < y1
            || self.end.y > y2
        {
            return false;
        }

        let begin_l = self.begin.x * T::from(2.0).unwrap() * self.clocc.a < self.clocc.n.x;
        let begin_t = self.begin.y * T::from(2.0).unwrap() * self.clocc.a < self.clocc.n.y;
        let end_l = self.begin.x * T::from(2.0).unwrap() * self.clocc.a < self.clocc.n.x;
        let end_t = self.begin.y * T::from(2.0).unwrap() * self.clocc.a < self.clocc.n.y;

        if self.big {
            ((begin_t && !end_t) || self.clocc.check_right(x2))
                && ((!begin_l && end_l) || self.clocc.check_bottom(x2))
                && ((!begin_t && end_t) || self.clocc.check_left(x2))
                && ((begin_l && !end_l) || self.clocc.check_top(x2))
        } else {
            (begin_t || !end_t || self.clocc.check_right(x2))
                && (!begin_l || end_l || self.clocc.check_bottom(x2))
                && (!begin_t || end_t || self.clocc.check_left(x2))
                && (begin_l || !end_l || self.clocc.check_top(x2))
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum LoCC<T: Float> {
    CLoCC(CLoCC<T>),
    SoCC(SoCC<T>),
}

use LoCC::*;

impl<T: Float> LoCC<T> {
    pub fn get_clocc(&self) -> &CLoCC<T> {
        match self {
            CLoCC(c) => &c,
            SoCC(s) => &s.clocc,
        }
    }

    pub fn translate(&self, delta: Point<T>) -> Self {
        match self {
            CLoCC(c) => CLoCC(c.translate(delta)),
            SoCC(s) => SoCC(s.translate(delta)),
        }
    }

    pub fn scale(&self, factor: T) -> Self {
        match self {
            CLoCC(c) => CLoCC(c.scale(factor)),
            SoCC(s) => SoCC(s.scale(factor)),
        }
    }

    pub fn distance(&self, x: Point<T>) -> T {
        match self {
            CLoCC(c) => c.distance(x),
            SoCC(s) => s.distance(x),
        }
    }

    pub fn in_rect(&self, corner1: Point<T>, corner2: Point<T>) -> bool {
        match self {
            CLoCC(c) => c.in_rect(corner1, corner2),
            SoCC(s) => s.in_rect(corner1, corner2),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle() {
        let curve = CLoCC::<f32>::circle(Point::new(1.0, 0.0), 0.0001);
        // can not check discriminant of curve because of big distortion
        let curve_shifted = curve.translate(Point::new(-1.0, 0.0));
        assert!((curve_shifted.discriminant() - 1.0).abs() < 0.0001);
        let another_radius = curve.change_radius(0.001).unwrap();
        assert!((another_radius.discriminant() - 1.0).abs() < 0.0001);
        let another_radius = curve_shifted.change_radius(0.001).unwrap();
        assert!((another_radius.discriminant() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_line() {
        let curve = CLoCC::<f32>::line(Point::new(1.0, 0.0), Point::new(1.0, 1.0));
        assert!((curve.discriminant() - 1.0).abs() < 0.0001);
        let curve_shifted = curve.translate(Point::new(1.0, 0.0));
        assert!((curve_shifted.discriminant() - 1.0).abs() < 0.0001);
        let another_radius = curve.change_radius(0.001).unwrap();
        assert!((another_radius.discriminant() - 1.0).abs() < 0.0001);
        let another_radius = curve.change_radius(-0.001).unwrap();
        assert!((another_radius.discriminant() - 1.0).abs() < 0.0001);
    }
}
