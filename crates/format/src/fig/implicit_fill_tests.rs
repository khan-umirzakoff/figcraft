//! Legacy native carriers omit regions and cached outlines for a filled closed contour.
use super::super::network::{self, Network, Region, Segment, Vertex};
use super::*;
use figcraft_doc::geom::Point;

fn square() -> Network {
    Network {
        vertices: [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)].into_iter().map(|(x, y)| Vertex { style: 0, x, y }).collect(),
        segments: [(0, 1), (1, 2), (2, 3), (3, 0)].into_iter().map(|(a, b)| Segment { style: 0, a, b, ta: (0.0, 0.0), tb: (0.0, 0.0) }).collect(),
        regions: vec![],
    }
}

fn import_network(net: &Network, filled: bool) -> (Document, Vec<String>) {
    let nodes = vec![
        node(0, None, "DOCUMENT", "D", vec![]),
        node(1, Some((0, "!")), "CANVAS", "P", vec![]),
        node(
            2,
            Some((1, "!")),
            "VECTOR",
            "V",
            vec![
                ("size", xy(20.0, 20.0)),
                ("vectorData", o(vec![("vectorNetworkBlob", u(0)), ("normalizedSize", xy(10.0, 10.0))])),
                ("fillPaints", arr(if filled { vec![solid(1.0, 0.0, 0.0)] } else { vec![] })),
            ],
        ),
    ];
    let msg = o(vec![("nodeChanges", arr(nodes)), ("blobs", arr(vec![o(vec![("bytes", Kv::Bytes(network::encode(net)))])]))]);
    import(&canvas(&schema(), &msg), "test").unwrap()
}

fn net(doc: &Document) -> &figcraft_doc::VectorNetwork {
    let NodeKind::Vector { network } = &named(doc, "V").kind else { panic!() };
    network
}

#[test]
fn a_legacy_closed_fill_keeps_its_editable_segments_and_handles() {
    let mut source = square();
    source.segments.swap(0, 2);
    source.segments[0].ta = (0.25, -0.5);
    let segment = &mut source.segments[1];
    std::mem::swap(&mut segment.a, &mut segment.b);
    let (doc, warnings) = import_network(&source, true);
    assert!(warnings.is_empty(), "{warnings:?}");
    let n = net(&doc);
    assert!(n.fill_contains(Point::new(10.0, 10.0)));
    assert_eq!((n.vertices.len(), n.segments.len(), n.regions.len()), (4, 4, 1));
    for (s, expected) in n.segments.iter().zip(&source.segments) {
        assert_eq!((s.a, s.b), (expected.a, expected.b));
        assert_eq!(s.ta, (expected.ta.0 * 2.0, expected.ta.1 * 2.0));
        assert_eq!(s.tb, (expected.tb.0 * 2.0, expected.tb.1 * 2.0));
    }
}

#[test]
fn unfilled_networks_stay_unfilled_and_explicit_holes_keep_their_rule() {
    let (doc, warnings) = import_network(&square(), false);
    assert!(warnings.is_empty());
    assert!(net(&doc).regions.is_empty());
    let mut source = square();
    source.regions.push(Region { style: 0, non_zero: false, loops: vec![vec![0, 1, 2, 3], vec![0, 1, 2, 3]] });
    let (doc, warnings) = import_network(&source, true);
    assert!(warnings.is_empty());
    assert_eq!(net(&doc).regions[0].loops, source.regions[0].loops);
    assert!(!net(&doc).fill_contains(Point::new(10.0, 10.0)));
}

#[test]
fn ambiguous_regionless_paths_warn_without_inventing_faces() {
    let mut open = square();
    open.segments.pop();
    let mut branch = square();
    branch.vertices.push(Vertex { style: 0, x: 20.0, y: 20.0 });
    branch.segments.push(Segment { style: 0, a: 2, b: 4, ta: (0.0, 0.0), tb: (0.0, 0.0) });
    let mut disjoint = square();
    disjoint.vertices.extend(square().vertices.into_iter().map(|v| Vertex { x: v.x + 20.0, ..v }));
    disjoint.segments.extend(square().segments.into_iter().map(|s| Segment { a: s.a + 4, b: s.b + 4, ..s }));
    let mut self_loop = square();
    self_loop.segments[0].b = self_loop.segments[0].a;
    for source in [open, branch, disjoint, self_loop] {
        let (doc, warnings) = import_network(&source, true);
        assert!(net(&doc).regions.is_empty());
        assert_eq!(net(&doc).segments.len(), source.segments.len());
        assert!(warnings.iter().any(|w| w.contains("regionless vector fill")), "{warnings:?}");
    }
}

#[test]
fn exporting_an_unpainted_face_preserves_its_unused_paints_on_reopen() {
    let mut source = square();
    source.regions.push(Region { style: 0, non_zero: true, loops: vec![vec![0, 1, 2, 3]] });
    let (mut doc, _) = import_network(&source, true);
    let id = named(&doc, "V").id;
    let (image, _, _) = crate::add_image(&mut doc, png()).unwrap();
    doc.node_mut(id).unwrap().fills.push(figcraft_doc::color::Paint::image(image.clone(), figcraft_doc::color::ImageScaleMode::Fill));
    let original = doc.node(id).unwrap().clone();
    let NodeKind::Vector { network } = &mut doc.node_mut(id).unwrap().kind else { panic!() };
    network.regions.clear();
    let (bytes, _) = export(&doc, None).unwrap();
    let (reopened, _) = import(&bytes, "reopened").unwrap();
    assert!(!net(&reopened).fill_contains(Point::new(10.0, 10.0)));
    assert_eq!(named(&reopened, "V").fills, original.fills);
    assert_eq!(net(&reopened).vertices, net(&doc).vertices);
    assert_eq!(net(&reopened).segments, net(&doc).segments);
    assert_eq!(reopened.images.get(&image), doc.images.get(&image), "unused image paint data survives");
    let archive = Archive::open(&bytes).unwrap();
    let (sc, msg) = decode_canvas(&archive.canvas).unwrap();
    let msg = sc.decode_message(sc.find("Message").unwrap(), &msg).unwrap();
    let node = msg.get("nodeChanges").unwrap().list().iter().find(|n| n.get("name").and_then(Kv::str) == Some("V")).unwrap();
    assert!(
        node.get("fillPaints").unwrap().list().iter().all(|p| p.get("visible").and_then(Kv::bool) == Some(false)),
        "native readers must not show a fill"
    );
}
