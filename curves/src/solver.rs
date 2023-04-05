use crate::*;

pub fn sqr<T: Float>(x: T) -> T {
    x * x
}

pub fn solve_square_equation<T: Float>(equation: &(T, T, T), eps: T) -> Vec<T> {
    let b2 = sqr(equation.1);
    let ac4 = T::from(4.0).unwrap() * equation.0 * equation.2;
    if b2 * (T::one() + eps + eps) < ac4 {
        // d < 0
        vec![]
    } else if b2 * (T::one() - eps - eps) <= ac4 {
        // d=0
        //-b/2a
        if equation.0.abs() <= equation.1.abs() * eps {
            vec![]
        } else {
            vec![equation.1 / (-equation.0 - equation.0)]
        }
    } else {
        if equation.0 == T::zero() {
            // a=0
            if equation.1 == T::zero() {
                // but b=0
                vec![]
            } else {
                // a=0 => -c/b
                vec![equation.2 / -equation.1]
            }
        } else if equation.2 == T::zero() {
            // c=0 => [0, -b/2]
            vec![T::zero(), equation.1 / -equation.0]
        } else {
            let sd = (b2 - ac4).sqrt();
            let sum = if equation.1 > T::zero() {
                equation.1 + sd
            } else {
                equation.1 - sd
            };
            vec![
                -sum / (equation.0 + equation.0),
                -(equation.2 + equation.2) / sum,
            ]
        }
    }
}

fn intersection_line_case<T: Float>(c1: &Contour<T>, c2: &Contour<T>, eps: T) -> Option<Point<T>> {
    let det = cross(c1.n, c2.n);
    if det.abs() < eps {
        return None;
    }

    Some(
        Point::new(c1.n.y * c2.c - c2.n.y * c1.c, c2.n.x * c1.c - c1.n.x * c2.c).scale(det.recip()),
    )
}

fn intersection_small_circle_case<T: Float>(
    c1: &Contour<T>,
    c2: &Contour<T>,
    eps: T,
) -> Vec<Point<T>> {
    let c1_center = c1.get_center();
    let c1_radius = c1.get_radius();
    let distance = c2.distance(c1_center);

    if distance > c1_radius + eps || distance < -c1_radius - eps {
        return vec![];
    }

    let direction = -c2.n - c1_center.scale(c2.a + c2.a);
    let direction_sqr_len = direction.sqr_length();

    if direction_sqr_len < eps * eps {
        return vec![];
    }

    let direction_normalized = direction.scale(direction_sqr_len.sqrt().recip());

    if distance > c1_radius * (T::one() - eps) {
        // out tangent
        return vec![c1_center + direction_normalized.scale(c1_radius)];
    }

    if distance < -c1_radius * (T::one() - eps) {
        // inner tangent
        if c1.a == c2.a {
            return vec![];
        }
        return vec![c1_center - direction_normalized.scale(c1_radius)];
    }

    let direction_len = direction.length();
    let dist_to_chorde = (c1_radius * c1_radius * c2.a
        + (direction_len + T::one()) * distance * T::from(0.5).unwrap())
        / direction_len;

    let sqr_height = c1_radius * c1_radius - dist_to_chorde * dist_to_chorde;
    let height = if sqr_height < T::zero() {
        T::zero()
    } else {
        sqr_height.sqrt()
    };

    let middle = c1_center + direction_normalized.scale(dist_to_chorde);
    let perp = direction_normalized.rot90().scale(height);

    vec![middle + perp, middle - perp]
}

fn intersection_big_circle_case<T: Float>(
    c1: &Contour<T>,
    c2: &Contour<T>,
    eps: T,
) -> Vec<Point<T>> {
    let common_chorde = (
        c1.n.scale(c2.a) - c2.n.scale(c1.a),
        c1.c * c2.a - c2.c * c1.a,
    );
    let common_n_sqr_len = common_chorde.0.sqr_length();

    if common_n_sqr_len <= sqr(eps * common_chorde.1) {
        return vec![];
    }

    let det = common_n_sqr_len.sqrt().recip();
    let common_chorde = (common_chorde.0.scale(det), common_chorde.1 * det);
    let x0 = common_chorde.0.scale(-common_chorde.1);
    let v = common_chorde.0.rot90();
    let roots = solve_square_equation(&(c1.a, dot(c1.n, v), c1.get_value(x0)), eps);
    if roots.is_empty() {
        return vec![];
    }

    roots.iter().map(|t| x0 + v.scale(*t)).collect()
}

pub fn intersection_contours<T: Float>(c1: &Contour<T>, c2: &Contour<T>, eps: T) -> Vec<Point<T>> {
    if c1.a < c2.a {
        return intersection_contours(c2, c1, eps);
    }

    if c1.a == T::zero() {
        return match intersection_line_case(c1, c2, eps) {
            Some(p) => vec![p],
            None => vec![],
        };
    }

    let as_circles = intersection_small_circle_case(c1, c2, eps);
    let nearest_root = match as_circles.len() {
        0 => return vec![],
        1 => as_circles[0],
        2 => {
            if as_circles[0].sqr_length() < as_circles[1].sqr_length() {
                as_circles[0]
            } else {
                as_circles[1]
            }
        }
        _ => panic!("Can not find more than 2 roots!"),
    };
    if nearest_root.sqr_length() < sqr(c1.get_radius()) * T::from(0.01).unwrap() {
        intersection_big_circle_case(&c1, &c2, eps)
    } else {
        as_circles
    }
}

pub fn intersection_curves<T: Float>(c1: &Curve<T>, c2: &Curve<T>, eps: T) -> Vec<Point<T>> {
    intersection_contours(c1.get_contour(), c2.get_contour(), eps).into_iter().filter(|p|
      match c1 {
        Segment(s) => s.inside_sector(*p, eps, false),
        _ => true
      } &&
      match c2 {
        Segment(s) => s.inside_sector(*p, eps, false),
        _ => true
      }
     ).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, SeedableRng};

    #[test]
    fn test_square_equation() {
        for i in 0..10_000 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(i);
            let root1 = rng.gen_range(-1.0..1.0);
            let root2 = root1 + rng.gen_range(1.0e-3..2.0);
            let a = 10.0.pow(rng.gen_range(-10.0..10.0)) as f32;
            let mut le =
                solve_square_equation::<f32>(&(a, -a * (root1 + root2), a * root1 * root2), 1.0e-7);
            le.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(le.len(), 2);
            assert!((le[0] - root1).abs() < 1.0e-3);
            assert!((le[1] - root2).abs() < 1.0e-3);
        }
        for i in 0..10_000 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(i);
            let root1 = rng.gen_range(-1.0..1.0);
            let a = 10.0.pow(rng.gen_range(-10.0..10.0)) as f32;
            let mut le =
                solve_square_equation::<f32>(&(a, -a * (root1 + root1), a * root1 * root1), 1.0e-7);
            le.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(le.len(), 1);
            assert!((le[0] - root1).abs() < 1.0e-3);
        }
        for i in 0..10_000 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(i);
            let root1 = rng.gen_range(-1.0..1.0);
            let a = 10.0.pow(rng.gen_range(-10.0..10.0)) as f32;
            let mut le = solve_square_equation::<f32>(&(0.0, -a, a * root1), 1.0e-7);
            le.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(le.len(), 1);
            assert!((le[0] - root1).abs() < 1.0e-3);
        }
        assert_eq!(
            solve_square_equation::<f32>(&(0.0, 0.0, 0.0), 1.0e-7),
            vec![]
        );
        assert_eq!(
            solve_square_equation::<f32>(&(0.0, 0.0, 1.0), 1.0e-7),
            vec![]
        );
        assert_eq!(
            solve_square_equation::<f32>(&(0.0, 1.0, 0.0), 1.0e-7),
            vec![0.0]
        );
    }

    fn compare<T: Float + std::fmt::Debug>(res1: &[Point<T>], res2: &[Point<T>], eps: T) {
        assert_eq!(res1.len(), res2.len());
        if res1.len() == 1 {
            assert!((res1[0] - res2[0]).length() < eps);
        } else if res1.len() == 2 {
            assert!(
                ((res1[0] - res2[0]).length() < eps && (res1[1] - res2[1]).length() < eps)
                    || ((res1[0] - res2[1]).length() < eps && (res1[1] - res2[0]).length() < eps)
            );
        }
    }

    #[test]
    fn test_tangent() {
        let curve1 = Contour::<f32>::circle(Point::new(-1.0, 0.0), 1.0);
        let curve2 = Contour::<f32>::circle(Point::new(1.0, 0.0), 1.0);
        compare(
            &intersection_contours(&curve1, &curve2, 1.0e-6),
            &[Point::new(0.0, 0.0)],
            1.0e-6,
        );

        let curve1 = Contour::<f32>::circle(Point::new(999999.0, 780.0), 1.0);
        let curve2 = Contour::<f32>::circle(Point::new(999997.0, 780.0), 1.0);
        compare(
            &intersection_contours(&curve1, &curve2, 1.0e-6),
            &[Point::new(999998.0, 780.0)],
            1.0e-6,
        );

        let curve1 = Contour::<f32>::circle(Point::new(2.0, 0.0), 2.0);
        let curve2 = Contour::<f32>::circle(Point::new(1.0, 0.0), 1.0);
        compare(
            &intersection_contours(&curve1, &curve2, 1.0e-6),
            &[Point::new(0.0, 0.0)],
            1.0e-6,
        );

        let curve1 = Contour::<f32>::circle(Point::new(0.0, 600.0), 600.0);
        let curve2 = Contour::<f32>::circle(Point::new(-0.1, 600.0), 600.1);
        compare(
            &intersection_contours(&curve1, &curve2, 1.0e-6),
            &[Point::new(600.0, 600.0)],
            1.0e-4,
        );

        let curve1 = Contour::<f32>::circle(Point::new(8000.0, 6000.0), 10000.0);
        let curve2 = Contour::<f32>::circle(Point::new(8000.06, 6000.08), 9999.9);
        compare(
            &intersection_contours(&curve1, &curve2, 1.0e-6),
            &[Point::new(14000.0, 14000.0)],
            15.0, // bad case
        );

        let curve1 = Contour::<f64>::circle(Point::new(8000.0, 6000.0), 10000.0);
        let curve2 = Contour::<f64>::circle(Point::new(8000.06, 6000.08), 9999.9);
        compare(
            &intersection_contours(&curve1, &curve2, 1.0e-10),
            &[Point::new(14000.0, 14000.0)],
            1.0e-7,
        );

        let curve1 = Contour::<f32>::circle(Point::new(8000.0, 6000.0), 10000.0);
        let curve2 = Contour::<f32>::circle(Point::new(8000.6, 6000.8), 9999.0);
        compare(
            &intersection_contours(&curve1, &curve2, 1.0e-6),
            &[Point::new(14000.0, 14000.0)],
            5.0, // bad case
        );

        let curve1 = Contour::<f32>::circle(Point::new(8000.0, 6000.0), 10000.0);
        let curve2 = Contour::<f32>::circle(Point::new(8006.0, 6008.0), 9990.0);
        compare(
            &intersection_contours(&curve1, &curve2, 1.0e-6),
            &[Point::new(14000.0, 14000.0)],
            0.5, // bad case
        );

        let curve1 = Contour::<f32>::line(Point::new(-1.0e-7, 1.0), Point::new(-1.0e-7, -1.0))
            .inverse()
            .translate(Point::new(0.1, 420.0));
        let curve2 = Contour::<f32>::line(Point::new(1.0e-7, 1.0), Point::new(1.0e-7, -1.0))
            .inverse()
            .translate(Point::new(0.1, 420.0));

        compare(
            &intersection_contours(&curve1, &curve2, 1.0e-6),
            &[Point::new(0.1, 420.0)],
            1.0e-7, // bad case, but we lucky, x coord is exact
        );
    }

    #[test]
    fn test_intersection() {
        let curve1 = Contour::<f32>::circle(Point::new(4.0, 0.0), 5.0);
        let curve2 = Contour::<f32>::circle(Point::new(-4.0, 0.0), 5.0);
        compare(
            &intersection_contours(&curve1, &curve2, 1.0e-6),
            &[Point::new(0.0, -3.0), Point::new(0.0, 3.0)],
            1.0e-5,
        );

        let curve1 = Contour::<f32>::circle(Point::new(7.0, 9999.0), 15.0);
        let curve2 = Contour::<f32>::circle(Point::new(-7.0, 9999.0), 13.0);
        compare(
            &intersection_contours(&curve1, &curve2, 1.0e-6),
            &[Point::new(-2.0, 10011.0), Point::new(-2.0, 9987.0)],
            1.0e-7,
        );

        let curve1 = Contour::<f32>::line(Point::new(5.01, 9999.0), Point::new(5.0, 9999.0));
        let curve2 = Contour::<f32>::circle(Point::new(-7.0, 9999.0), 14.0);
        compare(
            &intersection_contours(&curve1, &curve2, 1.0e-6),
            &[Point::new(-21.0, 9999.0), Point::new(7.0, 9999.0)],
            1.0e-5,
        );

        let mut curve1 = Contour::<f32>::circle(Point::new(8e+8, 6e+8), 10e+8);
        curve1.c = 0.0;
        let mut curve2 = Contour::<f32>::circle(Point::new(-8e+8, 6e+8), 10e+8);
        curve2.c = 0.0;
        curve1 = curve1.translate(Point::new(0.0, 0.1));
        curve2 = curve2.translate(Point::new(0.0, 0.1));
        compare(
            &intersection_contours(&curve1, &curve2, 1.0e-6),
            &[Point::new(0.0, 0.1), Point::new(0.0, 12e+8)],
            1.0e-5,
        );

        let curve1 = Contour::<f32>::line(Point::new(5.032, 15.012), Point::new(5.0, 0.0));
        let curve2 = Contour::<f32>::line(Point::new(5.032, 15.012), Point::new(5.01, 0.0));
        compare(
            &intersection_contours(&curve1, &curve2, 1.0e-6),
            &[Point::new(5.032, 15.012)],
            1.0e-3,
        );
    }
}
