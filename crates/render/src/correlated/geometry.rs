//! Own f64 fill geometry. The point rule is mathematical, not native curve parity.

type Point = [f64; 2];
pub(super) type Matrix = [f64; 6];
const LIMIT: f64 = 1e9;
const MAX_PIECES: usize = 16_384;
pub(super) type Solves = std::collections::HashMap<([u64; 4], u64, u64, u64), f64>;

fn valid(v: f64) -> bool {
    v.is_finite() && v.abs() <= LIMIT
}

fn point(p: Point, m: Matrix) -> Option<Point> {
    let p = [m[0] * p[0] + m[2] * p[1] + m[4], m[1] * p[0] + m[3] * p[1] + m[5]];
    p.into_iter().all(valid).then_some(p)
}

#[derive(Clone, Copy)]
struct Curve([Point; 4]);
impl Curve {
    fn at(self, t: f64) -> Point {
        let u = 1. - t;
        std::array::from_fn(|c| u * u * u * self.0[0][c] + 3. * u * u * t * self.0[1][c] + 3. * u * t * t * self.0[2][c] + t * t * t * self.0[3][c])
    }

    fn line(self) -> bool {
        self.0[0] == self.0[1] && self.0[2] == self.0[3]
    }

    fn roots(self) -> Vec<f64> {
        let y = self.0.map(|p| p[1]);
        let a = 3. * (-y[0] + 3. * y[1] - 3. * y[2] + y[3]);
        let b = 2. * (3. * y[0] - 6. * y[1] + 3. * y[2]);
        let c = 3. * (y[1] - y[0]);
        let mut roots = vec![0., 1.];
        if a == 0. {
            if b != 0. {
                roots.push(-c / b);
            }
        } else {
            let d = b * b - 4. * a * c;
            if d >= 0. {
                let q = -0.5 * (b + d.sqrt().copysign(b));
                if q == 0. {
                    roots.push(-b / (2. * a));
                } else {
                    roots.push(q / a);
                    roots.push(c / q);
                }
            }
        }
        roots.retain(|t| t.is_finite() && *t >= 0. && *t <= 1.);
        roots.sort_by(f64::total_cmp);
        roots.dedup();
        roots
    }
}

struct Piece {
    curve: Curve,
    t0: f64,
    t1: f64,
    y0: f64,
    y1: f64,
}

impl Piece {
    fn crossing(&self, y: f64, solves: &mut Solves) -> Option<(f64, i32)> {
        if !(y >= self.y0.min(self.y1) && y < self.y0.max(self.y1)) {
            return None;
        }
        let increasing = self.y1 > self.y0;
        let curve = self.curve;
        let x = if curve.line() {
            let p = curve.0[0];
            let q = curve.0[3];
            p[0] + (q[0] - p[0]) * (y - p[1]) / (q[1] - p[1])
        } else {
            // Identical Y predicates share t; evaluate each curve's original world X separately.
            let key = (curve.0.map(|p| p[1].to_bits()), self.t0.to_bits(), self.t1.to_bits(), y.to_bits());
            let t = *solves.entry(key).or_insert_with(|| {
                let (mut lo, mut hi) = (self.t0, self.t1);
                for _ in 0..64 {
                    let t = (lo + hi) * 0.5;
                    if t == lo || t == hi {
                        break;
                    }
                    if (curve.at(t)[1] < y) == increasing {
                        lo = t;
                    } else {
                        hi = t;
                    }
                }
                (lo + hi) * 0.5
            });
            curve.at(t)[0]
        };
        Some((x, if increasing { 1 } else { -1 }))
    }
}

pub(super) struct Geometry {
    parts: Vec<Piece>,
    bounds: [f64; 4],
    evenodd: bool,
}

pub(super) struct Row {
    crossings: Vec<f64>,
    suffix: Vec<i32>,
    evenodd: bool,
}
impl Row {
    pub(super) fn contains(&self, x: f64) -> bool {
        if !x.is_finite() {
            return false;
        }
        let i = self.crossings.partition_point(|&crossing| crossing <= x);
        let w = self.suffix[i];
        if self.evenodd { w & 1 != 0 } else { w != 0 }
    }

    /// Strict crossing>x coverage is constant when no crossing is in (left,right].
    pub(super) fn constant_between(&self, left: f64, right: f64) -> Option<bool> {
        if !left.is_finite() || !right.is_finite() || right < left {
            return None;
        }
        let begin = self.crossings.partition_point(|&x| x <= left);
        let end = self.crossings.partition_point(|&x| x <= right);
        if begin != end {
            return None;
        }
        let winding = self.suffix[begin];
        Some(if self.evenodd { winding & 1 != 0 } else { winding != 0 })
    }
}

impl Geometry {
    fn empty(evenodd: bool) -> Self {
        Self { parts: vec![], bounds: [0.; 4], evenodd }
    }

    fn from_curves(curves: Vec<Curve>, matrix: Matrix, evenodd: bool) -> Option<Self> {
        if !matrix.into_iter().all(valid) {
            return None;
        }
        if matrix[0] * matrix[3] - matrix[1] * matrix[2] == 0. {
            return Some(Self::empty(evenodd));
        }
        let mut result = Self { parts: vec![], bounds: [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY], evenodd };
        for curve in curves {
            let mut p = [[0.; 2]; 4];
            for (dest, original) in p.iter_mut().zip(curve.0) {
                if !original.into_iter().all(valid) {
                    return None;
                }
                *dest = point(original, matrix)?;
                result.bounds[0] = result.bounds[0].min(dest[0]);
                result.bounds[1] = result.bounds[1].min(dest[1]);
                result.bounds[2] = result.bounds[2].max(dest[0]);
                result.bounds[3] = result.bounds[3].max(dest[1]);
            }
            let curve = Curve(p);
            let roots = if curve.line() { vec![0., 1.] } else { curve.roots() };
            for pair in roots.windows(2) {
                let y0 = curve.at(pair[0])[1];
                let y1 = curve.at(pair[1])[1];
                if y0 != y1 {
                    if result.parts.len() == MAX_PIECES {
                        return None;
                    }
                    result.parts.push(Piece { curve, t0: pair[0], t1: pair[1], y0, y1 });
                }
            }
        }
        if !result.bounds[0].is_finite() {
            result.bounds = [0.; 4];
        }
        Some(result)
    }

    pub(super) fn path(path: &figcraft_geom::BezPath, matrix: Matrix, evenodd: bool) -> Option<Self> {
        use figcraft_geom::kurbo::PathEl;
        let mut curves = Vec::new();
        let (mut at, mut first) = ([0.; 2], None);
        for el in path.elements() {
            if curves.len() >= MAX_PIECES {
                return None;
            }
            match *el {
                PathEl::MoveTo(p) => {
                    if let Some(start) = first
                        && at != start
                    {
                        curves.push(Curve([at, at, start, start]));
                    }
                    at = [p.x, p.y];
                    first = Some(at);
                }
                PathEl::LineTo(p) => {
                    let end = [p.x, p.y];
                    curves.push(Curve([at, at, end, end]));
                    at = end;
                }
                PathEl::QuadTo(c, p) => {
                    let end = [p.x, p.y];
                    let c = [c.x, c.y];
                    curves.push(Curve([
                        at,
                        std::array::from_fn(|i| at[i] + (c[i] - at[i]) * 2. / 3.),
                        std::array::from_fn(|i| end[i] + (c[i] - end[i]) * 2. / 3.),
                        end,
                    ]));
                    at = end;
                }
                PathEl::CurveTo(a, b, p) => {
                    let end = [p.x, p.y];
                    curves.push(Curve([at, [a.x, a.y], [b.x, b.y], end]));
                    at = end;
                }
                PathEl::ClosePath => {
                    if let Some(start) = first {
                        curves.push(Curve([at, at, start, start]));
                        at = start;
                    }
                    first = None;
                }
            }
        }
        if let Some(start) = first
            && at != start
        {
            curves.push(Curve([at, at, start, start]));
        }
        Self::from_curves(curves, matrix, evenodd)
    }

    pub(super) fn bounds(&self) -> [f64; 4] {
        self.bounds
    }

    pub(super) fn pieces(&self) -> usize {
        self.parts.len()
    }

    pub(super) fn row(&self, y: f64, solves: &mut Solves) -> Row {
        let mut points: Vec<_> = if y.is_finite() { self.parts.iter().filter_map(|p| p.crossing(y, solves)).collect() } else { vec![] };
        points.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut suffix = vec![0; points.len() + 1];
        for i in (0..points.len()).rev() {
            suffix[i] = suffix[i + 1] + points[i].1;
        }
        Row { crossings: points.into_iter().map(|p| p.0).collect(), suffix, evenodd: self.evenodd }
    }
}
