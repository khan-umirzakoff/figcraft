# Native fidelity: bounded controls

2026-10-26, M6/M7. This fork keeps the shared FigCraft document/command/renderer
architecture. Changes are checked against original synthetic native outputs;
successful imports, command counts and generated round trips are not visual parity.

## Legacy regionless filled contours

[Figma's public VectorNetwork specification](https://developers.figma.com/docs/plugins/api/VectorNetwork/)
describes automatic filling of enclosed space when no regions are supplied,
and unordered segment directions within continuous loops. The recovery below
uses that documented behavior plus native synthetic controls; it does not
implement general automatic face reconstruction.

Two intact historical native clipboard carriers (`AA native boundaries` and
`AA native boundary quarter phase`) imported ten editable vectors each with
no warnings, but their 1280 × 256 renders were entirely transparent. Each
network was one closed four-segment contour with visible fills, no explicit
regions and no cached fill outline.

The importer now reconstructs one nonzero region only for a single closed
chain. A bounded linear walk preserves source vertices, segment order,
directions, handles, scaling and point styles. This changes the imported
document once; hit testing, rendering, masks and export consume the existing
region representation. It adds no parallel rendering or document engine.

Explicit regions, winding rules and holes stay as supplied. An unfilled
network stays unfilled. Regionless open, branched, self-loop and disconnected
networks without a cached outline are outside this recovery profile: their
segments remain editable and visible fills now produce a warning. Existing
cached-outline fallback remains; its general editable stroke fidelity is not
accepted by this change.

An instance with overridden fills cannot share an inferred network cache entry
with a copy that has no fills. Cache identity includes whether recovery applied.

## Unfilled faces on export

In a fresh synthetic Figma page, ordinary SVG paste and native Paint operations
were exercised through the browser UI. Closed, open, branched, even-odd holes
and nonzero nested contours were exported as PNG and original clipboard
carriers. These new carriers contain explicit regions, including an open
two-segment loop; they do not establish how every legacy regionless topology
must render.

Painting the last filled face off removed both its regions and its layer fill
paints. Exporting a FigCraft vector with no regions must therefore not advertise
active native fill paints. The exporter hides that vector's native fill paints and keeps its original
shape/paints/stroke in the existing FigCraft shape stash.
Reopening in FigCraft restores unused paints without filling the face. The
export report explains this metadata retention. Hidden image paints still load their assets, so an unused image fill also
survives export and reopening.

The tests exercise a shuffled/reversed legacy contour with handles, empty fills,
explicit winding cancellation, ambiguous graphs and an unfilled export/reopen.
The new native controls exercise the full import/render path separately.

## Replay result

21 intact original carriers were imported: 13 whole-image renders and eight
import-only normalization controls. Both previously blank scenes now draw;
maximum native alpha error is 32 and 16 respectively (255 before). This does
not pass native 1:1 acceptance. Two previously missing red sentinel contours
in the curved-composition carrier also recover; its worst alpha error remains
96. The other ten render images are byte-for-byte unchanged.

Across every imported document, the only changes are inferred regions on
eligible closed chains; all existing vertices, segments, paints, hierarchy and
other fields compare equal. The newly exported full native document also
contains four eligible legacy contours on other pages, which recover without
changing its selected-frame image. All eight normalization documents are
unchanged. Warnings remain empty for this supported corpus.

## Actual headless MCP check

59 stdio JSON-RPC requests exercised the CLI's real MCP server, with explicit
read/write roots and isolated preferences. Three original carriers opened,
exported the same pixels as the direct renderer, saved to `.figcraft` and
reopened. Opacity edits, undo/redo and a failing batch restored the original
PNG bytes. Saved documents retained vector layers and their networks, with no
image-layer substitution.

A native topology control also had a face unpainted through
`vector.edit.paint`, exported as `.fig`, reopened and painted back on. Its
original pixels returned. The format regression separately verifies that an
unused image fill retains its asset through the same export/reopen path.
This checked the headless workflow at the M6 checkpoint. Attached desktop
checks are recorded separately below; neither checks multiple agents or the
entire MCP command catalog.

## Uniform blur and zero-paint background activation

The uniform layer/background filter now uses a separable, pixel-integrated
triangular kernel with support equal to radius × device scale. Weighted prefix
sums keep work linear at large radii. Premultiplied colour and alpha remain
floating between the two passes and resolve to bytes once; no gamma, radius or
sample phase was fitted to native images. Layer-blur export bounds use the same
full-radius support: the native 256×128 radius12 control exports at 560×304
at 2×; asking explicitly for the geometric box still crops it to 512×256.
At `d8fdac8`, shadows and glass retained their previous Gaussian approximation.
The subsequent [native shadow correction](native-shadows.md) shares this kernel
for shadows; glass frost remains outside this filter acceptance.

The evidence replay expanded to 36 intact native carriers: 28 original PNG
comparisons and eight import-only controls. Complete imported document JSON is
byte-identical before/after. The renderer never reads the native PNGs.

| Native control | Before maximum error /255 | After maximum error /255 |
| --- | --- | --- |
| Solid rectangle / flattened vector blur12, alpha | 21 | 2 |
| Skewed vector blur12, alpha | 24 | 2 |
| Coloured gradient / alpha seam / unclipped parent blur12, alpha | 21–22 | 2 |
| Background blur12/24 with positive fill, encoded RGB | 16–22 | 2–3 |
| Background blur24, zero-alpha solid fill, complete RGBA | 191 | 0 |

These are residual measurements, not passing parity tolerances. All straight
RGBA differences, including RGB under transparent pixels, are retained in the
local report. Primitive AA, correlated edges/groups/masks and shadow residuals
were unchanged by the uniform-blur arc; the subsequent shadow correction is
recorded separately. The zero-paint correction is bounded to layers with explicit
fully transparent paints and no children or visible stroke; arbitrary paint
coverage masks and no-fill rectangles remain unaccepted. A fresh native
no-fill vector control clears both paints and regions and already renders
exactly; this does not establish the rectangle case. Its white 25% fill was
restored through the UI, and the restored typed nodes exactly match the
retained control. That fresh native PNG differs from the retained native PNG
by up to two RGB levels (alpha identical); cause/build/seed attribution is
unknown. The candidate still differs from the fresh PNG by up to three RGB
levels. Neither native variation nor transparent RGB is discarded as a
parity tolerance.

Progressive blur still interpolates a few uniform levels. Its levels now use
the same filter as uniform blur, preserving flat-ramp and endpoint consistency;
no native progressive interpolation acceptance follows. Mathematical tests
compare prefix sums with direct integrated convolution at fractional, large
and near-zero support, including two-pass premultiplied colour. Tests that
assumed Gaussian blur or monotonic contrast of periodic stripes were corrected:
a triangular kernel has frequency zeros and lobes. Sharp-end tests now sample
outside the ramp rather than a pixel centre already inside it.

SVG has a different Gaussian blur primitive, so blurred layers use the existing
whole-layer raster export path. An embedded PNG is checked byte-for-byte against
the renderer including a shadow; the internal document remains editable, while
the exported SVG blur is an image. Imported SVG Gaussian blur is an approximation
in the native blur model. PDF already uses the renderer for these effects.

## Actual desktop and attached MCP

A fresh desktop build was launched on the user's actual Wayland session with
isolated configuration/cache/data folders and confined automation roots. The
headless and attached stdio MCP servers each opened nine retained/fresh native
scenes, exported the same pixels as the direct renderer, saved/reopened editable
`.figcraft` files, changed opacity, undid/redid it, and rolled back a failing
batch. Attached Undo used the actual UI shortcut route. The blur control's full
export is 560×304, with no fitted crop. Other controls explicitly request their
retained geometric-box crops. Window captures were visually inspected.

Separate authoring checks created a frame, rounded rectangle and text through
MCP, replaced text, inspected geometry, hovered the pointer, saved/reopened and
verified the PNG bytes. No image node replaced those editable layers. Together
the checks exercised all 18 MCP tool entrypoints: 204 attached requests and 166
headless requests. This is bounded protocol/workflow evidence, not acceptance
of all commands, native fonts, concurrent agents, component libraries,
interactive desktop controls or prototypes. The desktop was left displaying
the restored native background-blur control.

## Validation

At `40de825`, `cargo +1.97.1 xtask ci` passed: formatting, workspace clippy,
3223 native unit/integration tests (eight ignored), doctests, English catalog
coverage, layer boundaries, asset attribution, WASM and the plugin workspace.
The first run caught the new import warning missing from the English catalog;
that entry was added and the complete gate rerun. The optional craft-fonts
checkout was absent; a preexisting unknown-clippy-lint warning remains on this
toolchain. No native font fidelity or interactive performance acceptance follows.

For the M7 filter/bounds tree, `cargo +1.97.1 xtask ci` also passed all of
those gates, including WASM and plugin builds. Export regressions verify full
blur support and the explicit geometric-box crop separately. The initial SVG
check expected editable filter metadata after raster export; it now checks the
document stays editable, the SVG reopens as an image and the embedded pixels
equal the renderer. An intermediate CI process ended with SIGTERM (cause
unidentified); the complete gate was rerun successfully after the bounds fix.

## Local evidence and limits

Original archives, clipboard HTML, decoded source, baseline/candidate documents,
PNG comparisons and command logs live under gitignored
`plan/figma/vector-fill-20261026/`. Native PNGs stay under gitignored
`plan/figma/screenshots/vector-fill-20261026/`. No incumbent image or executable
body is a committed asset or renderer input. The replay harness imports intact
carriers; native PNGs are read only by the separate comparison step.

This is a narrow import/export correction. General native antialiasing,
floating composition, general effects, arbitrary region inference, real-world
libraries, complete desktop interactions and full MCP parity remain unaccepted. The
native reader's treatment of FigCraft's metadata is not a guarantee that it
will preserve that metadata after editing and re-saving there.

The M7 uniform-blur evidence is separate, under gitignored
`plan/figma/alpha-desktop-20261026/`: 63 frozen retained inputs, before/after
whole documents, all residuals, native UI acquisitions, failed checks and
actual MCP transcripts. Two additional fresh carriers/PNGs are retained, with
their incumbent PNGs under `plan/figma/screenshots/alpha-desktop-20261026/`.
The final replay imports all 38 carriers without warnings; the 36 retained
document JSON files are byte-identical to their baselines. Primitive AA,
correlated composition, shadows, progressive blur and prototype motion still
need native corrections/acceptance. There is no 1:1 MVP acceptance from this arc.

The subsequent [native shadow controls and correction](native-shadows.md)
separate geometric caster/body/knockout, reduce the rectangular shadow errors
and retain unresolved curve residuals. They do not change the MVP status.

## Correlated solid scenes

The following M7 arc keeps sample correlation through solid paint stacks, groups,
alpha masks and clips in the shared renderer. Two native full-RGBA controls now
match; curved group/mask alpha max96→1. Cubic-grid max16 and 1x direction/context
max32 remain. See [correlated AA](native-correlated-aa.md) for original/fresh
whole-source comparisons, the bounded fallback profile and remaining failures.
