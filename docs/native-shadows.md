# Native shadows: bounded geometry/body/knockout controls

2026-10-26, M7. Baseline `d8fdac8`; no full native or MVP acceptance.

Twenty original synthetic native clipboard selections cover positive-alpha
linear/radial fills and strokes, paint opacity 1/.5/0, default/behind drop
shadows, inner shadows, three stroke alignments and fractional curved paths.
The earlier solid default-drop control is replayed too. All source fields,
regions, paths, parents and clipping contexts are retained. Frame crops come
from the original source dimensions before the candidate runs; native PNGs
are read by a separate comparison step, never by the renderer.

The kernel-only experiment improved the solid control but worsened one stroke
drop maximum from 60 to 70. Its failed result is retained. The root cause was
not only filtering: the renderer conflated three different planes:

- **P**, actual premultiplied painted pixels, with their colour and alpha.
- **G**, active opaque painted geometry, used for the drop caster and inner clip.
- **K**, default-drop knockout: a closed stroke includes its recorded interior
  plus its ring, despite having no fill paint. The interior is not part of G.

The renderer now uses its existing typed outlines and stroke cache to build
these independently. Positive paint opacity does not fade G. An inactive
paint does not create an inner-shadow body. A single-outline leaf's inner
shadow clips to G and composites over P, including a translucent stroke;
the empty middle stays empty. Drop/inner filters share the finite triangular
kernel, with floating passes and one byte resolve. Default drop bounds use
full-radius support plus spread/offset; the solid native default control
exports at 528×272 at 2×, rather than Gaussian padding.

| Control | Before maximum alpha /255 | Candidate maximum alpha /255 |
| --- | --- | --- |
| Solid default drop | 5 | 2 |
| Gradient fill drop, full/half/behind | 61–94 | 2 |
| Gradient stroke drop, full/half/behind | 58–75 | 2 |
| Fill inner, full/half | 3–6 | 2 |
| Stroke inner, linear/radial/alignments | 128 | 1–2 |
| Zero-opacity fill/stroke inner | 128 | 0 |
| Fractional curved drop / inner | 43 / 50 | 17 / 21 |
| Curved body without effects | 19 | 19 (unchanged) |

These are residual measurements, **not passing parity tolerances**. Full RGBA
and premultiplied residuals are retained; transparent RGB differences are not
discarded. No native baseline image is used as body, coverage or a fitted
correction. Curve coverage and subsequent mask/composition differences remain.

## Profile and limits

Geometric casting is enabled for childless non-text layers with active solid
paints or linear/radial paints whose stops all have positive alpha. Empty and
scalar-zero paint is inactive. Images, colour glyphs, spatial zero-alpha stops,
moving body effects and child subtrees keep their existing pixel-caster path;
they are not accepted by these controls. Background/glass underlays do not
enter the caster. Existing spread handling remains, including its unaccepted
fallbacks. Mixed-outline and child inner-shadow ordering remains unchanged.

SVG shadowed layers now use the existing whole-layer raster path, as blurred
layers already do, so SVG's different Gaussian primitive does not substitute
for the renderer. The exported SVG effect is an image; the internal and
`.figcraft` layers remain editable. Ordinary un-effected SVG shapes stay vectors.
SVG tests compare embedded PNG bytes with the renderer, including spread,
negative spread, groups and mixed fill/stroke ordering. PDF shares the filter and drop-caster implementation, but retains a separate
vector-content/inner-image composition path; equivalent inner clip/ordering
is not established there. Oversized/empty raster fallbacks remain unaccepted.

Generated regression tests distinguish a geometric caster from paint alpha,
empty geometry from an inner-shadow body, an inner clip from a solid region,
and a stroke ring from its recorded knockout. Earlier Gaussian/paint-alpha
test expectations were corrected using the retained native controls.

Local evidence: gitignored `plan/figma/shadows-20261026/` (original input
hashes, whole imported models, baseline, rejected kernel-only result,
geometry candidate, full residuals and verification logs). Native originals
remain unchanged in the earlier reference archive; none is a shipped asset.

## Verification of geometry/body/knockout (`2b873d3`)

Full `cargo +1.97.1 xtask ci` passed, including native tests, lint, layering,
assets and WASM. All 58 intact carriers replay without warnings; their whole
imported models are byte-identical to baseline. Fifty have native PNG comparisons.
Unrelated render outputs and the three shadow-free body controls are unchanged.
Near-zero activation thresholds outside the captured values remain unaccepted.

The actual Linux Wayland desktop and both attached and headless MCP were exercised
with this build. Eighteen retained native scenes per mode matched direct renderer
pixels through open, save/reopen, edit, undo/redo and failed-batch rollback.
Attached keyboard Undo was checked. Separate MCP authoring created editable
frame, rounded rectangle and text layers; all 18 MCP entrypoints were exercised
across these checks. Dynamic fill/stroke opacity 0→.5 and undo/redo matched the
original half-opacity model's render. Actual window captures were visually
inspected. These workflow checks establish command/render consistency, not native
visual equivalence, font parity, prototype fidelity or representative library QA.

## Shared inner-edge coverage

A follow-up against `2b873d3` removes double geometric coverage for one Normal
paint without a backdrop. Painted pixels P and already-clipped inner shadow S
share clip C, so their resolved composition is `S + P * (1 - S.alpha / C)`, with
zero C skipped. Independent SourceOver of both clipped images incorrectly
increases an opaque body's edge coverage. Multiple effects reuse the same clip;
paint stacks, backdrops, mixed outlines and child composition retain their
existing path until sample-correlated composition is established.

A fractional rotated rounded-body test verifies that two inner shadows leave
its opaque edge alpha unchanged; a partial translucent-colour test distinguishes
shared clipping from independent SourceOver. The 58 whole imported models remain
byte-identical. Of 50 native PNG comparisons, 49 outputs are unchanged; curved
inner shadow alpha maximum improves 27→21 and alpha MAE .05594→.05300 /255.
One maximum premultiplied blue residual increases 14→14.84 /255 while its mean
absolute straight-blue error decreases .11076→.10857. All residuals are retained;
this is not full-colour or native 1:1 acceptance. Shadow-free curved body alpha
still differs by 19; group/mask and primitive AA cases are unchanged.

Local follow-up evidence: gitignored `plan/figma/inner-coverage-20261026/`.

The follow-up passed full `cargo +1.97.1 xtask ci` and was also run on the real Wayland desktop. Three relevant
scenes per mode passed attached/headless MCP open/save/reopen, direct-render
comparison, edit/undo/redo and failed-batch rollback. Actual inspector X and
inner-shadow Blur fields were changed with raw mouse/keyboard input; both
changed rendered pixels and keyboard Undo/Redo restored exact PNG bytes. The
original values and editable curve scene were restored. Window captures were
visually inspected. This checks those fields, not every inspector setting.
