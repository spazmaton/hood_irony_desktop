# Pitfalls and fixes (Windows + egui pet)

Notes on non-obvious things, already handled or likely to come up.

## Window

- **Always-on-top + borderless**: `ViewportBuilder::with_always_on_top()`,
  `with_decorations(false)`, `with_taskbar(false)`.
  `with_taskbar(false)` removes the window from both the taskbar and Alt+Tab.
- **Never steal focus**: `with_active(false)` plus `WS_EX_NOACTIVATE` via Win32.
- **Square corners**: Windows 11 rounds windows; disable with
  `DwmSetWindowAttribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND)`.
- **Transparency**: a transparent window shows the `clear_color`. Return
  `[0,0,0,0]` from `App::clear_color` when you want a truly transparent window,
  otherwise eframe fills it with an opaque color.
- egui repaints on request: keep calling
  `request_repaint_after(Duration::from_secs_f64(1.0 / 60.0))`.

## Smart App Control

- Windows Smart App Control (SAC) blocks **all** unsigned executables. Cargo
  generates many temporary build-script exes, so builds fail with
  `os error 4551`. There are no exclusions: SAC must be turned off.
  Malwarebytes and similar real-time scanners behave the same way — add the
  project and cargo folders to their exclusions.

## ffmpeg-sidecar

- On first run it downloads ffmpeg (~80 MB) into its cache unless a system
  ffmpeg is found. `resolve_ffmpeg()` prefers the sidecar copy and falls back
  to `ffmpeg` from PATH.
- There is no `ffprobe` in the sidecar package, so `probe()` parses the stderr
  of `ffmpeg -i <file>` (note the `-i`, otherwise ffmpeg treats the file as an
  *output* and reports "no video stream").
- Read rawvideo from stdout in a separate thread with a **bounded blocking**
  buffer; never read in the egui update.
- **Do not use a drop-oldest buffer.** ffmpeg without `-re` decodes far faster
  than real time; if the buffer discards the oldest frames, the frames you pull
  are always the newest, so the perceived speed becomes ffmpeg's *decode*
  speed. Block the producer instead (backpressure) so frames stay in order.
- `-stream_loop -1` loops the input.

## Window position

- Move the pet with `ViewportCommand::OuterPosition` from the egui update.
- `OuterPosition` takes **logical** points (egui multiplies by `pixels_per_point`).
- Only send the command when the position actually changed (> 0.4 pt);
  redundant window moves make the content stutter.

## Dragging

- egui pointer positions are **local to the window**. Treating them as global
  makes the pet jump around. Use the incremental formula
  `pos = pos + local - grab` (see `docs/ARCHITECTURE.md`).

## Tray

- `tray-icon` is created on the main thread before `eframe::run_native`.
- Poll events with `MenuEvent::receiver().try_recv()`; do not block the egui loop.

## Sound

- `rodio` 0.22 uses `MixerDeviceSink` + `Player` (`OutputStream`/`Sink` were removed).
- Keep one `Player`: on a new sound, `clear()` then `append()`.
- With `rand` 0.10, `random_range`/`random_bool` live in the `RngExt` trait.

## Build

- `cargo build --release` produces one exe. `ffmpeg-sidecar` downloads ffmpeg
  into its own cache on first run; `resolve_ffmpeg()` uses that cache first and
  then falls back to `ffmpeg` on `PATH`. There is no lookup next to the exe, so
  for an offline distribution either ship ffmpeg on `PATH` or extend
  `resolve_ffmpeg()` to check the exe's folder.
- The exe icon can be set with `winres` in `build.rs`.
