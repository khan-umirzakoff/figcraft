# Native attached MCP wake (M7)

Attached state queries and edits now wake native app logic even when a Wayland
compositor withholds redraw callbacks. The command engine, document and renderer
are unchanged by this correction.

## Cause and correction

`egui::Context::request_repaint` coalesces requests behind an outstanding redraw.
On the observed Wayland route a covered window reports visibility as unknown;
a pending compositor frame therefore prevented the earlier logic dispatcher
from running. Moving dispatch to `App::logic` alone did not fix this.

The desktop server now installs a thread-safe native logic waker. Every admitted
request posts native events; requests queued during startup are also woken.
The events enter the framework's existing root logic-only path on the UI thread,
including its input retention, viewport commands and error propagation. Two
ticks let native Close commands be observed on the following tick. There is no
polling loop or automatic focus change. Normal redraws retain their original
path, and the original `eframe::run_native` startup/error handling remains.

The exact `eframe 0.36.2` source is pinned locally, with six Rust files changed.
[Patch, provenance and removal criteria](../vendor/eframe/FIGCRAFT_PATCH.md)
include the upstream checksum, original-file inventory, licenses and complete
diff. Its branded fallback icon is omitted; FigCraft supplies its own icon.
WGPU and Glow compile; the actual runtime checks below use WGPU on Wayland.

## Observed verification

Local receipts stay in gitignored `plan/figma/control-wake-20261026/` and
`plan/figma/correlated-aa-20261026/`. The final executable's SHA-256 is
`d1d50c427136c6866160709982ea2322d02ec2d9694f73663854f7ccc5e3a2fd`.

- Full `cargo +1.97.1 xtask ci` passes, including native tests, WASM and plugin
  gates. Socket tests cover pending repaint coalescing and pre-install requests.
  A separate local package also type-checks the optional Glow path.
- Final attached MCP passes 28 scenes / 675 requests and two additional complete
  native cubic files / 50 requests. Whole exports match the direct renderer;
  save/reopen, edits, keyboard Undo/Redo and failed-batch rollback are checked.
- MCP authoring passes 23 requests across the 18 advertised entry points, creating
  editable frame, rectangle and text layers. Actual window captures were read.
- The native inspector field changes X from 1016.375 to 1016.5; raw window
  keyboard Undo/Redo restores both exact stored coordinates and PNGs (39 calls).
- With no new compositor frames, 75 seconds of uninterrupted idle is followed
  by 47 attached MCP requests on two 452-layer / 32-page native files. The frame
  counter stays 1939 throughout. Open, inspect, export, save/reopen, editing,
  keyboard Undo/Redo, rollback and node screenshots work. Read/write confinement
  still rejects the synthetic out-of-root sentinel. Idle process CPU is 0.28 s
  over 75.001 s; the slowest request is 0.302 s. These are bounded observations,
  not platform-wide CPU or latency promises.
- One `app.quit` request then exits the covered process within 0.252 s, without
  another state query. The earlier one-tick Close failure is retained.

A whole-window screenshot still requires a real native frame. In the unknown
Wayland visibility state it can time out while state requests continue to work;
`window.focus` did not restore that covered window in this observation. This
failed capture is retained. A freshly foregrounded final window passes the
visible capture and field-edit checks. The first authoring retry also retained
a test-script fixture-path error; correcting that path changed no product code.
Windows, macOS, FreeBSD, Glow runtime and multiple native windows were not
revalidated here.

## Fidelity remains open

The [correlated AA evidence](native-correlated-aa.md) and its native pixel
residuals are unchanged. A further diagnostic quantizes transformed cubic
control points to f32 once, then solves the curves in f64. It leaves the whole
historical cubic grid's alpha output byte-identical and max error at16; it is
not a renderer correction. No per-pixel adjustment or incumbent implementation
body was used. Cubic AA, mixed unsupported paint/effect scenes, fonts and the
representative library/prototype corpus remain unaccepted. This wake fix does
not establish a 1:1 MVP.
