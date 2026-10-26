# FigCraft native logic wake

Upstream: crates.io `eframe 0.36.2`, crate checksum
`55ad41fac6e3149abb4866e93df37dbd33baa95db525c3368c944584e95a5dc2`,
[egui commit 49682f8](https://github.com/emilk/egui/tree/49682f8baa058bf49e011035cfbd6e825f88a5ef/crates/eframe).
The upstream MIT and Apache-2.0 licenses are included. Original package files
are inventoried in `UPSTREAM-SHA256.json`; only six Rust files differ.
`FIGCRAFT.patch` is the complete Rust diff against that exact package.
The branded default icon in `OMITTED.json` is not redistributed; the framework
fallback is empty and FigCraft supplies its own original icon.

On Wayland, an apparently visible window can receive no compositor frame
callbacks while covered. A queued redraw then need not reach `App::logic`.
egui also coalesces subsequent requests behind that pending redraw. Attached
MCP state queries and edits must still run on the UI thread.

The native creation context now exposes a thread-safe logic waker. Its event
runs the existing root logic-only integration path immediately, including the
existing input preservation, viewport-command dispatch and error handling.
Two logic ticks let the integration observe a native viewport command (such as
Close) on its following tick. Requests queued before the waker is installed are
woken when the app creation context becomes ready.
Normal redraw calls keep their original path. WGPU and Glow both handle the
native event; browser code is unchanged. It adds no polling or focus request.

Remove this patch when the pinned upstream supports a compositor-independent
root logic wake. Do not reformat the upstream package with the parent project's
rustfmt settings; keep the small patch reviewable and the remaining files exact.
