use super::{Blend, Content, Plan, geometry::Row};
use std::{collections::BTreeSet, mem::size_of};

type Pixel = [f64; 4];
const ZERO: Pixel = [0.; 4];
const TILE: usize = 16;
const SURFACES: f64 = 16_777_216.;
const SAMPLE_WORK: f64 = 256_000_000.;
const ROW_WORK: f64 = 128_000_000.;
// Preserve the original row/column/four-point accumulation order.
const POINTS: [(f64, usize); 16] = [
    (3. / 16., 0),
    (7. / 16., 1),
    (1. / 16., 2),
    (5. / 16., 3),
    (11. / 16., 0),
    (15. / 16., 1),
    (9. / 16., 2),
    (13. / 16., 3),
    (3. / 16., 4),
    (7. / 16., 5),
    (1. / 16., 6),
    (5. / 16., 7),
    (11. / 16., 4),
    (15. / 16., 5),
    (9. / 16., 6),
    (13. / 16., 7),
];

#[derive(Clone, Copy, Default)]
struct Span([usize; 4]);
impl Span {
    fn empty(self) -> bool {
        self.0[0] >= self.0[2] || self.0[1] >= self.0[3]
    }
    fn intersect(self, other: Self) -> Self {
        Self([self.0[0].max(other.0[0]), self.0[1].max(other.0[1]), self.0[2].min(other.0[2]), self.0[3].min(other.0[3])])
    }
    fn tiles(self) -> std::ops::Range<usize> {
        self.0[0] / TILE..self.0[2].div_ceil(TILE)
    }
}

#[derive(Clone, Copy)]
struct Link {
    parent: usize,
    rank: Option<usize>,
}

struct Prepared {
    spans: Vec<Span>,
    links: Vec<Option<Link>>,
    starts: Vec<(usize, usize)>,
    ends: Vec<(usize, usize)>,
    tile_capacity: usize,
}

fn preflight(plan: &Plan) -> Option<Prepared> {
    let n = plan.entries.len();
    let (width, height) = (plan.width as usize, plan.height as usize);
    let area = (width as f64) * height as f64;
    if n == 0 || n > 32_768 || plan.root >= n || plan.geometries.len() > 16_384 || width == 0 || height == 0 || area * 16. > SAMPLE_WORK {
        return None;
    }
    let pixel_bytes = width.checked_mul(height)?.checked_mul(8)?;
    let pieces = plan.geometries.iter().try_fold(0usize, |sum, geometry| sum.checked_add(geometry.pieces()))?;
    // Charge the existing plan as well as both preflight and rendering storage.
    // The per-entry/geometry allowances include vector capacity, tree nodes and
    // temporary metadata; piece storage includes all eight cached crossing rows.
    // This is a conservative allocation allowance, not a process-RSS estimate.
    // Check it before allocating even preflight metadata when reserved is high.
    let fixed_bytes = pixel_bytes as f64
        + width as f64 * 16.
        + 1_048_576.
        + n as f64 * (2 * size_of::<super::Entry>() + 1_024) as f64
        + plan.geometries.len() as f64 * (2 * size_of::<super::geometry::Geometry>() + size_of::<Option<[Row; 8]>>() + 256) as f64
        // Includes eight cached crossing rows and the worst bucket growth of shared Y solves.
        + pieces as f64 * 2_048.
        + width.div_ceil(TILE) as f64 * 24.;
    if fixed_bytes / 4. > SURFACES {
        return None;
    }
    let mut links = vec![None; n];
    let mut spans = Vec::with_capacity(n);
    for (id, entry) in plan.entries.iter().enumerate() {
        let b = entry.bounds;
        if b.iter().any(|v| !v.is_finite()) || b[0] > b[2] || b[1] > b[3] {
            return None;
        }
        let mut span = Span([
            b[0].floor().clamp(0., width as f64) as usize,
            b[1].floor().clamp(0., height as f64) as usize,
            b[2].ceil().clamp(0., width as f64) as usize,
            b[3].ceil().clamp(0., height as f64) as usize,
        ]);
        match &entry.content {
            Content::Fill { geometry, color } => {
                if *geometry >= plan.geometries.len()
                    || color.iter().any(|v| !v.is_finite())
                    || !(0. ..=1.).contains(&color[3])
                    || color[..3].iter().any(|v| *v < 0. || *v > color[3])
                {
                    return None;
                }
                if color[3] == 0. {
                    span = Span::default();
                }
            }
            Content::Group { children, opacity, pass, blend, clip, mask } => {
                if !opacity.is_finite() || !(0. ..=1.).contains(opacity) || (*pass && !matches!(blend, Blend::Normal)) {
                    return None;
                }
                for (child, rank) in
                    children.iter().copied().enumerate().map(|(r, c)| (c, Some(r))).chain(clip.iter().chain(mask.iter()).copied().map(|c| (c, None)))
                {
                    if child >= id || links[child].is_some() {
                        return None;
                    }
                    links[child] = Some(Link { parent: id, rank });
                }
                if *opacity == 0. {
                    span = Span::default();
                }
            }
        }
        spans.push(span);
    }
    if links[plan.root].is_some() || links.iter().enumerate().any(|(i, p)| i != plan.root && p.is_none()) {
        return None;
    }
    let mut depth = vec![0u8; n];
    for id in (0..n).rev() {
        if let Some(link) = links[id] {
            depth[id] = depth[link.parent].checked_add(1)?;
            if depth[id] > 64 {
                return None;
            }
            spans[id] = spans[id].intersect(spans[link.parent]);
        }
    }
    let mut geometry_spans = vec![Span::default(); plan.geometries.len()];
    let mut work = area * 16.;
    let mut starts = Vec::with_capacity(n);
    let mut ends = Vec::with_capacity(n);
    for (id, span) in spans.iter().copied().enumerate().filter(|(_, s)| !s.empty()) {
        let rows = span.0[3] - span.0[1];
        work += span.tiles().len() as f64 * rows as f64 * (TILE * 16 + 4) as f64;
        if work > SAMPLE_WORK {
            return None;
        }
        starts.push((span.0[1], id));
        ends.push((span.0[3], id));
        if let Content::Fill { geometry, .. } = plan.entries[id].content {
            let old = &mut geometry_spans[geometry];
            *old = if old.empty() {
                span
            } else {
                Span([old.0[0].min(span.0[0]), old.0[1].min(span.0[1]), old.0[2].max(span.0[2]), old.0[3].max(span.0[3])])
            };
        }
    }
    let mut row_work = 0.;
    for (geometry, span) in plan.geometries.iter().zip(geometry_spans) {
        let count = geometry.pieces() as f64;
        if !span.empty() {
            // Includes monotone solve iterations and sorting.
            row_work += (span.0[3] - span.0[1]) as f64 * 8. * count * (68. + count.max(1.).log2().ceil());
            if row_work > ROW_WORK {
                return None;
            }
        }
    }
    starts.sort_unstable();
    ends.sort_unstable();
    let (mut first, mut last, mut active, mut peak) = (0, 0, 0usize, 0usize);
    while first < starts.len() || last < ends.len() {
        if last < ends.len() && (first == starts.len() || ends[last].0 <= starts[first].0) {
            active = active.checked_sub(spans[ends[last].1].tiles().len())?;
            last += 1;
        } else {
            active = active.checked_add(spans[starts[first].1].tiles().len())?;
            peak = peak.max(active);
            first += 1;
        }
    }
    let bytes = fixed_bytes + peak as f64 * size_of::<usize>() as f64;
    let used_surfaces = bytes / 4.;
    (used_surfaces <= SURFACES).then_some(Prepared { spans, links, starts, ends, tile_capacity: peak })
}

fn over(src: Pixel, dst: Pixel, blend: Blend) -> Pixel {
    let mut out = ZERO;
    out[3] = if matches!(blend, Blend::Plus) { (src[3] + dst[3]).min(1.) } else { src[3] + dst[3] * (1. - src[3]) };
    for c in 0..3 {
        out[c] = match blend {
            Blend::Normal => src[c] + dst[c] * (1. - src[3]),
            Blend::Screen => src[c] + dst[c] - src[c] * dst[c],
            Blend::Lighten => src[c] * (1. - dst[3]) + dst[c] * (1. - src[3]) + (src[c] * dst[3]).max(dst[c] * src[3]),
            Blend::Plus => (src[c] + dst[c]).min(1.),
        };
    }
    out
}

struct TileScene<'a> {
    plan: &'a Plan,
    active: &'a [bool],
    children: &'a [Vec<usize>],
    rows: &'a [Option<[Row; 8]>],
}
impl TileScene<'_> {
    fn constant_tile(&self, ids: &[usize], start: usize, end: usize, y: usize) -> bool {
        let left = start as f64 + 1. / 16.;
        let right = (end - 1) as f64 + 15. / 16.;
        let top = y as f64 + 1. / 16.;
        let bottom = y as f64 + 15. / 16.;
        for &id in ids {
            let entry = &self.plan.entries[id];
            let b = entry.bounds;
            // TileScene::sample also checks entry bounds, including group bounds.
            // Constant geometry alone cannot bypass a varying bounds guard.
            if b[0] > left || b[2] < right || b[1] > top || b[3] < bottom {
                return false;
            }
            if let Content::Fill { geometry, .. } = entry.content {
                let Some(rows) = self.rows.get(geometry).and_then(Option::as_ref) else { return false };
                let Some(covered) = rows[0].constant_between(left, right) else {
                    return false;
                };
                if rows[1..].iter().any(|row| row.constant_between(left, right) != Some(covered)) {
                    return false;
                }
            }
        }
        true
    }

    fn pixel(&self, x: usize, y: usize, backdrop: Pixel) -> [u8; 4] {
        let mut pixel = ZERO;
        for (sx, row) in POINTS {
            let sample = self.sample(self.plan.root, x as f64 + sx, y as f64 + (2 * row + 1) as f64 / 16., row, backdrop);
            for c in 0..4 {
                pixel[c] += sample[c] / 16.;
            }
        }
        resolve(pixel)
    }

    fn constant_pixel(&self, x: usize, y: usize, backdrop: Pixel) -> [u8; 4] {
        let sample = self.sample(self.plan.root, x as f64 + POINTS[0].0, y as f64 + 1. / 16., 0, backdrop);
        let mut pixel = ZERO;
        // Keep the same sixteen floating additions; geometry and materials are constant here.
        for _ in POINTS {
            for c in 0..4 {
                pixel[c] += sample[c] / 16.;
            }
        }
        resolve(pixel)
    }

    fn sample(&self, id: usize, x: f64, y: f64, row: usize, dst: Pixel) -> Pixel {
        if !self.active[id] {
            return dst;
        }
        let entry = &self.plan.entries[id];
        let b = entry.bounds;
        if x < b[0] || x > b[2] || y < b[1] || y > b[3] {
            return dst;
        }
        match &entry.content {
            Content::Fill { geometry, color } => {
                if self.rows.get(*geometry).and_then(Option::as_ref).is_some_and(|rows| rows[row].contains(x)) {
                    over(*color, dst, Blend::Normal)
                } else {
                    dst
                }
            }
            Content::Group { opacity, pass, blend, clip, mask, .. } => {
                if *opacity == 0. || clip.is_some_and(|c| self.sample(c, x, y, row, ZERO)[3] == 0.) {
                    return dst;
                }
                let alpha = *opacity * mask.map_or(1., |m| self.sample(m, x, y, row, ZERO)[3]);
                if alpha == 0. {
                    return dst;
                }
                let pass = *pass && mask.is_none();
                let mut body = if pass { dst } else { ZERO };
                for &child in &self.children[id] {
                    body = self.sample(child, x, y, row, body);
                }
                if pass {
                    if alpha == 1. { body } else { std::array::from_fn(|c| dst[c] + (body[c] - dst[c]) * alpha) }
                } else {
                    over(body.map(|v| v * alpha), dst, *blend)
                }
            }
        }
    }
}

fn resolve(pixel: Pixel) -> [u8; 4] {
    let alpha = (pixel[3].clamp(0., 1.) * 255.).round() as u8;
    std::array::from_fn(|c| if c == 3 { alpha } else { ((pixel[c].clamp(0., 1.) * 255.).round() as u8).min(alpha) })
}

pub(super) fn render(plan: &Plan, backdrop: &tiny_skia::Pixmap) -> Option<tiny_skia::Pixmap> {
    render_tiles::<true>(plan, backdrop)
}

fn render_tiles<const CONSTANT_TILES: bool>(plan: &Plan, backdrop: &tiny_skia::Pixmap) -> Option<tiny_skia::Pixmap> {
    let prepared = preflight(plan)?;
    let (width, height) = (plan.width as usize, plan.height as usize);
    let columns = width.div_ceil(TILE);
    let mut output = Vec::new();
    let len = width.checked_mul(height)?.checked_mul(4)?;
    output.try_reserve_exact(len).ok()?;
    let mut flat = Vec::new();
    flat.try_reserve_exact(prepared.tile_capacity).ok()?;
    let mut rows: Vec<Option<[Row; 8]>> = (0..plan.geometries.len()).map(|_| None).collect();
    let mut cached = Vec::new();
    let mut geometry_refs = vec![0usize; plan.geometries.len()];
    let mut active_geometry = BTreeSet::new();
    let mut active_entries = BTreeSet::new();
    let mut flags = vec![false; plan.entries.len()];
    let mut children: Vec<Vec<usize>> = plan
        .entries
        .iter()
        .map(|e| match &e.content {
            Content::Group { children, .. } => Vec::with_capacity(children.len()),
            _ => Vec::new(),
        })
        .collect();
    let mut counts = vec![0usize; columns];
    let mut offsets = vec![0usize; columns + 1];
    let mut bytes = vec![0u8; width * 4];
    let mut first = 0;
    let mut last = 0;
    let mut solves = super::geometry::Solves::new();
    for y in 0..height {
        solves.clear();
        for &g in &cached {
            rows[g] = None;
        }
        cached.clear();
        while last < prepared.ends.len() && prepared.ends[last].0 <= y {
            let id = prepared.ends[last].1;
            active_entries.remove(&id);
            if let Content::Fill { geometry, .. } = plan.entries[id].content {
                geometry_refs[geometry] -= 1;
                if geometry_refs[geometry] == 0 {
                    active_geometry.remove(&geometry);
                }
            }
            last += 1;
        }
        while first < prepared.starts.len() && prepared.starts[first].0 <= y {
            let id = prepared.starts[first].1;
            active_entries.insert(id);
            if let Content::Fill { geometry, .. } = plan.entries[id].content {
                geometry_refs[geometry] += 1;
                active_geometry.insert(geometry);
            }
            first += 1;
        }
        for &g in &active_geometry {
            rows[g] = Some(std::array::from_fn(|r| plan.geometries[g].row(y as f64 + (2 * r + 1) as f64 / 16., &mut solves)));
            cached.push(g);
        }
        counts.fill(0);
        for &id in &active_entries {
            for t in prepared.spans[id].tiles() {
                counts[t] += 1;
            }
        }
        offsets[0] = 0;
        for t in 0..columns {
            offsets[t + 1] = offsets[t] + counts[t];
            counts[t] = offsets[t];
        }
        flat.resize(offsets[columns], 0);
        for &id in &active_entries {
            for t in prepared.spans[id].tiles() {
                flat[counts[t]] = id;
                counts[t] += 1;
            }
        }
        let back = backdrop.data().get(y * width * 4..(y + 1) * width * 4)?;
        bytes.copy_from_slice(back);
        for tile in 0..columns {
            let ids = &flat[offsets[tile]..offsets[tile + 1]];
            if ids.is_empty() {
                continue;
            }
            for &id in ids {
                flags[id] = true;
                if let Some(Link { parent, rank: Some(_) }) = prepared.links[id] {
                    children[parent].push(id);
                }
            }
            for &id in ids {
                children[id].sort_unstable_by_key(|&child| prepared.links[child].and_then(|link| link.rank));
            }
            let scene = TileScene { plan, active: &flags, children: &children, rows: &rows };
            let start = tile * TILE;
            let end = ((tile + 1) * TILE).min(width);
            let at = |x: usize| std::array::from_fn(|c| f64::from(back[x * 4 + c]) / 255.);
            if CONSTANT_TILES
                && scene.constant_tile(ids, start, end, y)
                && back[start * 4..end * 4].as_chunks::<4>().0.iter().all(|p| *p == back[start * 4..start * 4 + 4])
            {
                // Keep all sixteen original floating operations even when
                // coverage is constant; a single sample changes rounding.
                let pixel = scene.constant_pixel(start, y, at(start));
                for target in bytes[start * 4..end * 4].chunks_exact_mut(4) {
                    target.copy_from_slice(&pixel);
                }
            } else {
                for x in start..end {
                    bytes[x * 4..x * 4 + 4].copy_from_slice(&scene.pixel(x, y, at(x)));
                }
            }
            for &id in ids {
                flags[id] = false;
                children[id].clear();
            }
        }
        output.extend_from_slice(&bytes);
    }
    tiny_skia::Pixmap::from_vec(output, tiny_skia::IntSize::from_wh(plan.width, plan.height)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use figcraft_color::{Color, Paint};
    use figcraft_doc::{Document, NodeKind};
    use figcraft_geom::Affine;

    #[test]
    fn constant_tiles_preserve_all_sixteen_resolve_operations() {
        let mut d = Document::new("constant tiles");
        let page = d.first_page().unwrap();
        let frame = d.create(page, NodeKind::Frame, "Frame", 0., 0., 128., 64.).unwrap();
        d.get_mut(frame).unwrap().fills.clear();
        for opacity in [0.215, 0.5, 0.973] {
            let id = d.create(frame, NodeKind::Rectangle, "Rect", 3.25, 4.125, 117., 54.).unwrap();
            let n = d.get_mut(id).unwrap();
            n.opacity = opacity;
            n.fills = vec![Paint::solid(Color::new(0.31, 0.71, 0.41, 0.63))];
        }
        let plan = super::super::compile::scene(&d, &[(frame, Affine::IDENTITY)], 128, 64).unwrap();
        let mut backdrop = tiny_skia::Pixmap::new(128, 64).unwrap();
        backdrop.fill(tiny_skia::Color::from_rgba8(31, 97, 123, 215));
        backdrop.data_mut()[4..8].copy_from_slice(&[3, 2, 1, 4]);
        let exact = render_tiles::<false>(&plan, &backdrop).unwrap();
        let optimized = render_tiles::<true>(&plan, &backdrop).unwrap();
        assert_eq!(exact.data(), optimized.data());
    }
}
