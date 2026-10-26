use super::*;
use figcraft_color::{BlendMode, Color, Gradient, GradientKind, Paint};
use figcraft_doc::{Document, NodeKind};

fn doc() -> (Document, NodeId) {
    let mut d = Document::new("sample correlation");
    let page = d.first_page().unwrap();
    let f = d.create(page, NodeKind::Frame, "Frame", 0., 0., 4., 4.).unwrap();
    d.get_mut(f).unwrap().fills.clear();
    (d, f)
}
fn rect(d: &mut Document, parent: NodeId) -> NodeId {
    let id = d.create(parent, NodeKind::Rectangle, "Shape", 0.25, 0.125, 2., 2.).unwrap();
    d.get_mut(id).unwrap().fills = vec![Paint::solid(Color::WHITE)];
    id
}
fn image(d: &Document, root: NodeId) -> Pixmap {
    render(d, &[(root, Affine::IDENTITY)], &Pixmap::new(4, 4).unwrap()).unwrap()
}
#[test]
fn identical_alpha_mask_preserves_fractional_shape_coverage() {
    let (mut d, f) = doc();
    let mask = rect(&mut d, f);
    d.get_mut(mask).unwrap().is_mask = true;
    rect(&mut d, f);
    let masked = image(&d, f);
    d.remove(mask).unwrap();
    assert_eq!(masked.data(), image(&d, f).data(), "the mask shares sample membership, not squared resolved alpha");
    assert!(masked.data().as_chunks::<4>().0.iter().any(|p| p[3] > 0 && p[3] < 255));
}
#[test]
fn normal_group_opacity_and_alpha_mask_compose_before_resolve() {
    let (mut d, f) = doc();
    let group = d.create(f, NodeKind::Group, "Group", 0., 0., 0., 0.).unwrap();
    let n = d.get_mut(group).unwrap();
    n.blend = BlendMode::Normal;
    n.opacity = 0.5;
    rect(&mut d, group);
    let grouped = image(&d, f);
    d.remove(group).unwrap();
    let mask = rect(&mut d, f);
    let n = d.get_mut(mask).unwrap();
    n.is_mask = true;
    n.opacity = 0.5;
    rect(&mut d, f);
    assert_eq!(grouped.data(), image(&d, f).data());
}
#[test]
fn solid_paint_stack_shares_one_geometric_coverage() {
    let (mut d, f) = doc();
    let id = rect(&mut d, f);
    let single = image(&d, f);
    d.get_mut(id).unwrap().fills.push(Paint::solid(Color::WHITE));
    assert_eq!(single.data(), image(&d, f).data());
}
#[test]
fn pass_through_blends_see_the_existing_nonuniform_backdrop() {
    let (mut d, f) = doc();
    let n = d.get_mut(f).unwrap();
    n.blend = BlendMode::PassThrough;
    let id = rect(&mut d, f);
    d.get_mut(id).unwrap().transform = Affine::IDENTITY;
    d.get_mut(id).unwrap().fills = vec![Paint::solid(Color::new(1., 0., 0., 1.))];
    d.get_mut(id).unwrap().blend = BlendMode::Screen;
    let mut under = Pixmap::new(4, 4).unwrap();
    under.fill(tiny_skia::Color::from_rgba8(0, 0, 255, 255));
    under.data_mut()[4..8].copy_from_slice(&[0, 255, 0, 255]);
    let result = render(&d, &[(f, Affine::IDENTITY)], &under).unwrap();
    assert_eq!(&result.data()[..8], &[255, 0, 255, 255, 255, 255, 0, 255]);
    assert_eq!(&result.data()[12..16], &under.data()[12..16]);
}
#[test]
fn unsupported_active_content_declines_the_whole_scene() {
    let (mut d, f) = doc();
    rect(&mut d, f);
    let id = rect(&mut d, f);
    d.get_mut(id).unwrap().fills = vec![Paint::gradient(Gradient::default_for(GradientKind::Linear, Color::WHITE))];
    let mut target = Pixmap::new(4, 4).unwrap();
    target.fill(tiny_skia::Color::WHITE);
    let before = target.data().to_vec();
    assert!(render(&d, &[(f, Affine::IDENTITY)], &target).is_none());
    assert_eq!(target.data(), before);
    d.get_mut(id).unwrap().visible = false;
    assert!(render(&d, &[(f, Affine::IDENTITY)], &target).is_some());
}
#[test]
fn invalid_geometry_and_excessive_work_decline_without_mutation() {
    let (mut d, f) = doc();
    let id = rect(&mut d, f);
    d.get_mut(id).unwrap().transform = Affine::scale(f64::NAN);
    assert!(render(&d, &[(f, Affine::IDENTITY)], &Pixmap::new(4, 4).unwrap()).is_none());
    assert!(compile::scene(&d, &[(f, Affine::IDENTITY)], u32::MAX, 2).is_none());
    d.get_mut(id).unwrap().transform = Affine::IDENTITY;
    d.get_mut(id).unwrap().children.push(f);
    assert!(render(&d, &[(f, Affine::IDENTITY)], &Pixmap::new(4, 4).unwrap()).is_none());
}
#[test]
fn cubic_crossings_close_the_original_f64_path() {
    let mut path = figcraft_geom::BezPath::new();
    path.move_to((0., 0.));
    path.curve_to((0., 1.), (1., 1.), (1., 0.));
    let g = Geometry::path(&path, Affine::IDENTITY.as_coeffs(), false).unwrap();
    let mut solves = super::geometry::Solves::new();
    let row = g.row(0.5, &mut solves);
    assert!(row.contains(0.5));
    assert!(!row.contains(0.01));
    let count = solves.len();
    let shifted = Geometry::path(&path, Affine::translate((20., 0.)).as_coeffs(), false).unwrap();
    assert!(shifted.row(0.5, &mut solves).contains(20.5));
    assert_eq!(solves.len(), count, "only the identical Y solve is reused, not X coverage");
    assert!(!g.row(0.8, &mut solves).contains(0.5));
    assert!(Geometry::path(&path, Affine::scale(f64::INFINITY).as_coeffs(), false).is_none());
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn correlated_masks_match_banded_and_partial_views() {
    let (mut d, f) = doc();
    let page = d.first_page().unwrap();
    let n = d.get_mut(f).unwrap();
    n.width = 320.;
    n.height = 256.;
    for mask in [true, false] {
        let id = d.create(f, NodeKind::Rectangle, "Tall", 24.25, 12.125, 220., 232.).unwrap();
        let n = d.get_mut(id).unwrap();
        n.is_mask = mask;
        n.fills = vec![Paint::solid(Color::WHITE.with_alpha(0.5))];
    }
    for scale in [1., 1.5, 2.] {
        let view = crate::View { origin: (-17.25, -12.5), scale, width: 512, height: 384 };
        let opts = crate::Options { background: Some(Color::new(0., 0., 0., 0.)), ..Default::default() };
        let full = crate::render_view_single(&d, page, &view, &opts).unwrap();
        let bands = crate::render_bands(&d, page, &view, &opts, 4).unwrap();
        assert_eq!(full.data(), bands.data());
        let part = crate::View { origin: (view.origin.0 + 16. / scale, view.origin.1 + 97. / scale), width: 160, height: 120, ..view.clone() };
        let crop = crate::render_view_single(&d, page, &part, &opts).unwrap();
        for y in 0..120usize {
            assert_eq!(&crop.data()[y * 160 * 4..(y + 1) * 160 * 4], &full.data()[((y + 97) * 512 + 16) * 4..((y + 97) * 512 + 176) * 4]);
        }
    }
}
