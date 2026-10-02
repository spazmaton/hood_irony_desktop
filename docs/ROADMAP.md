# Roadmap

## v0.1 — Base pet

- [x] Project skeleton: Cargo, eframe, borderless always-on-top window
- [x] Video decoding and looping through ffmpeg-sidecar
- [x] Walk along the bottom edge and turn around at screen boundaries
- [x] Random sounds through rodio
- [x] Dragging and gravity-based falling
- [x] Tray menu: Pause / Settings / Quit
- [x] Settings window with a live preview
- [x] `config.toml` for size, speed, sound interval, and volume
- [x] Release build

## v0.2 — Polish

- [x] Green-screen removal (chroma key)
- [x] Time-based video playback
- [x] Throw inertia, bounces, and landing sounds
- [x] Sync walking animation to movement speed
- [x] Show playback frame rate in Settings
- [x] Release build
- [ ] Volume control in the tray menu
- [ ] Remember the pet's position between runs
- [ ] Multi-monitor support

## v0.3 — Walk on windows

- [ ] Walk on window title bars and the taskbar using WinAPI `EnumWindows`
- [ ] Fall or jump when a window moves or closes
- [ ] Climb along window edges

## Future ideas

- [ ] Mouse-click reactions, effects, and sounds
- [ ] Alternate video costumes selectable from the tray menu
- [ ] Launch on Windows startup

## Status

Current version: v0.2, with synchronized video timing, throw physics, and
walking-animation sync.
