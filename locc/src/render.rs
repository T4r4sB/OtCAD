use num::traits::*;
use num::NumCast;

use crate::points::*;
use crate::*;
use application::image::*;

fn minimal_neg_value<T: Float>(x1: T, x2: T, a: T, b: T, c: T) -> T {
    let f = |x| (a * x + b) * x + c;

    let mut x1 = x1;
    let mut x2 = x2;
    let mut f1 = f(x1);
    if f1 <= T::zero() {
        return x1;
    }

    let mut f2 = f(x2);
    if f2 > T::zero() {
        return x2;
    }

    let treshold = T::from(0.1).unwrap();

    loop {
        if x1 + treshold + T::one() > x2 {
            return x2;
        }
        let mut mx = (x1 + (x2 - x1) * f1 / (f1 - f2)).ceil();
        if mx + treshold > x2 {
            mx = x2 - T::one();
        }
        while x1 + treshold > mx {
            mx = x1 + T::one();
        }

        let mf = f(mx);
        if mf <= T::zero() {
            x2 = mx;
            f2 = mf;
        } else {
            x1 = mx;
            f1 = mf;
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct QuadFunc<T: Float> {
    dd: T,
    dx: T,
    dy: T,
    v: T,
}

impl<T: Float> QuadFunc<T> {
    fn clocc(clocc: &CLoCC<T>, x0: T, y0: T) -> Self {
        Self {
            dd: clocc.a + clocc.a,
            dx: clocc.n.x + clocc.a * (x0 + x0 + T::one()),
            dy: clocc.n.y + clocc.a * (y0 + y0 + T::one()),
            v: clocc.get_value(Point::new(x0, y0)),
        }
    }

    fn step_y(&mut self) {
        self.v = self.v + self.dy;
        self.dy = self.dy + self.dd;
    }

    fn step_x_left(&mut self) {
        self.dx = self.dx - self.dd;
        self.v = self.v - self.dx;
    }

    fn step_x_right(&mut self) {
        self.v = self.v + self.dx;
        self.dx = self.dx + self.dd;
    }

    fn get_value(&self) -> T {
        self.v
    }
}

#[derive(Debug, Copy, Clone)]
struct ProjectivePoint<T: Float> {
    p: Point<T>,
    a: T,
}

impl<T: Float> ProjectivePoint<T> {
    fn new(p: Point<T>, a: T) -> Self {
        Self { p, a }
    }
}

fn draw_rhombe<T: Float>(
    size: ImageSize,
    top: &ProjectivePoint<T>,
    left: &ProjectivePoint<T>,
    right: &ProjectivePoint<T>,
    bottom: &ProjectivePoint<T>,
    left_top: &CLoCC<T>,
    left_bottom: &CLoCC<T>,
    right_top: &CLoCC<T>,
    right_bottom: &CLoCC<T>,
    span_buffer: &mut [(usize, usize)],
) -> (usize, usize) {
    if right.p.x <= T::zero() || bottom.p.y <= T::zero() {
        return (0, 0);
    }

    let size_x = T::from(size.0).unwrap();
    let size_y = T::from(size.1).unwrap();
    if left.p.x > size_x * left.a || top.p.y > size_y * top.a {
        return (0, 0);
    }

    let unbounded_top = if top.p.y <= T::zero() {
        T::zero()
    } else {
        (top.p.y / top.a).ceil()
    };
    let unbounded_bottom = if bottom.p.y > size_y * bottom.a {
        size_y
    } else {
        T::min(size_y, (bottom.p.y / bottom.a).ceil())
    };

    let y1 = {
        let r = top.p.x > size_x * top.a;
        let l = top.p.x <= T::zero();
        let real_y = unbounded_top;
        if r {
            minimal_neg_value(
                unbounded_top,
                unbounded_bottom,
                left_top.a,
                left_top.n.y,
                left_top.a * size_x * size_x + left_top.n.x * size_x + left_top.c,
            )
        } else if l {
            minimal_neg_value(
                unbounded_top,
                unbounded_bottom,
                right_top.a,
                right_top.n.y,
                right_top.c,
            )
        } else {
            if top.p.y <= T::zero() {
                T::zero()
            } else {
                real_y
            }
        }
    };

    let y2 = {
        let r = bottom.p.x > size_x * bottom.a;
        let l = bottom.p.x <= T::zero();

        let real_y = unbounded_bottom;
        if r {
            minimal_neg_value(
                unbounded_top,
                unbounded_bottom,
                -left_bottom.a,
                -left_bottom.n.y,
                -(left_bottom.a * size_x * size_x + left_bottom.n.x * size_x + left_bottom.c),
            )
        } else if l {
            minimal_neg_value(
                unbounded_top,
                unbounded_bottom,
                -right_bottom.a,
                -right_bottom.n.y,
                -right_bottom.c,
            )
        } else {
            if bottom.p.y > size_y * bottom.a {
                size_y
            } else {
                real_y
            }
        }
    };

    let get_y = |p: &ProjectivePoint<T>| -> usize {
        if p.p.y <= T::zero() {
            0
        } else if p.p.y > size_y * p.a {
            size.1
        } else {
            NumCast::from((p.p.y / p.a).ceil()).unwrap()
        }
    };

    let unbounded_left = if left.p.x <= T::zero() {
        T::zero()
    } else {
        (left.p.x / left.a).ceil()
    };
    let unbounded_right = if right.p.x > size_x * right.a {
        size_x
    } else {
        (right.p.x / right.a).ceil()
    };
    if unbounded_left >= unbounded_right {
        return (0, 0);
    }

    let uleft: usize = NumCast::from(unbounded_left).unwrap();
    let uright: usize = NumCast::from(unbounded_right).unwrap();
    let uy1: usize = NumCast::from(y1).unwrap();
    let uy2: usize = NumCast::from(y2).unwrap();
    let ul = get_y(left);
    let ur = get_y(right);

    if uy1 < ul {
        let fy = T::from(uy1).unwrap();
        let fx = minimal_neg_value(
            unbounded_left,
            unbounded_right,
            left_top.a,
            left_top.n.x,
            left_top.a * fy * fy + left_top.n.y * fy + left_top.c,
        );
        let mut x: usize = NumCast::from(fx).unwrap();
        let mut qf = QuadFunc::clocc(left_top, fx, fy);
        span_buffer[uy1].0 = NumCast::from(x).unwrap();
        qf.step_x_left();
        for y in uy1 + 1..ul {
            qf.step_y();
            while x > uleft && qf.get_value() <= T::zero() {
                x -= 1;
                qf.step_x_left();
            }
            span_buffer[y].0 = NumCast::from(x).unwrap();
        }
    }

    if ul < uy2 {
        let fy = T::from(ul).unwrap();
        let fx = minimal_neg_value(
            unbounded_left,
            unbounded_right,
            left_bottom.a,
            left_bottom.n.x,
            left_bottom.a * fy * fy + left_bottom.n.y * fy + left_bottom.c,
        );
        let mut x: usize = NumCast::from(fx).unwrap();
        let mut qf = QuadFunc::clocc(left_bottom, fx, fy);
        span_buffer[ul].0 = NumCast::from(x).unwrap();
        for y in ul + 1..uy2 {
            qf.step_y();
            while x < uright && qf.get_value() > T::zero() {
                x += 1;
                qf.step_x_right();
            }
            span_buffer[y].0 = NumCast::from(x).unwrap();
        }
    }

    if uy1 < ur {
        let fy = T::from(uy1).unwrap();
        let fx = minimal_neg_value(
            unbounded_left,
            unbounded_right,
            -right_top.a,
            -right_top.n.x,
            -(right_top.a * fy * fy + right_top.n.y * fy + right_top.c),
        );
        let mut x: usize = NumCast::from(fx).unwrap();
        let mut qf = QuadFunc::clocc(right_top, fx, fy);
        span_buffer[uy1].1 = NumCast::from(x).unwrap();
        for y in uy1 + 1..ur {
            qf.step_y();
            while x < uright && qf.get_value() <= T::zero() {
                x += 1;
                qf.step_x_right();
            }
            span_buffer[y].1 = NumCast::from(x).unwrap();
        }
    }

    if ur < uy2 {
        let fy = T::from(ur).unwrap();
        let fx = minimal_neg_value(
            unbounded_left,
            unbounded_right,
            -right_bottom.a,
            -right_bottom.n.x,
            -(right_bottom.a * fy * fy + right_bottom.n.y * fy + right_bottom.c),
        );
        let mut x: usize = NumCast::from(fx).unwrap();
        let mut qf = QuadFunc::clocc(right_bottom, fx, fy);
        span_buffer[ur].1 = NumCast::from(x).unwrap();
        qf.step_x_left();
        for y in ur + 1..uy2 {
            qf.step_y();
            while x > uleft && qf.get_value() > T::zero() {
                x -= 1;
                qf.step_x_left();
            }
            span_buffer[y].1 = NumCast::from(x).unwrap();
        }
    }

    (uy1, uy2)
}

fn draw_quadrant<T: Float>(
    size: ImageSize,
    border_funcs: [&CLoCC<T>; 4],
    corners: [&ProjectivePoint<T>; 4],
    q: usize,
    span_buffer: &mut [(usize, usize)],
) -> (usize, usize) {
    draw_rhombe(
        size,
        corners[(5 - q) % 4],
        corners[(4 - q) % 4],
        corners[(6 - q) % 4],
        corners[(3 - q) % 4],
        border_funcs[(4 - q) % 4],
        border_funcs[(3 - q) % 4],
        border_funcs[(5 - q) % 4],
        border_funcs[(6 - q) % 4],
        span_buffer,
    )
}

fn use_quadrant_bounds(
    dst: &mut ImageViewMut<u32>,
    span_buffer: &[(usize, usize)],
    ybounds: (usize, usize),
    color: u32,
) {
    for y in ybounds.0..ybounds.1 {
        let line = &mut dst[y];
        for x in span_buffer[y].0..span_buffer[y].1 {
            line[x] = color;
        }
    }
}

fn use_quadrant_bounds_aa2(
    dst: &mut ImageViewMut<u32>,
    span_buffer: &[(usize, usize)],
    ybounds: (usize, usize),
    color: u32,
) {
    let color = color & 0xFCFCFC;
    for y in ybounds.0..ybounds.1 {
        let line = &mut dst[y / 2];
        for x in span_buffer[y].0..span_buffer[y].1 {
            let dst = &mut line[x / 2];
            let d = *dst & 0xFCFCFC;
            *dst = u32::wrapping_add(d, u32::wrapping_sub(color, d) >> 2);
        }
    }
}

fn use_quadrant_bounds_aa4(
    dst: &mut ImageViewMut<u32>,
    span_buffer: &[(usize, usize)],
    ybounds: (usize, usize),
    color: u32,
) {
    let cr = color & 0x000000ff;
    let cg = color & 0x0000ff00;
    let cb = color & 0x00ff0000;
    let ca = color & 0xff000000;
    for y in ybounds.0..ybounds.1 {
        let line = &mut dst[y / 4];
        for x in span_buffer[y].0..span_buffer[y].1 {
            let dst = &mut line[x / 4];
            let dr = *dst & 0x000000ff;
            let dg = *dst & 0x0000ff00;
            let db = *dst & 0x00ff0000;
            let da = *dst & 0xff000000;
            let rr = u32::wrapping_add(dr, u32::wrapping_sub(cr, dr) >> 4) & 0x000000ff;
            let rg = u32::wrapping_add(dg, u32::wrapping_sub(cg, dg) >> 4) & 0x0000ff00;
            let rb = u32::wrapping_add(db, u32::wrapping_sub(cb, db) >> 4) & 0x00ff0000;
            let ra = u32::wrapping_add(da, u32::wrapping_sub(ca, da) >> 4) & 0xff000000;
            *dst = rr | rg | rb | ra;
        }
    }
}

pub fn draw_locc<T: Float>(
    dst: &mut ImageViewMut<u32>,
    locc: &LoCC<T>,
    color: u32,
    width: T,
    span_buffer: &mut [(usize, usize)],
    anti_aliasing: usize,
) {
    assert!(anti_aliasing == 1 || anti_aliasing == 2 || anti_aliasing == 4);
    let entity = locc.scale(T::from(anti_aliasing).unwrap());
    let curve = entity.get_clocc();
    let width = width * T::from(anti_aliasing).unwrap();
    let half_width = width / (T::one() + T::one());

    let out_curve = curve.change_radius(half_width).unwrap();
    let maybe_in_curve = curve.change_radius(-half_width);
    let in_curve = maybe_in_curve.unwrap_or(CLoCC::zero()).neg();

    let vert_curve = CLoCC::<T> {
        a: T::zero(),
        n: Point::new(curve.a + curve.a, T::zero()),
        c: curve.n.x,
    };

    let horz_curve = CLoCC::<T> {
        a: T::zero(),
        n: Point::new(T::zero(), curve.a + curve.a),
        c: curve.n.y,
    };

    let center = ProjectivePoint::new(-curve.n, curve.a + curve.a);

    macro_rules! in_or_center {
        ($f: expr) => {
            if maybe_in_curve.is_some() {
                $f
            } else {
                center
            }
        };
    }

    let in_d = T::one();
    let out_d = T::one();

    let default_border_curves = [vert_curve.neg(), horz_curve.neg(), vert_curve, horz_curve];
    let default_in_points = [
        in_or_center!(ProjectivePoint::new(
            Point::new(in_curve.n.x + in_d, in_curve.n.y),
            -in_curve.a - in_curve.a
        )),
        in_or_center!(ProjectivePoint::new(
            Point::new(in_curve.n.x, in_curve.n.y + in_d),
            -in_curve.a - in_curve.a
        )),
        in_or_center!(ProjectivePoint::new(
            Point::new(in_curve.n.x - in_d, in_curve.n.y),
            -in_curve.a - in_curve.a
        )),
        in_or_center!(ProjectivePoint::new(
            Point::new(in_curve.n.x, in_curve.n.y - in_d),
            -in_curve.a - in_curve.a
        )),
    ];

    let default_out_points = [
        ProjectivePoint::new(
            Point::new(-out_curve.n.x + out_d, -out_curve.n.y),
            out_curve.a + out_curve.a,
        ),
        ProjectivePoint::new(
            Point::new(-out_curve.n.x, -out_curve.n.y + out_d),
            out_curve.a + out_curve.a,
        ),
        ProjectivePoint::new(
            Point::new(-out_curve.n.x - out_d, -out_curve.n.y),
            out_curve.a + out_curve.a,
        ),
        ProjectivePoint::new(
            Point::new(-out_curve.n.x, -out_curve.n.y - out_d),
            out_curve.a + out_curve.a,
        ),
    ];

    macro_rules! use_quadrant {
        ($ys: ident) => {{
            let ys = $ys;
            match anti_aliasing {
                2 => use_quadrant_bounds_aa2(dst, span_buffer, ys, color),
                4 => use_quadrant_bounds_aa4(dst, span_buffer, ys, color),
                _ => use_quadrant_bounds(dst, span_buffer, ys, color),
            }
        }};
    }

    let view_bounds = (
        dst.get_size().0 * anti_aliasing,
        dst.get_size().1 * anti_aliasing,
    );
    match entity {
        LoCC::CLoCC(_) => {
            for q in 0..4 {
                let ys = draw_quadrant(
                    view_bounds,
                    [
                        &in_curve,
                        &default_border_curves[(q + 1) % 4],
                        &out_curve,
                        &default_border_curves[q],
                    ],
                    [
                        &default_in_points[(q + 1) % 4],
                        &default_in_points[q],
                        &default_out_points[q],
                        &default_out_points[(q + 1) % 4],
                    ],
                    q,
                    span_buffer,
                );
                use_quadrant!(ys);
            }
        }
        LoCC::SoCC(s) => {
            let begin_direction = s.begin_direction();
            let end_direction = s.end_direction();

            let begin_curve = CLoCC {
                a: T::zero(),
                n: -begin_direction.rot90(),
                c: dot(s.begin, begin_direction.rot90()),
            };

            let end_curve = CLoCC {
                a: T::zero(),
                n: end_direction.rot90(),
                c: -dot(s.end, end_direction.rot90()),
            };

            let begin_in_point = in_or_center!(ProjectivePoint::new(
                s.begin - begin_direction.normalize().scale(half_width),
                T::one()
            ));
            let end_in_point = in_or_center!(ProjectivePoint::new(
                s.end - end_direction.normalize().scale(half_width),
                T::one()
            ));
            let begin_out_point = ProjectivePoint::new(
                s.begin + begin_direction.normalize().scale(half_width),
                T::one(),
            );
            let end_out_point = ProjectivePoint::new(
                s.end + end_direction.normalize().scale(half_width),
                T::one(),
            );

            let get_quadrant = |p: Point<T>| -> usize {
                if p.x >= T::zero() && p.y >= T::zero() {
                    0
                } else if p.x < T::zero() && p.y >= T::zero() {
                    1
                } else if p.x < T::zero() && p.y < T::zero() {
                    2
                } else {
                    3
                }
            };

            let begin_quadrant = get_quadrant(begin_direction);
            let end_quadrant = get_quadrant(end_direction);
            if begin_quadrant == end_quadrant && !s.big {
                let q = begin_quadrant;
                let ys = draw_quadrant(
                    view_bounds,
                    [&in_curve, &begin_curve, &out_curve, &end_curve],
                    [
                        &end_in_point,
                        &begin_in_point,
                        &begin_out_point,
                        &end_out_point,
                    ],
                    q,
                    span_buffer,
                );
                use_quadrant!(ys);
            } else {
                let mut q = begin_quadrant;
                let ys = draw_quadrant(
                    view_bounds,
                    [
                        &in_curve,
                        &begin_curve,
                        &out_curve,
                        &default_border_curves[q],
                    ],
                    [
                        &default_in_points[(q + 1) % 4],
                        &begin_in_point,
                        &begin_out_point,
                        &default_out_points[(q + 1) % 4],
                    ],
                    q,
                    span_buffer,
                );
                use_quadrant!(ys);
                q = (q + 1) % 4;

                while q != end_quadrant {
                    let ys = draw_quadrant(
                        view_bounds,
                        [
                            &in_curve,
                            &default_border_curves[(q + 1) % 4],
                            &out_curve,
                            &default_border_curves[q],
                        ],
                        [
                            &default_in_points[(q + 1) % 4],
                            &default_in_points[q],
                            &default_out_points[q],
                            &default_out_points[(q + 1) % 4],
                        ],
                        q,
                        span_buffer,
                    );
                    use_quadrant!(ys);
                    q = (q + 1) % 4;
                }

                let ys = draw_quadrant(
                    view_bounds,
                    [
                        &in_curve,
                        &default_border_curves[(q + 1) % 4],
                        &out_curve,
                        &end_curve,
                    ],
                    [
                        &end_in_point,
                        &default_in_points[q],
                        &default_out_points[q],
                        &end_out_point,
                    ],
                    q,
                    span_buffer,
                );
                use_quadrant!(ys);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, SeedableRng};

    #[test]
    fn fuzzy_test_line() {
        let mut dst = Image::<u32>::new((4096, 4096));
        let mut span_buffer = vec![(0, 0); dst.get_size().1 * 4];

        for i in 0..2_000 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(i);
            let x1 = rng.gen_range(-10_000.0..10_000.0);
            let y1 = rng.gen_range(-10_000.0..10_000.0);
            let x2 = rng.gen_range(-10_000.0..10_000.0);
            let y2 = rng.gen_range(-10_000.0..10_000.0);
            let pt1 = Point::new(x1, y1);
            let pt2 = Point::new(x2, y2);

            let c = CLoCC::<f32>::line(pt1, pt2);
            let e = &LoCC::SoCC(SoCC {
                clocc: c,
                begin: pt1,
                end: pt2,
                big: false,
            });

            draw_locc(
                &mut dst.as_view_mut(),
                &e,
                0,
                rng.gen_range(0.001..10.0),
                &mut span_buffer,
                4,
            );
        }
    }

    #[test]
    fn fuzzy_test_circle() {
        let mut dst = Image::<u32>::new((4096, 4096));
        let mut span_buffer = vec![(0, 0); dst.get_size().1 * 4];

        for i in 0..2_000 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(i);
            let radius = rng.gen_range(0.00_01..10_000.0);
            let x = rng.gen_range(-100_00.0..100_00.0);
            let y = rng.gen_range(-100_00.0..100_00.0);
            let center = Point::new(x, y);

            use std::f32::consts::PI;
            let angle1 = rng.gen_range(0.0..PI * 2.0);
            let angle2 = rng.gen_range(0.0..PI * 2.0);

            let c = CLoCC::<f32>::circle(center, radius);
            let e = &LoCC::SoCC(SoCC {
                clocc: c,
                begin: center + Point::angle(angle1).scale(radius),
                end: center + Point::angle(angle1 + angle2).scale(radius),
                big: angle2 > PI,
            });

            draw_locc(
                &mut dst.as_view_mut(),
                &e,
                0,
                rng.gen_range(0.001..10.0),
                &mut span_buffer,
                4,
            );
        }
    }

    #[test]
    fn correct_test_line() {
        let mut dst = Image::<u32>::new((64, 64));
        let mut span_buffer = vec![(0, 0); dst.get_size().1];

        for i in 0..10_000 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(i);
            dst.as_view_mut().fill(|p| *p = 1);
            let x1 = rng.gen_range(-100.0..100.0);
            let y1 = rng.gen_range(-100.0..100.0);
            let x2 = rng.gen_range(-100.0..100.0);
            let y2 = rng.gen_range(-100.0..100.0);

            let pt1 = Point::new(x1, y1);
            let pt2 = Point::new(x2, y2);

            let c = CLoCC::<f32>::line(pt1, pt2);
            let s = SoCC {
                clocc: c,
                begin: pt1,
                end: pt2,
                big: false,
            };
            let e = LoCC::SoCC(s);

            let width = rng.gen_range(0.001..40.0);
            let half_width = width * 0.5;
            draw_locc(&mut dst.as_view_mut(), &e, 0, width, &mut span_buffer, 1);
            let out_curve = c.change_radius(half_width).unwrap();
            let in_curve = c.change_radius(-half_width).unwrap_or(CLoCC::zero()).neg();

            let begin_direction = s.begin_direction();
            let end_direction = s.end_direction();

            let begin_curve = CLoCC {
                a: 0.0,
                n: -begin_direction.rot90(),
                c: dot(s.begin, begin_direction.rot90()),
            };

            let end_curve = CLoCC {
                a: 0.0,
                n: end_direction.rot90(),
                c: -dot(s.end, end_direction.rot90()),
            };

            for y in 0..dst.get_size().1 {
                let line = &dst.as_view()[y];
                for x in 0..line.len() {
                    let p = Point::new(x as f32, y as f32);
                    let control = f32::max(
                        f32::max(out_curve.get_value(p), in_curve.get_value(p)),
                        f32::max(begin_curve.get_value(p), end_curve.get_value(p)),
                    );
                    assert!(
                        (line[x] > 0 && control > -0.1) || (line[x] <= 0 && control < 0.1),
                        "mismatch on seed {} curve {:?} and width {:?} at [{}:{}]",
                        i,
                        s,
                        width,
                        x,
                        y
                    );
                }
            }
        }
    }

    #[test]
    fn correct_test_circle() {
        let mut dst = Image::<u32>::new((64, 64));
        let mut span_buffer = vec![(0, 0); dst.get_size().1];

        for i in 0..10_000 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(i);
            dst.as_view_mut().fill(|p| *p = 1);
            let radius = rng.gen_range(2.0..100.0);
            let x = rng.gen_range(-100.0..100.0);
            let y = rng.gen_range(-100.0..100.0);
            let center = Point::new(x, y);

            use std::f32::consts::PI;
            let angle1 = rng.gen_range(0.0..PI * 2.0);
            let angle2 = rng.gen_range(0.0..PI * 2.0);

            let c = CLoCC::<f32>::circle(center, radius);
            let s = SoCC {
                clocc: c,
                begin: center + Point::angle(angle1).scale(radius),
                end: center + Point::angle(angle1 + angle2).scale(radius),
                big: angle2 > PI,
            };
            let e = LoCC::SoCC(s);

            let width = rng.gen_range(0.001..40.0);
            let half_width = width * 0.5;
            draw_locc(&mut dst.as_view_mut(), &e, 0, width, &mut span_buffer, 1);
            let out_curve = c.change_radius(half_width).unwrap();
            let in_curve = c.change_radius(-half_width).unwrap_or(CLoCC::zero()).neg();

            let begin_direction = s.begin_direction();
            let end_direction = s.end_direction();

            let begin_curve = CLoCC {
                a: 0.0,
                n: -begin_direction.rot90(),
                c: dot(s.begin, begin_direction.rot90()),
            };

            let end_curve = CLoCC {
                a: 0.0,
                n: end_direction.rot90(),
                c: -dot(s.end, end_direction.rot90()),
            };

            if angle2 > PI {
                for y in 0..dst.get_size().1 {
                    let line = &dst.as_view()[y];
                    for x in 0..line.len() {
                        let p = Point::new(x as f32, y as f32);
                        let control = f32::max(
                            f32::max(out_curve.get_value(p), in_curve.get_value(p)),
                            f32::min(begin_curve.get_value(p), end_curve.get_value(p)),
                        );
                        assert!(
                            (line[x] > 0 && control > -0.1) || (line[x] <= 0 && control < 0.1),
                            "mismatch on seed {} curve {:?} with center {:?} and width {:?} at [{}:{}]", i, s, center, width, x, y
                        );
                    }
                }
            } else {
                for y in 0..dst.get_size().1 {
                    let line = &dst.as_view()[y];
                    for x in 0..line.len() {
                        let p = Point::new(x as f32, y as f32);
                        let control = f32::max(
                            f32::max(out_curve.get_value(p), in_curve.get_value(p)),
                            f32::max(begin_curve.get_value(p), end_curve.get_value(p)),
                        );
                        assert!(
                            (line[x] > 0 && control > -0.1) || (line[x] <= 0 && control < 0.1),
                            "mismatch on seed {} curve {:?} with center {:?} and width {:?} at [{}:{}]", i, s, center, width, x, y
                        );
                    }
                }
            }
        }
    }
}
