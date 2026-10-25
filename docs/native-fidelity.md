# Native fidelity: bounded controls

2026-10-26, M6. This fork keeps the shared FigCraft document/command/renderer
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
This checks the headless workflow, not the attached desktop, multiple agents,
or the entire MCP command catalog.

## Validation

`cargo +1.97.1 xtask ci` passed on the final tree: formatting, workspace clippy,
3223 native unit/integration tests (eight ignored), doctests, English catalog
coverage, layer boundaries, asset attribution, WASM and the plugin workspace.
The first run caught the new import warning missing from the English catalog;
that entry was added and the complete gate rerun. The optional craft-fonts
checkout was absent; a preexisting unknown-clippy-lint warning remains on this
toolchain. No native font fidelity or interactive performance acceptance follows.

## Local evidence and limits

Original archives, clipboard HTML, decoded source, baseline/candidate documents,
PNG comparisons and command logs live under gitignored
`plan/figma/vector-fill-20261026/`. Native PNGs stay under gitignored
`plan/figma/screenshots/vector-fill-20261026/`. No incumbent image or executable
body is a committed asset or renderer input. The replay harness imports intact
carriers; native PNGs are read only by the separate comparison step.

This is a narrow import/export correction. General native antialiasing,
floating composition, blur kernels, arbitrary region inference, real-world
libraries, desktop interactions and full MCP parity remain unaccepted. The
native reader's treatment of FigCraft's metadata is not a guarantee that it
will preserve that metadata after editing and re-saving there.
