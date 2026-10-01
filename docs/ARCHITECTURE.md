# Architecture

## Overview

A single-threaded `eframe`/`egui` app. The video is decoded on a background
thread, sound goes through `rodio`, and the tray uses `tray-icon`.

```
┌────────────────────────────────────────────────┐
│ main.rs                                        │
│  ├─ config (config.toml, serde)                │
│  ├─ eframe window: borderless, always_on_top,  │
│  │  no taskbar, size from config               │
│  ├─ tray-icon (menu: Pause / Settings / Quit)  │
│  └─ App (egui)                                 │
│      ├─ VideoPlayer  — frames from ffmpeg      │
│      ├─ Pet          — behavior state machine  │
│      ├─ SoundPlayer  — rodio                   │
│      └─ settings UI  — separate viewport       │
└────────────────────────────────────────────────┘
```

## Crates

| Crate | Purpose |
|---|---|
| `eframe` / `egui` | window + drawing frames as a texture |
| `ffmpeg-sidecar` | decode mp4: spawns ffmpeg, reads rawvideo from stdout |
| `rodio` | mp3/wav playback |
| `tray-icon` | tray icon + menu |
| `image` | load the tray icon |
| `serde` + `toml` | config |
| `rand` | random sound / interval / pauses |
| `windows-sys` | DWM corners, `WS_EX_NOACTIVATE` |
| `raw-window-handle` | get the HWND from the winit window |
| `anyhow` | errors |

## Video pipeline

1. On start, `ffmpeg -stream_loop -1 -i pet.mp4 -an -vf scale=W:H -f rawvideo -pix_fmt rgb24 -` is spawned.
2. A background thread reads stdout and pushes frames into a **bounded blocking
   buffer** (12 frames). When it is full the producer blocks, which throttles
   ffmpeg through pipe backpressure — so frames are always sequential and none
   are dropped. (An unbounded/dropping buffer makes playback run at ffmpeg's
   decode speed instead of the real frame rate.)
3. The egui update pulls frames by real elapsed time using a playback
   accumulator (so uneven repaints do not cause stutter) and draws them as a texture.
4. Memory stays low: frames are not accumulated, only a small look-ahead buffer.

The ffmpeg output size is the pet's window size in physical pixels (default
180×320, aspect 720×1280). Changing the size restarts the decoder.

The video's audio track is not played (intentionally); `video_audio` in the
config is reserved for the future.

## Pet state machine

```
        ┌──────────────────────────────────┐
        │                                  │
        ▼                                  │
     ┌──────┐   rand(1..4s)   ┌──────────┐ │
     │ Idle ├────────────────►│  Walk    ├─┘
     └──────┘                 └────┬─────┘
        ▲  ▲                      │ edge → flip
        │  │ drag start           │ rand(30..120s)
        │  ▼                      ▼
        │ ┌──────────┐        ┌──────────┐
        └─┤ Dragged  │        │  Sound   ├──── sound from sounds/
   drop  └────┬─────┘        └──────────┘
   ┌──────────┘ release
   │
   │   gravity → fall to the bottom edge
   └────────────────────────► Idle
```

- **Idle** — stands still (video paused)
- **Walk** — the window moves along X; at the screen edge it flips direction
  and mirrors the video (egui UV flip)
- **Sound** — briefly freezes (optional), plays a random file
- **Dragged** — the window follows the cursor; drag velocity is tracked
- Release: the pet keeps the drag velocity as throw inertia, flies, bounces
  off walls and the floor, and can settle (playing a `hit_*` sound on contact)

## Dragging (the important bit)

egui reports the cursor position **inside the window**, not on the screen.
With the window's top-left at `pos`, the global cursor is `pos + local`.
Keeping the grab offset constant:

```
pos = pos + local - grab
```

This avoids the feedback loop you get from treating the local pointer as global.

## Window behavior

- Borderless, always-on-top, no taskbar, no Alt+Tab (`with_taskbar(false)`).
- Never takes focus (`with_active(false)` + `WS_EX_NOACTIVATE`).
- Square corners via `DwmSetWindowAttribute` (`DWMWA_WINDOW_CORNER_PREFERENCE`).
- `clear_color` returns transparent only in chroma-key mode, otherwise opaque.

## Tray

- `tray-icon` on the main thread, menu: Pause / Settings / Quit.
- Events are polled each frame (`MenuEvent::receiver().try_recv()`).

## File layout

```
src/
  main.rs        // startup, config, window, tray
  app.rs         // egui App: rendering, timing, settings UI
  pet.rs         // behavior state machine
  video.rs       // ffmpeg decoder + frame buffer
  sound.rs       // rodio: sound list, random playback
  config.rs      // load/save config.toml
  tray.rs        // tray icon and menu
assets/
  video/pet.mp4
  sounds/*.mp3|wav
  icon.png
```
