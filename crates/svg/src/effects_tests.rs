//! Shadow layers use the renderer's finite kernel and geometric masks, embedded in valid SVG.

use figcraft_color::{Color, Paint};
use figcraft_doc::{Document, Effect, EffectKind, NodeId, NodeKind, Shadow};

use super::*;

fn with_effects(kind: NodeKind, effects: Vec<EffectKind>) -> (Document, NodeId) {
    let mut d = Document::new("svg fx");
    let p = d.first_page().unwrap();
    let n = d.create(p, kind, "S", 10.0, 10.0, 100.0, 80.0).unwrap();
    let node = d.get_mut(n).unwrap();
    node.fills = vec![Paint::solid(Color::from_hex("3366CC").unwrap())];
    node.effects = effects.into_iter().map(|kind| Effect { kind, visible: true }).collect();
    (d, n)
}

fn shadow(spread: f64) -> Shadow {
    Shadow { color: Color::from_hex("00000080").unwrap(), offset: (3.0, 5.0), radius: 8.0, spread, ..Default::default() }
}

/// Export and check the result parses as SVG.
fn export(d: &Document, id: NodeId) -> String {
    use base64::Engine as _;
    let svg = export_svg(d, &[id], &SvgOptions::default()).unwrap();
    let mut d2 = Document::new("back");
    let p2 = d2.first_page().unwrap();
    import_svg(&mut d2, svg.as_bytes(), p2, 0.0, 0.0, "Imported").unwrap();
    assert_eq!(svg.matches("<image").count(), 1);
    assert!(!svg.contains("feGaussianBlur"));
    let encoded = svg.split("data:image/png;base64,").nth(1).unwrap().split('"').next().unwrap();
    let png = base64::engine::general_purpose::STANDARD.decode(encoded).unwrap();
    let view = raster_view(figcraft_render::raster_bounds(d, id).unwrap(), RASTER_SCALE).unwrap();
    let expected = figcraft_render::render_alone(d, id, &view, Alone::WithEffects).unwrap();
    assert!(png == figcraft_render::encode_png(&expected).unwrap());
    svg
}

#[test]
fn drop_shadow_spread_offsets_the_outline() {
    let star = NodeKind::Star { count: 5, inner_ratio: 0.4 };
    let (d, n) = with_effects(star, vec![EffectKind::DropShadow(shadow(6.0))]);
    let svg = export(&d, n);
    assert!(!svg.contains("feMorphology"), "{svg}");
}

#[test]
fn negative_spread_and_inner_shadows_keep_the_rendered_masks() {
    let ellipse = NodeKind::Ellipse { start: 0.0, sweep: std::f64::consts::TAU, inner: 0.0 };
    let (d, n) = with_effects(ellipse.clone(), vec![EffectKind::DropShadow(shadow(-4.0))]);
    export(&d, n);
    let (d, n) = with_effects(ellipse, vec![EffectKind::InnerShadow(shadow(4.0))]);
    export(&d, n);
}

#[test]
fn spread_without_an_outline_keeps_the_rendered_group() {
    let mut d = Document::new("g");
    let p = d.first_page().unwrap();
    let g = d.create(p, NodeKind::Group, "G", 0.0, 0.0, 50.0, 50.0).unwrap();
    let r = d.create(g, NodeKind::Rectangle, "R", 0.0, 0.0, 50.0, 50.0).unwrap();
    d.get_mut(r).unwrap().fills = vec![Paint::solid(Color::BLACK)];
    d.get_mut(g).unwrap().effects = vec![Effect { kind: EffectKind::DropShadow(shadow(5.0)), visible: true }];
    export(&d, g);
    d.get_mut(g).unwrap().effects = vec![Effect { kind: EffectKind::DropShadow(shadow(0.0)), visible: true }];
    export(&d, g);
}

#[test]
fn hostile_shadow_values_still_export() {
    for s in [f64::NAN, f64::INFINITY, -1e12, 1e12] {
        let (d, n) = with_effects(
            NodeKind::Rectangle,
            vec![
                EffectKind::DropShadow(Shadow { spread: s, radius: s, offset: (s, 0.0), ..Shadow::default() }),
                EffectKind::InnerShadow(Shadow { spread: s, radius: s, offset: (0.0, s), ..Shadow::default() }),
            ],
        );
        assert!(export_svg(&d, &[n], &SvgOptions::default()).is_some());
    }
}
