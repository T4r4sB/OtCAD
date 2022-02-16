use num::traits::*;
use std::ops::*;

#[derive(Debug, Copy, Clone)]
pub struct NumberWithDistortion<T: Float> {
  pub value: T,
  pub distortion: T,
}

impl<T: Float> NumberWithDistortion<T> {
  pub fn new(value: T) -> Self {
    Self{value, distortion: value.abs()}
  }
}

impl<T: Float> Add for NumberWithDistortion<T> {
  type Output = Self;

  fn add(self, other: Self) -> Self::Output {
      Self {
        value: self.value + other.value,
        distortion: self.distortion + other.distortion,
      }
  }
}

impl<T: Float> Sub for NumberWithDistortion<T> {
  type Output = Self;

  fn sub(self, other: Self) -> Self::Output {
      Self {
        value: self.value - other.value,
        distortion: self.distortion + other.distortion,
      }
  }
}

impl<T: Float> Neg for NumberWithDistortion<T> {
  type Output = Self;

  fn neg(self) -> Self::Output {
    Self {
      value: -self.value,
      distortion: self.distortion,
    }
  }
}

impl<T: Float> Mul for NumberWithDistortion<T> {
  type Output = Self;

  fn mul(self, other: Self) -> Self::Output {
      Self {
        value: self.value * other.value,
        distortion: (self.distortion * other.value).abs() + (other.distortion * self.value).abs(),
      }
  }
}

impl<T: Float> Div for NumberWithDistortion<T> {
  type Output = Self;

  fn div(self, other: Self) -> Self::Output {
      Self {
        value: self.value / other.value,
        distortion: ((self.distortion * other.value).abs() + (other.distortion * self.value).abs()) / (other.value * other.value),
      }
  }
}
