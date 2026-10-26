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

## Fresh cubic follow-up and rejected regressions

A further ordinary native UI export captures the original 800×608 cubic grid at
1x. Both complete native saves retain 434 layers and 31 pages without import
warnings. Relative to the previous complete native save, the entire typed-model
ledger contains only the added default 1x export preset. After removing it,
another native save has a typed model identical to the baseline; all three
candidate PNGs are byte-identical. These are complete typed-model ledgers, not
an independent raw-schema field oracle.

The new native residual is RGBA [255,255,255,16], including transparent RGB;
premultiplied RGBA is [16,16,16,16], with 1,309 differing pixels. It is a retained
failure. The full file also passes attached/headless MCP open, inspect, export,
save/reopen, edit, Undo/Redo and rollback (26/22 requests). Source preset changes
do not explain the residual. Native build identity remains unknown.
Three further unmodified native 1x exports match the initial PNG byte-for-byte,
including all RGBA pixels. This observed residual is therefore repeatable in
this UI route; no native-export noise allowance is claimed.

Additional whole-frame diagnostics reject a viewport fix: f32 NDC plus f32
viewport improves selected 1x line controls from max32 to1 but regresses the
axis grid from max1 to32. The original/fresh cubic controls remain at16.
Separate integral-cubic implicit classification follows the publicly published
[Loop/Blinn paper](https://www.microsoft.com/en-us/research/wp-content/uploads/2005/01/p1000-loop.pdf),
using its equations and original source geometry. Its f64 predictions reproduce
the current bisection renderer byte-for-byte. Predeclared local/world f32
variants worsen the historical maximum to17 and retain fresh max16. Neither
diagnostic changes the product. No incumbent implementation body was read.

Local follow-up evidence is gitignored `plan/figma/cubic-precision-20261026/`;
the original 124 frozen inputs and protected predecessor files still match.
The successful renderer/control arc remains `3b64680`; this follow-up makes
no further fidelity or MVP acceptance claim.

The follow-up gate exposed an existing test-harness cleanup omission: the
unsaved-changes accessibility test dropped an unconsumed egui texture delta.
It now clears the headless frame's texture output, like the adjacent helpers;
the Save/Don't Save/Cancel assertions are unchanged. This is test-only cleanup,
not a production renderer/dialog change. The failed gate is retained locally.
The focused regression and complete CI/WASM gate pass after that cleanup.

## Independent cubic shapes

A new synthetic native page contains four independently authored convex,
concave, backtracking and Y-extremum cubic families at two phases. The own SVG
builder was frozen before native pixels were read. Ordinary UI exports at
1x/2x retain the complete 452-layer, 32-page native files. Their whole typed-model
ledger differs only in the root frame's export scale 1→2; no projected fixture
or image fill is used. No independent raw-schema oracle is claimed for these
new files.

Against the unpatched `69aea87` CLI, maximum alpha improves 33→16 at 1x and
65→17 at 2x; differing alpha pixels fall 1,478→506 and 2,885→519. Candidate RGB
matches native exactly at both scales. The committed CLI and direct probe agree
on complete RGBA. These are retained residuals, not accepted error thresholds.

Headless MCP passes both original files through open, inspect, export,
save/reopen, edit, Undo/Redo and rollback. The first attached open times out
after a long idle interval; an independent state request also times out with
the app alive and idle. That failed receipt is retained. The earlier logic
dispatch fix and successful 28-scene run do not prove that the native event loop
wakes in every covered/idle-window state. This reliability issue remains open
alongside cubic AA; it is not a successful attached check for these new files.


## Native event-loop follow-up

The attached idle-window failure above has a verified correction for state
requests: the server now posts compositor-independent native logic events.
Final visible checks pass both additional files, and the same files pass
editing/export/Undo/Redo without drawing any UI frames after75s idle. The
original failure remains retained; whole-window captures still require a native
frame. See [native MCP wake](native-control-wake.md) for the exact patch, final
binary, full gate, covered-window exit and platform limits. This changes no
renderer or native AA metric and grants no MVP acceptance.

## Identity groups and fresh native scale controls

A subsequent whole-file native control retains 452 layers /32 pages and exports
the same 1200×160 curved-composition frame at1x,2x and4x. No geometry is removed
or projected. The complete typed ledgers differ only in the root export preset;
restored typed documents equal the baseline exactly. Complete raw-message
ledgers preserve the additional distinction that restoration leaves an empty
export-settings array where the baseline omitted that field. No other raw
field changes. The initial asynchronous2x save accidentally captured preset4x;
that failed acquisition is retained and a corrected2x source was saved before
further edits.

The bounded renderer declined the4x scene. A generated regression reproduces
the same class of failure:44 nested identity pass-through groups around a simple
alpha mask exhaust the work allowance although they change no coverage. The
compiler now folds only a single-child pass-through group with opacity1,
Normal blending and no clip/mask. All editable document groups remain present.
Sampling, floating resolve and all resource limits are unchanged. The regression
fails before the correction and passes afterward; clipping, isolation, opacity,
blend and mask tests remain intact.

| Fresh native scale | Unpatched69 alpha max | Before identity folding | After identity folding |
| --- | ---: | ---: | ---: |
| 1x | 96 | 16 | 16 |
| 2x | 96 | 17 | 17 |
| 4x | 112 | 112 | 17 |

At4x, differing alpha pixels fall682,910→226,428. There are still transparent-RGB
and alpha differences; this table grants no error tolerance. All63 previous
whole typed models and55 previous PNGs are byte-identical after folding.
The new3-scale files also pass actual attached MCP edits, save/reopen,
keyboard Undo/Redo and failed-batch rollback; commands preserve editable layers.

### Historical PNGs do not establish current native equality

The new native2x export is repeatable: a second ordinary Export PNG, Copy as PNG,
and both Basic/Detailed image-resampling choices all produce exactly the same
PNG bytes for this vector-only frame. They still differ from the historical
native PNG. Native half-mask interiors now contain127 where the historical
reference and candidate contain128; new curved boundaries retain larger errors.
The old candidate and new candidate outputs at2x are themselves byte-identical.
Thus the earlier curved-control max1 remains a measurement against that frozen
historical artifact, not a claim about every current native export.

Data-only source attribution checks all12 original vector-network blobs against
the new whole file byte-for-byte. The10 newly present cached cubic paths have
control points exactly equal to the original network arithmetic. Cached path
presence, clipboard-versus-export choice and Basic/Detailed resampling therefore
provide no verified explanation or correction. The generic raw decoder is
self-checked against the previously independent24-node whole-message oracle;
no independent decoder claim is made for the fresh whole native files.
Native build/backend identity remains unknown. No vendor implementation body
was read, and no reference-derived per-pixel adjustment was installed.

Predeclared2×2 box resolves of the native4x/2x images also fail to reproduce the
native2x/1x outputs, with alpha maxima28–32 across floor/nearest/ties-even rules.
Those diagnostics are rejected. Evidence and retained failures stay local in
`plan/figma/resolve-scale-20261026/`. Cubic AA, placement-sensitive1x cases,
unsupported mixed scenes, font/library fidelity and the overall1:1 MVP remain
open.

Final folding verification passes the complete `cargo +1.97.1 xtask ci` gate,
including WASM/plugins, and810 actual attached MCP requests:28 previous scenes,
two independent cubic controls, three fresh scale controls and editable
frame/rectangle/text authoring across the18 entry points. Another39 raw window
calls verify exact inspector edit/Undo/Redo. The final desktop4x node capture
matches the direct renderer's complete RGBA; the actual window capture was read.
The local `validation-final.json` seals source/binary hashes, preserved inputs,
full ledgers, gate and workflow receipts. These checks verify the implementation
and transport; native pixel equality and MVP acceptance remain false.
