use super::{Blend, Content, Entry, Geometry, Plan};
use figcraft_color::{BlendMode, PaintKind};
use figcraft_doc::{Document, MaskType, NodeId, NodeKind};
use figcraft_geom::Affine;
use std::collections::HashSet;

pub(super) fn scene(doc: &Document, roots: &[(NodeId, Affine)], width: u32, height: u32) -> Option<Plan> {
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > 16_000_000 || roots.len() > 16_384 {
        return None;
    }
    let mut c =
        Compiler { doc, plan: Plan { entries: vec![], geometries: vec![], root: 0, width, height }, active: HashSet::new(), nodes: 0, pieces: 0 };
    let mut children = Vec::new();
    for &(id, parent) in roots {
        if let Some(node) = c.node(id, parent, 0, false)? {
            children.push(node);
        }
    }
    c.plan.root = c.group(children, 1., true, Blend::Normal, None, None)?;
    Some(c.plan)
}
struct Compiler<'a> {
    doc: &'a Document,
    plan: Plan,
    active: HashSet<NodeId>,
    nodes: usize,
    pieces: usize,
}
fn blend(b: BlendMode) -> Option<Blend> {
    Some(match b {
        BlendMode::Normal | BlendMode::PassThrough => Blend::Normal,
        BlendMode::Screen => Blend::Screen,
        BlendMode::Lighten => Blend::Lighten,
        BlendMode::PlusLighter => Blend::Plus,
        _ => return None,
    })
}
fn union(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [a[0].min(b[0]), a[1].min(b[1]), a[2].max(b[2]), a[3].max(b[3])]
}
fn intersect(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    let x = a[0].max(b[0]);
    let y = a[1].max(b[1]);
    [x, y, a[2].min(b[2]).max(x), a[3].min(b[3]).max(y)]
}
impl Compiler<'_> {
    fn entry(&mut self, bounds: [f64; 4], content: Content) -> Option<usize> {
        if self.plan.entries.len() >= 32_768 {
            return None;
        }
        let id = self.plan.entries.len();
        self.plan.entries.push(Entry { bounds, content });
        Some(id)
    }
    fn geometry(&mut self, path: &figcraft_geom::BezPath, xf: Affine, rule: tiny_skia::FillRule) -> Option<usize> {
        if self.plan.geometries.len() >= 16_384 {
            return None;
        }
        let g = Geometry::path(path, xf.as_coeffs(), rule == tiny_skia::FillRule::EvenOdd)?;
        self.pieces = self.pieces.checked_add(g.pieces())?;
        if self.pieces > 131_072 {
            return None;
        }
        let id = self.plan.geometries.len();
        self.plan.geometries.push(g);
        Some(id)
    }
    fn fill(&mut self, geometry: usize, color: [f64; 4]) -> Option<usize> {
        self.entry(self.plan.geometries.get(geometry)?.bounds(), Content::Fill { geometry, color })
    }
    fn group(&mut self, children: Vec<usize>, opacity: f64, pass: bool, blend: Blend, clip: Option<usize>, mask: Option<usize>) -> Option<usize> {
        let mut bounds = children.iter().filter_map(|&id| self.plan.entries.get(id).map(|e| e.bounds)).reduce(union).unwrap_or([0.; 4]);
        for id in clip.iter().chain(&mask) {
            bounds = intersect(bounds, self.plan.entries.get(*id)?.bounds);
        }
        self.entry(bounds, Content::Group { children, opacity, pass, blend, clip, mask })
    }
    fn node(&mut self, id: NodeId, parent: Affine, depth: usize, as_mask: bool) -> Option<Option<usize>> {
        self.nodes += 1;
        if self.nodes > 16_384 || depth > 60 || !self.active.insert(id) {
            return None;
        }
        let doc = self.doc;
        let n = doc.node(id)?;
        if !n.visible || n.opacity <= 0. || (n.is_mask && !as_mask) {
            self.active.remove(&id);
            return Some(None);
        }
        if !n.opacity.is_finite() || n.effects.iter().any(|e| e.visible) || n.has_visible_stroke() {
            return None;
        }
        // shortcut: gradients/images/text/effects and other shapes retain the entire existing scene renderer.
        match &n.kind {
            NodeKind::Group | NodeKind::Page { .. } => {}
            NodeKind::Frame | NodeKind::Rectangle if n.corners.radii.iter().all(|&r| r == 0.) => {}
            NodeKind::Vector { network }
                if network.vertices.iter().all(|v| v.radius == 0.) && network.regions.windows(2).all(|r| r[0].winding == r[1].winding) => {}
            _ => return None,
        }
        let xf = parent * n.transform;
        if !xf.as_coeffs().into_iter().all(|v| v.is_finite() && v.abs() <= 1e9) {
            return None;
        }
        let mut children = Vec::new();
        let is_group = matches!(n.kind, NodeKind::Group | NodeKind::Page { .. });
        let outline = (!is_group).then(|| crate::draw::node_outline(doc, n)).flatten();
        let geom = match &outline {
            Some((p, r)) => Some(self.geometry(p, xf, *r)?),
            None => None,
        };
        if !is_group {
            for paint in n.fills.iter().filter(|p| p.visible && p.opacity > 0.) {
                let PaintKind::Solid { color } = paint.kind else { return None };
                if ![color.r, color.g, color.b, color.a, paint.opacity].into_iter().all(f32::is_finite) {
                    return None;
                }
                let a = f64::from(color.a.clamp(0., 1.)) * f64::from(paint.opacity.clamp(0., 1.));
                if a == 0. {
                    continue;
                }
                let Some(g) = geom else { continue };
                let f = self
                    .fill(g, [f64::from(color.r.clamp(0., 1.)) * a, f64::from(color.g.clamp(0., 1.)) * a, f64::from(color.b.clamp(0., 1.)) * a, a])?;
                let f = if paint.blend == BlendMode::Normal { f } else { self.group(vec![f], 1., false, blend(paint.blend)?, None, None)? };
                children.push(f);
            }
        }
        let mut kids = Vec::new();
        for run in doc.paint_runs(n) {
            let kid = *n.children.get(run.start)?;
            let kn = doc.node(kid)?;
            if kn.is_mask && kn.visible {
                if kn.mask_type != MaskType::Alpha {
                    return None;
                }
                let mask = self.node(kid, xf, depth + 1, true)?;
                let mut content = Vec::new();
                for child in n.children.get(run.start + 1..run.end)? {
                    if let Some(c) = self.node(*child, xf, depth + 1, false)? {
                        content.push(c);
                    }
                }
                // An invisible/zero-alpha mask still clips all following siblings to empty.
                let mask = match mask {
                    Some(m) => m,
                    None => self.group(vec![], 1., false, Blend::Normal, None, None)?,
                };
                kids.push(self.group(content, 1., false, Blend::Normal, None, Some(mask))?);
            } else if let Some(c) = self.node(kid, xf, depth + 1, false)? {
                kids.push(c);
            }
        }
        if n.kind.is_frame_like()
            && n.clip
            && let Some(g) = geom
        {
            let clip = self.fill(g, [1.; 4])?;
            kids = vec![self.group(kids, 1., true, Blend::Normal, Some(clip), None)?];
        }
        children.extend(kids);
        let result = self.group(children, f64::from(n.opacity.clamp(0., 1.)), n.blend == BlendMode::PassThrough, blend(n.blend)?, None, None)?;
        self.active.remove(&id);
        Some(Some(result))
    }
}
