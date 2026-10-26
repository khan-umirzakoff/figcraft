//! Bounded solid scenes retain geometric sample correlation until the final pixel resolve.
//! Math comes from the independently measured native-control experiment, not vendor code.
mod compile;
mod geometry;
mod raster;
#[cfg(test)]
mod tests;

use figcraft_doc::{Document, NodeId};
use figcraft_geom::Affine;
use geometry::Geometry;
use tiny_skia::Pixmap;

#[derive(Clone, Copy, Debug)]
enum Blend {
    Normal,
    Screen,
    Lighten,
    Plus,
}
enum Content {
    Fill { geometry: usize, color: [f64; 4] },
    Group { children: Vec<usize>, opacity: f64, pass: bool, blend: Blend, clip: Option<usize>, mask: Option<usize> },
}
struct Entry {
    bounds: [f64; 4],
    content: Content,
}
struct Plan {
    entries: Vec<Entry>,
    geometries: Vec<Geometry>,
    root: usize,
    width: u32,
    height: u32,
}

/// Unsupported active content or a resource bound declines the entire scene without touching pixels.
pub(crate) fn render(doc: &Document, roots: &[(NodeId, Affine)], backdrop: &Pixmap) -> Option<Pixmap> {
    let plan = compile::scene(doc, roots, backdrop.width(), backdrop.height())?;
    raster::render(&plan, backdrop)
}
