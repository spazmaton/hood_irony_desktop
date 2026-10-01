# Roadmap

## v0.1 — base pet

- [x] Project skeleton: cargo, eframe, borderless always-on-top window
- [x] Video decoder via ffmpeg-sidecar, looped playback
- [x] Walk along the bottom edge, turn around at edges (mirroring)
- [x] Random sounds via rodio
- [x] Drag with the mouse + gravity fall
- [x] Tray icon: Pause / Settings / Quit
- [x] Settings window with a live preview
- [x] `config.toml` (size, speeds, sound interval, volume)
- [x] Accurate video timing (blocking buffer, no dropped frames)
- [x] Throw physics (drag inertia, bounce, landing sound)
- [x] Auto-sync of walk animation to walk speed
- [x] Playback fps readout
- [ ] Release build

## v0.2 — polish

- [x] Chroma-key (green screen) cutout option
- [ ] Volume control from the tray
- [ ] Remember start position between runs
- [ ] Multi-monitor support

## v0.3 — real shimeji

- [ ] Walk on top of window title bars / the taskbar (WinAPI `EnumWindows`)
- [ ] Fall / jump off when a window moves or closes
- [ ] Climb window edges

## v??? — ideas

- [ ] Click reactions (effects / sounds)
- [ ] Multiple video costumes (switch from the tray)
- [ ] Launch on Windows startup

## Status

Current: v0.2 — video timing fixed, throw physics and animation sync added.

