mod app;
mod config;
mod pet;
mod sound;
mod tray;
mod video;

use anyhow::{Context as _, Result};

fn main() -> Result<()> {
    let cfg = config::Config::load();

    let video_dir = config::assets_dir().join("video");
    let video_path = video_dir.join("pet.mp4");
    anyhow::ensure!(
        video_path.exists(),
        "Video not found: {}\nPut an mp4 at assets/video/pet.mp4",
        video_path.display()
    );

    // ffmpeg must be available (auto-downloaded on first run if missing)
    if !video::ffmpeg_available() {
        println!("ffmpeg not found, downloading (one time, ~80 MB)...");
        ffmpeg_sidecar::download::auto_download().context("failed to download ffmpeg")?;
    }

    let sounds_dir = config::assets_dir().join("sounds");
    let sound = sound::SoundPlayer::new(&sounds_dir, &cfg.audio_device);
    if !sound.has_sounds() {
        eprintln!("No sounds in assets/sounds/ — the pet will walk silently");
    }

    // Tray (created on the main thread before eframe)
    let tray = load_tray();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([cfg.pet_width, cfg.pet_width * 16.0 / 9.0])
            .with_always_on_top()
            .with_decorations(false)
            .with_resizable(false)
            .with_taskbar(false)
            .with_transparent(cfg.chroma_key)
            .with_active(false),
        ..Default::default()
    };

    eframe::run_native(
        "hood-irony-desktop",
        options,
        Box::new(move |_cc| Ok(Box::new(app::App::new(cfg, sound, tray)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe exited with an error: {e}"))?;

    Ok(())
}

fn load_tray() -> Option<tray::Tray> {
    let icon_path = config::assets_dir().join("icon.png");
    let icon_rgba = std::fs::read(&icon_path)
        .ok()
        .and_then(|bytes| {
            image::load_from_memory(&bytes)
                .ok()
                .map(|img| img.to_rgba8())
        })
        .map(|img| {
            let (w, h) = (img.width(), img.height());
            (img.into_raw(), w, h)
        });

    match icon_rgba {
        Some((rgba, w, h)) => match tray::Tray::new(rgba, (w, h)) {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("Tray unavailable: {e:#}");
                None
            }
        },
        None => {
            eprintln!("Missing tray icon {:?}", icon_path);
            None
        }
    }
}
