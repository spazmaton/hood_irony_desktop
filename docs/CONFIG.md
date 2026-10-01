# config.toml

Created automatically on first run next to the exe (in the project root in
debug builds). You can delete it — it will be recreated with defaults.
Most values can also be edited live in the Settings window.

```toml
# Pet width on screen in px. Height follows the video aspect.
# Default 180 (for a 720x1280 video -> 180x320).
pet_width = 180.0

# Walk speed in px/s.
walk_speed = 45.0

# Random sound interval, seconds.
sound_interval_min = 30.0
sound_interval_max = 120.0

# Sound volume 0.0..=1.0
volume = 0.8

# Video playback speed (1.0 = original, <1 slower).
video_speed = 0.5

# Cut out the green screen (shimeji mode).
# false = green screen background, like the meme.
chroma_key = false

# Keep animating while idle (true) or only while walking (false).
animate_when_idle = false

# Play the video's own (looped) audio track.
video_audio = false

# Freeze while a sound is playing.
stop_on_sound = true

# Pause (kept in sync with the tray checkbox).
paused = false
```

## Behavior

- The file is read at startup. Changes made in the Settings window are applied
  immediately; changes made by hand are applied on the next launch.
- Broken values fall back to defaults; the app does not crash.
- `pet_width` resizes the window and restarts the ffmpeg decoder with the new scale.
