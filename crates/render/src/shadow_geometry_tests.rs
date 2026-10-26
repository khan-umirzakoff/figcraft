//! Source-bound shadow invariants: geometric caster, painted body and closed-stroke knockout.
use super::*;
use figcraft_color::{Color, Paint};
use figcraft_doc::{Document, Effect, EffectKind, NodeId, NodeKind, Shadow, VectorNetwork};
use figcraft_geom::Point;

fn scene(kind: NodeKind, effect: EffectKind) -> (Document, NodeId, NodeId) {
    let mut d = Document::new("shadow geometry");
    let page = d.first_page().unwrap();
    let frame = d.create(page, NodeKind::Frame, "Frame", 0.0, 0.0, 160.0, 160.0).unwrap();
    d.get_mut(frame).unwrap().fills.clear();
    let id = d.create(frame, kind, "Body", 40.0, 40.0, 80.0, 80.0).unwrap();
    let n = d.get_mut(id).unwrap();
    n.effects = vec![Effect { kind: effect, visible: true }];
    n.fills = vec![Paint::solid(Color::WHITE)];
    (d, frame, id)
}
fn render(d: &Document, id: NodeId) -> Pixmap {
    let mut p = Pixmap::new(160, 160).unwrap();
    render_nodes_into(d, &[id], &View { origin: (0.0, 0.0), scale: 1.0, width: 160, height: 160 }, &mut p);
    p
}
fn px(p: &Pixmap, x: usize, y: usize) -> [u8; 4] {
    p.data()[(y * 160 + x) * 4..(y * 160 + x + 1) * 4].try_into().unwrap()
}
fn shadow() -> Shadow {
    Shadow { color: Color::BLACK.with_alpha(0.5), radius: 12.0, offset: (4.0, 4.0), ..Shadow::default() }
}
#[test]
fn translucent_paint_does_not_fade_the_geometric_drop_caster() {
    let (mut d, frame, id) = scene(NodeKind::Rectangle, EffectKind::DropShadow(shadow()));
    let opaque = px(&render(&d, frame), 36, 80)[3];
    assert!(opaque > 0);
    for alpha in [0.25, 0.5, 0.75] {
        d.get_mut(id).unwrap().fills = vec![Paint::solid(Color::WHITE.with_alpha(alpha))];
        assert_eq!(px(&render(&d, frame), 36, 80)[3], opaque);
    }
}
#[test]
fn zero_paint_does_not_create_an_inner_shadow_body() {
    let (mut d, frame, id) = scene(NodeKind::Rectangle, EffectKind::InnerShadow(shadow()));
    for stroke in [false, true] {
        let n = d.get_mut(id).unwrap();
        n.fills = if stroke { vec![] } else { vec![Paint::solid(Color::WHITE.with_alpha(0.0))] };
        n.strokes = if stroke { vec![Paint::solid(Color::WHITE.with_alpha(0.0))] } else { vec![] };
        n.stroke.weight = 8.0;
        assert!(render(&d, frame).data().as_chunks::<4>().0.iter().all(|p| p[3] == 0));
    }
}
#[test]
fn stroke_inner_shadow_clips_to_the_ring_and_paints_over_its_colour() {
    let (mut d, frame, id) = scene(NodeKind::Rectangle, EffectKind::InnerShadow(shadow()));
    let n = d.get_mut(id).unwrap();
    n.fills.clear();
    n.strokes = vec![Paint::solid(Color::new(1.0, 0.0, 0.0, 0.5))];
    n.stroke.weight = 8.0;
    n.stroke.align = figcraft_doc::StrokeAlign::Center;
    let p = render(&d, frame);
    assert_eq!(px(&p, 48, 80)[3], 0, "the unpainted interior is not the inner-shadow clip");
    let edge = px(&p, 41, 80);
    assert!(edge[0] < 128 && edge[3] > 128, "shadow over translucent red: {edge:?}");
}
#[test]
fn recorded_closed_region_knocks_out_a_stroke_drop_shadow() {
    let network = VectorNetwork::from_points(&[Point::new(0.0, 0.0), Point::new(80.0, 0.0), Point::new(80.0, 80.0), Point::new(0.0, 80.0)], true);
    let s = Shadow { color: Color::BLACK, offset: (12.0, 0.0), radius: 0.0, ..Shadow::default() };
    let (mut d, frame, id) = scene(NodeKind::Vector { network }, EffectKind::DropShadow(s.clone()));
    let n = d.get_mut(id).unwrap();
    n.fills.clear();
    n.strokes = vec![Paint::solid(Color::WHITE)];
    n.stroke.weight = 8.0;
    n.stroke.align = figcraft_doc::StrokeAlign::Center;
    assert_eq!(px(&render(&d, frame), 49, 80)[3], 0);
    d.get_mut(id).unwrap().effects[0].kind = EffectKind::DropShadow(Shadow { show_behind: true, ..s });
    assert_eq!(px(&render(&d, frame), 49, 80)[3], 255);
}
