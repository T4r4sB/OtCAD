use num::traits::*;
use serde::{Deserialize, Serialize};
use std::ops::*;

#[derive(Default, Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct Point<T: Float> {
    pub x: T,
    pub y: T,
}

impl<T: Float> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn angle(a: T) -> Self {
        Self {
            x: T::cos(a),
            y: T::sin(a),
        }
    }

    pub fn sqr_length(self) -> T {
        self.x * self.x + self.y * self.y
    }

    pub fn length(self) -> T {
        self.sqr_length().sqrt()
    }

    pub fn rot90(self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    pub fn scale(self, factor: T) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }

    pub fn normalize(self) -> Self {
        self.scale(self.length().recip())
    }

    pub fn complex_conj(self) -> Self {
        Self {
            x: self.x,
            y: -self.y,
        }
    }

    pub fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

pub fn dot<T: Float>(a: Point<T>, b: Point<T>) -> T {
    a.x * b.x + a.y * b.y
}

pub fn cross<T: Float>(a: Point<T>, b: Point<T>) -> T {
    a.x * b.y - a.y * b.x
}

pub fn complex_mul<T: Float>(a: Point<T>, b: Point<T>) -> Point<T> {
    Point {
        x: a.x * b.x - a.y * b.y,
        y: a.x * b.y + a.y * b.x,
    }
}

impl<T: Float> Add for Point<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: Float> Sub for Point<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Float> Neg for Point<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}
