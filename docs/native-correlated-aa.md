# Correlated solid-scene antialiasing

2026-10-26, M7; baseline `69aea87`. Native 1:1 and the requested full MVP remain
unaccepted. Results below are whole-source comparisons, not passing tolerances.

## Shared renderer integration

The existing renderer now keeps geometric sample correlation for bounded solid
scenes. A private compiler reads the existing typed Document, vector regions and
BezPath outlines directly; there is no SVG parser, alternate document model,
command API or imported reference image. Original f64 curves are split at Y
extrema and intersected by bounded monotone bisection, without flattening,
geometry fitting or a boundary epsilon. The fixed sixteen-point pattern and
composition math come from the predecessor's independent synthetic UI experiments.
No incumbent bundle, shader body or executable implementation was used here.

Paint stacks, Normal and Pass-through containers, alpha masks and frame clips
compose per sample before final premultiplied pixel resolve. The supported
Screen, Lighten and Plus-lighter formulas also see the actual existing backdrop,
including nonuniform pixels. This avoids independently resolving an edge and
multiplying its marginal alpha again in a group or mask. PNG export, the canvas,
retained partial views and MCP use the shared renderer entrypoints. The internal
and saved layers stay editable.

## Original native controls

The original 58 carriers still import without warnings and their complete typed
models remain byte-identical. Seven of their 50 PNG outputs change; the other
43, including all previous blur/shadow controls, remain byte-identical.

| Entire original frame | Before maximum alpha /255 | Candidate maximum alpha /255 |
| --- | --- | --- |
| Axis phases | 64 | 1 |
| Line phases | 17 | 1 |
| Joint phases | 64 | 1 |
| Cubic fill phases | 48 | 16 |
| Rectangle/group/mask boundaries | 32 | 0 |
| Second rectangle/group/mask phase | 16 | 0 |
| Curved group/mask composition | 96 | 1 |

Both rectangle/group/mask PNGs match complete native RGBA. The curved composition
matches RGB and retains alpha differences up to 1. The cubic grid's six earlier
nearest-sixteen-count diagnostic mismatches remain; no failed-pixel correction was added.

Three further immutable direction/placement controls are also replayed. At 2x,
maximum alpha improves 48→1. At 1x, both ordinary and counterbalanced-direction
controls retain maximum 32, although differing alpha pixels fall 540→253. These
are retained failures. Reversing a contour is not used as a correction.

## Fresh native exports

Ordinary authenticated Chrome Figma UI exported the same synthetic direction
frame at 2x and 1x and saved two complete native `.fig` copies. Each imports as
434 layers on 31 pages, including 279 vectors, without warnings. The complete
typed-model difference ledger contains exactly one change: the frame's export
scale value 2→1. Geometry, paints and parent context are unchanged. The preset
was restored to 2x afterward, verified in the UI; downloads and screenshots stay
local, never shipped assets. The native build identity is unknown.

Fresh native RGBA residuals are [0,0,0,1] at 2x and [0,0,0,32] at 1x. Thus the
position/edge-tie defect is still present in current exports. These controls do
not establish a native quantizer or a universal sample/precision model. The
complete file is synthetic; importing its other pages without warnings does
not establish component, library or prototype visual fidelity.

## Profile, bounds and verification

The correlated path accepts solid fills on sharp rectangles/frames and supported
closed vector regions, groups and alpha masks. Mixed winding regions, rounded
primitive corners, per-vertex rounding, gradients, strokes, images, text,
effects and other active features decline the **whole** scene to the existing
renderer. Outline/solo/skip views and low-zoom LOD also retain their existing
path. General affine math is supported, but native acceptance extends only to
captured controls. Adding unsupported active content can therefore change which
AA path renders a scene; arbitrary mixed scenes are not accepted.

Traversal, geometry, depth, pixel area, memory and work limits are checked before
touching destination pixels. Per-call accounting caps conservative working
storage at 64 MiB, including the output/backdrop allowance, row caches and worst
bucket growth of Y-solve memoization. It is not process RSS or a cross-thread
memory/FPS guarantee. Excessive work declines the entire scene. No new public
renderer setting or dependency was added.

Constant interior tiles evaluate geometry once but retain all sixteen original
floating additions. Identical Y predicates share only their original bisection
t value; each curve evaluates its own original world X. Caches clear each scanline.
All 63 complete models and 55 PNGs, including the fresh/held-out cases, are
byte-identical before/after these optimizations. Generated tests cover shared
mask and paint coverage, Normal opacity, backdrop blending, unsupported-scene
fallback, invalid/cyclic data, cubic crossings, constant-tile equality and
banded/partial views. Standalone vector SVG/PDF viewers still choose their own
AA; no export-format parity follows from these PNG comparisons.

Local evidence: gitignored `plan/figma/correlated-aa-20261026/`: frozen inputs,
whole models, before/after PNGs, full residuals, fresh native source ledger,
rejected/failed receipts, benchmarks and validation logs. Reference screenshots
remain under gitignored `plan/figma/screenshots/correlated-aa-20261026/`.

## Attached control dispatch

Stress testing exposed an attached keyboard Undo request waiting for a frame,
with the app alive and idle; another socket state query timed out too. The
control bridge previously deferred all requests while visibility was not known
hidden. State requests now run during UI-thread logic for visible/unknown/hidden
windows, without requiring a compositor frame callback. Only window captures,
raw window input and new windows remain frame-bound. Edits still request a
repaint. Generated regressions cover Undo/inspection with unknown visibility,
per-window document routing and frame-only deferral. This changes no wire method,
file permission or engine command. Actual final runtime validation is recorded
with the local receipts, including the failed attempt.

JSON inspection also stops rounding positions, sizes and absolute bounds to two
decimals. A source X of 1016.375 previously returned 1016.38 to agents. The shared
engine fix reaches CLI, headless/attached MCP and document inspection; a grouped
read/write regression preserves the original transform and size. Panel display
rounding remains, as does the existing rounded rotation representation. This is
not a lossless serialization of every node property.

## Actual desktop and MCP checks

The final native Wayland app and headless CLI replay 28 original scenes each:
open, select the source page, inspect, export, save/reopen, edit opacity,
Undo/Redo and failing-batch rollback. Every original/restored PNG matches the
direct renderer. Separate authoring creates editable frame/rectangle/text
layers, types text and saves/reopens without changing pixels. Across these
workflows all 18 MCP entrypoints run: 698 attached and 584 headless requests.
Counts establish exercised transport paths, not visual fidelity.

The desktop inspector's X field was clicked and typed through raw window input:
1016.375→1016.5. Actual window keyboard Undo/Redo restores exact coordinates and
byte-identical original/edited PNGs. Captures were read visually; the final
window shows the restored editable mask group. File confinement stays in the
existing UI guard; no new API method or capability was added.

Performance remains a limitation. Whole-CLI paired runs of the final renderer
build (before the inspection-only precision fix) include startup, import, PNG
encoding and disk I/O: candidate medians 0.042–0.284 s versus baseline
0.020–0.057 s across the seven changed controls. These are not frame times.
Desktop full/partial captures confirm actual rendering, not interactive FPS
acceptance. The optional craft-fonts checkout is absent; native font fidelity,
real iOS/Material libraries and complex prototype motion remain unaccepted.
The complete `cargo +1.97.1 xtask ci` gate exits 0 after the final source changes
(formatting, workspace clippy/tests, layering/assets and WASM/plugin builds).
The local validation receipt rechecks all 124 frozen inputs, fresh native
downloads, optimization equality and the three protected predecessor files.

## Retained precision hypotheses

An isolated diagnostic preregistered identity and three textbook f32
orthographic/viewport models before reading comparison pixels. All predictions
use the complete original linear frame; identity reproduces the current
renderer exactly. One model reduces the historical/counterbalanced and fresh
1x linear maximum from 32 to 1. The same models, applied to the complete cubic
grid, retain maximum 16. Their identity output again reproduces the current
renderer. No viewport quantization, epsilon, orientation correction or
failed-pixel rule was added to the product. More independent native dimensions
and curve evidence are required before treating this hypothesis as a correction.
