use num::traits::*;
use crate::points::*;

#[derive(Debug, Copy, Clone)]
pub struct Curve<T: Float> {
  // Equation a*<x,x> + <n,x> + c = 0
  // Constraints: <n,n> - 4*a*c = 1.0; a >= 0.0
  pub a: T,
  pub n: Point<T>,
  pub c: T,
}

impl<T: Float> Curve<T> {
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

    Self {a, n, c}
  }

  pub fn zero() -> Self {
    Self {a: T::zero(), n: Point::new(T::zero(), T::zero()), c: T::zero()}
  }

  pub fn neg(self) -> Self {
    Self {a: -self.a, n: -self.n, c: -self.c}
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
    let n = self.n + delta.scale(a + a);
    let num = n.sqr_length() - T::one();
    let c = if num.abs() > T::from(0.01).unwrap() {
      num / (a + a + a + a)
    } else {
      a * delta.sqr_length() + dot(n, delta) + self.c
    };

    Self {a,n,c}
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

    Some(Self{a,n,c})
  }

  pub fn discriminant(&self) -> T {
    self.n.sqr_length() - (T::one() + T::one() + T::one() + T::one()) * self.a * self.c
  }

  pub fn get_value(&self, x: Point<T>) -> T {
    x.sqr_length() * self.a + dot(x, self.n) + self.c
  }
}

#[derive(Debug, Copy, Clone)]
pub struct CurveSegment<T: Float> {
  pub curve: Curve<T>,
  pub begin: Point<T>,
  pub end: Point<T>,
  pub big: bool, // is arc bigger than 180 degrees
}

impl<T: Float> CurveSegment<T> {
  pub fn begin_direction(&self) -> Point<T> {
    // begin + n/2a in common case
    self.begin.scale(self.curve.a + self.curve.a) + self.curve.n
  }

  pub fn end_direction(&self) -> Point<T> {
    self.end.scale(self.curve.a + self.curve.a) + self.curve.n
  }

  pub fn scale(self, factor: T) -> Self {
    Self {
      curve: self.curve.scale(factor),
      begin: self.begin.scale(factor),
      end: self.end.scale(factor),
      big: self.big,
    }
  }
}


#[derive(Debug, Copy, Clone)]
pub enum Entity<T: Float> {
  Curve(Curve<T>),
  CurveSegment(CurveSegment<T>),
}

impl<T: Float> Entity<T> {
  pub fn get_curve(&self) -> &Curve<T> {
    match self {
      Entity::Curve(c) => &c,
      Entity::CurveSegment(s) => &s.curve,
    }
  }
  
  pub fn scale(&self, factor: T) -> Self {
    match self {
      Entity::Curve(c) => Entity::Curve(c.scale(factor)),
      Entity::CurveSegment(s) => Entity::CurveSegment(s.scale(factor)),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_circle() {
    let curve = Curve::<f32>::circle(Point::new(1.0, 0.0), 0.0001);
    // can not check discriminant of curve because of big distortion
    let curve_shifted = curve.translate(Point::new(1.0, 0.0));
    assert!((curve_shifted.discriminant() - 1.0).abs() < 0.0001);
    let another_radius = curve.change_radius(0.001).unwrap();
    assert!((another_radius.discriminant() - 1.0).abs() < 0.0001);
    let another_radius = curve_shifted.change_radius(0.001).unwrap();
    assert!((another_radius.discriminant() - 1.0).abs() < 0.0001);
  }

  #[test]
  fn test_line() {
    let curve = Curve::<f32>::line(Point::new(1.0, 0.0), Point::new(1.0, 1.0));
    assert!((curve.discriminant() - 1.0).abs() < 0.0001);
    let curve_shifted = curve.translate(Point::new(1.0, 0.0));
    assert!((curve_shifted.discriminant() - 1.0).abs() < 0.0001);
    let another_radius = curve.change_radius(0.001).unwrap();
    assert!((another_radius.discriminant() - 1.0).abs() < 0.0001);
    let another_radius = curve.change_radius(-0.001).unwrap();
    assert!((another_radius.discriminant() - 1.0).abs() < 0.0001);
  }
}