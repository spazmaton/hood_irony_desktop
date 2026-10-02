# config.toml

Created automatically on first run next to the executable (in the project root
for debug builds). If deleted, it will be recreated with default values. Most
settings can also be changed in the Settings window.

```toml
# Pet width in pixels. Height follows the video's aspect ratio.
# Default: 180 (a 720×1280 video produces a 180×320 window).
pet_width = 180.0

# Walking speed in pixels per second.
walk_speed = 45.0

# Interval between random sounds, in seconds.
sound_interval_min = 30.0
sound_interval_max = 120.0

# Sound volume: 0.0 to 1.0.
volume = 0.8

# Video playback speed: 1.0 is normal, values below 1.0 are slower.
video_speed = 0.5

# Derive video playback speed from walking speed.
animation_sync = false

# Walking speed in pixels per second that corresponds to 1× video speed.
sync_walk_at_1x = 45.0

# Remove the green screen.
# false keeps the original green background.
chroma_key = false

# Keep animating while idle; false freezes the video when the pet is idle.
animate_when_idle = false

# Reserved: the video's audio track is not currently played.
video_audio = false

# Freeze the pet while a random sound is playing.
stop_on_sound = true

# Pause the pet (kept in sync with the tray menu).
paused = false
```

## Applying settings

- The file is read at startup. Changes made in the Settings window apply
  immediately; manual edits take effect the next time the app starts.
- Invalid values are replaced with safe values or defaults.
- Changing `pet_width` resizes the window and restarts the FFmpeg decoder.
