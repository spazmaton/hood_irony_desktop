//! egui application: the pet window, video rendering and interaction.
//! eframe 0.36: the main method is `App::ui`, the context comes from `ui.ctx()`.

use crate::config::Config;
use crate::pet::Pet;
use crate::sound::SoundPlayer;
use crate::tray::{Tray, TrayEvent};
use crate::video::VideoPlayer;
use std::time::{Duration, Instant};

pub fn settings_viewport_id() -> egui::ViewportId {
    egui::ViewportId(egui::Id::new("settings"))
}

pub struct App {
    pub cfg: Config,
    pub video: Option<VideoPlayer>,
    pub texture: Option<egui::TextureHandle>,
    pub pet: Option<Pet>,
    pub sound: SoundPlayer,
    pub tray: Option<Tray>,
    pub video_inited: bool,
    pub applied_chroma_key: bool,
    pub video_needs_refresh: bool,
    pub hwnd: Option<isize>,
    pub settings_open: bool,
    pub applied_width: f32,
    pub aspect: f32,
    pub ppp: f32,
    pub styled: bool,
    /// Unsaved settings changes
    pub dirty: bool,
    /// Playback accumulator for the video (seconds of source material)
    pub frame_accum: f64,
    /// Last outer position sent to the OS (avoids redundant window moves)
    pub last_pos_sent: Option<egui::Pos2>,
    /// Measured playback frame rate (for the settings readout)
    pub measured_fps: f32,
    fps_count: u32,
    fps_window: Instant,
    /// Folder with sounds, rescanned periodically
    sounds_dir: std::path::PathBuf,
    last_sound_scan: Instant,
    /// Available audio output devices
    devices: Vec<String>,
}

impl App {
    pub fn new(cfg: Config, sound: SoundPlayer, tray: Option<Tray>) -> Self {
        let applied_chroma_key = cfg.chroma_key;
        Self {
            cfg,
            video: None,
            texture: None,
            pet: None,
            sound,
            tray,
            video_inited: false,
            applied_chroma_key,
            video_needs_refresh: false,
            hwnd: None,
            settings_open: false,
            applied_width: 0.0,
            aspect: 16.0 / 9.0,
            ppp: 1.0,
            styled: false,
            dirty: false,
            frame_accum: 0.0,
            last_pos_sent: None,
            measured_fps: 0.0,
            fps_count: 0,
            fps_window: Instant::now(),
            sounds_dir: crate::config::assets_dir().join("sounds"),
            last_sound_scan: Instant::now(),
            devices: crate::sound::list_output_devices(),
        }
    }

    fn ensure_video(&mut self, ctx: &egui::Context) {
        if self.video_inited {
            return;
        }
        self.video_inited = true;

        let video_path = crate::config::assets_dir().join("video").join("pet.mp4");
        if !video_path.exists() {
            eprintln!("Missing file: {:?}", video_path);
            return;
        }

        self.ppp = ctx.pixels_per_point();
        match self.build_decoder(&video_path) {
            Ok(player) => {
                self.aspect = player.info.src_height as f32 / player.info.src_width as f32;
                let logical_w = self.cfg.pet_width;
                let logical_h = logical_w * self.aspect;
                let screen = ctx
                    .input(|i| i.viewport().monitor_size)
                    .unwrap_or(egui::Vec2::new(1920.0, 1080.0));
                self.pet = Some(Pet::new(
                    egui::Vec2::new(logical_w, logical_h),
                    screen,
                    &self.cfg,
                ));
                self.video = Some(player);
                self.applied_width = logical_w;
                self.applied_chroma_key = self.cfg.chroma_key;
                self.video_needs_refresh = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                    logical_w, logical_h,
                )));
            }
            Err(e) => eprintln!("Failed to open video: {e:#}"),
        }
    }

    fn build_decoder(&self, video_path: &std::path::Path) -> Result<VideoPlayer, anyhow::Error> {
        let phys_w = (self.cfg.pet_width * self.ppp).round().max(2.0);
        VideoPlayer::open(video_path, phys_w as u32, self.cfg.chroma_key)
    }

    fn poll_tray(&mut self, ctx: &egui::Context) {
        let Some(tray) = &self.tray else { return };
        match tray.poll_events() {
            Some(TrayEvent::Settings) => self.settings_open = true,
            Some(TrayEvent::PauseToggle) => {
                self.cfg.paused = !self.cfg.paused;
                tray.set_checked(self.cfg.paused);
                let _ = self.cfg.save();
            }
            Some(TrayEvent::Quit) => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            None => {}
        }
    }

    /// Playback speed actually used, honoring auto-sync with the walk speed.
    fn effective_speed(&self) -> f32 {
        let s = if self.cfg.animation_sync {
            self.cfg.walk_speed / self.cfg.sync_walk_at_1x.max(1.0)
        } else {
            self.cfg.video_speed
        };
        s.clamp(0.05, 3.0)
    }

    /// Pull video frames according to real elapsed time.
    /// Uses an accumulator so playback stays in sync even if repaints are uneven.
    fn update_video_frame(&mut self, ctx: &egui::Context, dt: f64) {
        let animate = !self.cfg.paused
            && (self.cfg.animate_when_idle
                || self.pet.as_ref().map(|p| p.animating()).unwrap_or(true));
        let needs_refresh = self.video_needs_refresh;

        let Some(video) = &self.video else { return };
        if !animate && !needs_refresh {
            self.frame_accum = 0.0;
            return;
        }

        let frame_interval = video.frame_interval().as_secs_f64();
        let speed = self.effective_speed() as f64;
        self.frame_accum += dt * speed;

        let mut advance = 0u32;
        while self.frame_accum >= frame_interval {
            self.frame_accum -= frame_interval;
            advance += 1;
            if advance >= 6 {
                self.frame_accum = 0.0; // dropped too far behind, resync
                break;
            }
        }
        if needs_refresh {
            advance = advance.max(1);
        }
        if advance == 0 {
            return;
        }

        // The producer is throttled by backpressure, so frames arrive in order
        // and we simply take `advance` of them.
        let mut last = None;
        for _ in 0..advance {
            match video.next_frame(Duration::ZERO) {
                Some(f) => last = Some(f),
                None => break,
            }
        }
        if let Some(bytes) = last {
            self.video_needs_refresh = false;
            self.fps_count += advance;
            let img = egui::ColorImage::from_rgba_unmultiplied(
                [video.width as usize, video.height as usize],
                &bytes,
            );
            if let Some(tex) = &mut self.texture {
                tex.set(img, egui::TextureOptions::NEAREST);
            } else {
                self.texture = Some(ctx.load_texture("pet", img, egui::TextureOptions::NEAREST));
            }
        }

        // Refresh the measured fps roughly twice a second
        let elapsed = self.fps_window.elapsed().as_secs_f32();
        if elapsed >= 0.5 {
            self.measured_fps = self.fps_count as f32 / elapsed;
            self.fps_count = 0;
            self.fps_window = Instant::now();
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if !self.styled {
            self.styled = true;
            setup_style(&ctx);
        }

        self.poll_tray(&ctx);
        self.ensure_video(&ctx);
        self.grab_hwnd(frame);

        // Pick up sounds dropped into the folder while running
        if self.last_sound_scan.elapsed().as_secs_f32() >= 2.0 {
            self.last_sound_scan = Instant::now();
            let dir = self.sounds_dir.clone();
            self.sound.refresh(&dir);
        }

        let dt = ctx.input(|i| i.stable_dt) as f64;
        let monitor = ctx
            .input(|i| i.viewport().monitor_size)
            .unwrap_or(egui::Vec2::new(1920.0, 1080.0));

        self.update_video_frame(&ctx, dt);

        // The root Ui has no background of its own; allocate the full area.
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());
        let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

        // --- Dragging ---
        if response.drag_started() {
            if let Some(pet) = &mut self.pet {
                let local = ctx.input(|i| i.pointer.latest_pos().unwrap_or_default());
                pet.begin_drag(egui::Vec2::new(local.x, local.y));
            }
            self.last_pos_sent = None;
        }
        if response.dragged() {
            if let Some(pet) = &mut self.pet {
                let local = ctx.input(|i| i.pointer.latest_pos().unwrap_or_default());
                pet.drag_update(egui::Vec2::new(local.x, local.y), dt as f32);
            }
        }
        if response.drag_stopped() {
            if let Some(pet) = &mut self.pet {
                pet.release();
            }
        }

        // --- Logic ---
        if let Some(pet) = &mut self.pet {
            pet.screen = monitor;
            let pu = pet.update(dt as f32, &self.cfg);
            if pu.random_sound && self.sound.has_sounds() {
                let dur = self.sound.play_random(self.cfg.volume);
                pet.play_sound(dur, &self.cfg);
            }
            if pu.landed {
                self.sound.play_hit(self.cfg.volume);
            }

            // Move the window only when it actually moved (fewer OS window ops = smoother)
            let p = egui::pos2(pet.pos.x, pet.pos.y);
            let needs_move = match self.last_pos_sent {
                Some(prev) => (p - prev).length() > 0.4,
                None => true,
            };
            if needs_move {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(p));
                self.last_pos_sent = Some(p);
            }
        }

        // --- Paint ---
        if let (Some(tex), Some(pet)) = (&self.texture, &self.pet) {
            // The source video walks left, so mirror when moving right
            let uv = if pet.facing() > 0.0 {
                egui::Rect::from_min_max(egui::pos2(1.0, 0.0), egui::pos2(0.0, 1.0))
            } else {
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))
            };
            ui.painter().image(tex.id(), rect, uv, egui::Color32::WHITE);
        }

        self.show_settings(&ctx);
        self.apply_video_options_if_changed(&ctx);

        ctx.request_repaint_after(Duration::from_secs_f64(1.0 / 60.0));
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Transparent background only in chroma-key mode
        if self.cfg.chroma_key {
            [0.0, 0.0, 0.0, 0.0]
        } else {
            [0.0, 0.0, 0.0, 1.0]
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.cfg.save();
    }
}

impl App {
    fn grab_hwnd(&mut self, frame: &mut eframe::Frame) {
        if self.hwnd.is_some() {
            return;
        }
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let Some(window) = frame.winit_window() else {
            return;
        };
        let Ok(handle) = window.window_handle() else {
            return;
        };
        let RawWindowHandle::Win32(h) = handle.as_raw() else {
            return;
        };
        let hwnd = h.hwnd.get() as isize;
        self.hwnd = Some(hwnd);

        unsafe {
            // Square corners (Windows 11 rounds windows otherwise)
            let preference: i32 = 1; // DWMWCP_DONOTROUND
            windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute(
                hwnd as *mut core::ffi::c_void,
                33, // DWMWA_WINDOW_CORNER_PREFERENCE
                &preference as *const i32 as *const core::ffi::c_void,
                std::mem::size_of::<i32>() as u32,
            );

            // Never steal focus from the active window
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_NOACTIVATE,
            };
            let ex = GetWindowLongW(hwnd as *mut core::ffi::c_void, GWL_EXSTYLE);
            SetWindowLongW(
                hwnd as *mut core::ffi::c_void,
                GWL_EXSTYLE,
                ex | WS_EX_NOACTIVATE as i32,
            );
        }
    }

    fn show_settings(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }

        // Closed via the native title bar
        let closed = ctx.input(|i| {
            i.raw
                .viewports
                .get(&settings_viewport_id())
                .map(|v| {
                    v.events
                        .iter()
                        .any(|e| matches!(e, egui::ViewportEvent::Close))
                })
                .unwrap_or(false)
        });
        if closed {
            self.settings_open = false;
            let _ = self.cfg.save();
            return;
        }

        let ctx2 = ctx.clone();
        let builder = egui::ViewportBuilder::default()
            .with_title("Hood Irony — Settings")
            .with_inner_size([420.0, 520.0])
            .with_min_inner_size([360.0, 400.0])
            .with_resizable(true)
            .with_always_on_top();

        ctx.show_viewport_immediate(settings_viewport_id(), builder, |ui, _class| {
            self.settings_ui(ui, &ctx2);
        });
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::Frame::NONE
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                // --- Header with preview ---
                ui.horizontal(|ui| {
                    self.preview(ui, 74.0);
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Hood Irony").heading().strong());
                        ui.label(
                            egui::RichText::new("A small desktop companion")
                                .color(egui::Color32::from_gray(150)),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("Right-click the tray icon to open the menu")
                                .small()
                                .color(egui::Color32::from_gray(120)),
                        );
                    });
                });

                ui.add_space(8.0);

                // --- Pet ---
                section(ui, "Pet");
                egui::Grid::new("g_pet")
                    .num_columns(2)
                    .spacing([12.0, 10.0])
                    .show(ui, |ui| {
                        row_check(ui, "Pause", &mut self.cfg.paused, &mut self.dirty);
                        row_slider(
                            ui,
                            "Width, px",
                            &mut self.cfg.pet_width,
                            100.0..=400.0,
                            "",
                            &mut self.dirty,
                        );
                        row_slider(
                            ui,
                            "Walking speed",
                            &mut self.cfg.walk_speed,
                            10.0..=200.0,
                            " px/s",
                            &mut self.dirty,
                        );
                    });

                // --- Animation ---
                section(ui, "Animation");
                let speed = self.effective_speed();
                egui::Grid::new("g_anim")
                    .num_columns(2)
                    .spacing([12.0, 10.0])
                    .show(ui, |ui| {
                        let sync = &mut self.cfg.animation_sync;
                        row_check(ui, "Sync animation to walking speed", sync, &mut self.dirty);

                        ui.label("Video speed");
                        let enabled = !self.cfg.animation_sync;
                        if ui
                            .add_enabled(
                                enabled,
                                egui::Slider::new(&mut self.cfg.video_speed, 0.1..=1.5).suffix("×"),
                            )
                            .changed()
                        {
                            self.dirty = true;
                        }
                        ui.end_row();

                        if self.cfg.animation_sync {
                            ui.label("Walking speed at 1×");
                            if ui
                                .add(
                                    egui::DragValue::new(&mut self.cfg.sync_walk_at_1x)
                                        .range(5.0..=300.0)
                                        .suffix(" px/s")
                                        .speed(1.0),
                                )
                                .changed()
                            {
                                self.dirty = true;
                            }
                            ui.end_row();
                        }

                        row_check(
                            ui,
                            "Animate while idle",
                            &mut self.cfg.animate_when_idle,
                            &mut self.dirty,
                        );

                        ui.label("Frames per second");
                        let target = self
                            .video
                            .as_ref()
                            .map(|v| v.info.fps * speed)
                            .unwrap_or(0.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "{:.1} (target: {:.1})",
                                self.measured_fps, target
                            ))
                            .color(egui::Color32::from_gray(150)),
                        );
                        ui.end_row();
                    });

                // --- Sound ---
                section(ui, "Sound");
                let mut device_changed = false;
                let devices = self.devices.clone();
                egui::Grid::new("g_sound")
                    .num_columns(2)
                    .spacing([12.0, 10.0])
                    .show(ui, |ui| {
                        row_slider(
                            ui,
                            "Volume",
                            &mut self.cfg.volume,
                            0.0..=1.0,
                            "",
                            &mut self.dirty,
                        );
                        ui.label("Output device");
                        let selected = if self.cfg.audio_device.is_empty() {
                            "System default".to_owned()
                        } else {
                            self.cfg.audio_device.clone()
                        };
                        egui::ComboBox::from_id_salt("audio_device")
                            .selected_text(selected)
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_label(
                                        self.cfg.audio_device.is_empty(),
                                        "System default",
                                    )
                                    .clicked()
                                {
                                    self.cfg.audio_device.clear();
                                    device_changed = true;
                                }
                                for d in &devices {
                                    if ui
                                        .selectable_label(*d == self.cfg.audio_device, d)
                                        .clicked()
                                    {
                                        self.cfg.audio_device = d.clone();
                                        device_changed = true;
                                    }
                                }
                            });
                        ui.end_row();
                        ui.label("Sound interval, sec");
                        ui.horizontal(|ui| {
                            let changed_min = ui
                                .add(
                                    egui::DragValue::new(&mut self.cfg.sound_interval_min)
                                        .range(0.1..=600.0)
                                        .speed(1.0),
                                )
                                .changed();
                            ui.label("to");
                            let changed_max = ui
                                .add(
                                    egui::DragValue::new(&mut self.cfg.sound_interval_max)
                                        .range(0.1..=600.0)
                                        .speed(1.0),
                                )
                                .changed();
                            if changed_min || changed_max {
                                // Keep min < max so the random range is never empty
                                self.cfg.sanitize();
                                self.dirty = true;
                            }
                        });
                        ui.end_row();
                        row_check(
                            ui,
                            "Freeze pet while a sound plays",
                            &mut self.cfg.stop_on_sound,
                            &mut self.dirty,
                        );
                        let n = self.sound.sounds.len();
                        ui.label("Sound files");
                        ui.label(egui::RichText::new(format!("{n}")).color(if n == 0 {
                            egui::Color32::from_rgb(220, 120, 120)
                        } else {
                            egui::Color32::from_rgb(120, 220, 150)
                        }));
                        ui.end_row();
                    });

                if device_changed {
                    // Restart the audio thread on the newly selected device
                    self.sound = SoundPlayer::new(&self.sounds_dir.clone(), &self.cfg.audio_device);
                    self.dirty = true;
                }

                // --- Appearance ---
                section(ui, "Appearance");
                egui::Grid::new("g_view")
                    .num_columns(2)
                    .spacing([12.0, 10.0])
                    .show(ui, |ui| {
                        row_check(
                            ui,
                            "Remove green background",
                            &mut self.cfg.chroma_key,
                            &mut self.dirty,
                        );
                        ui.label("");
                        ui.label(
                            egui::RichText::new("Off: show the original green background")
                                .small()
                                .color(egui::Color32::from_gray(120)),
                        );
                        ui.end_row();
                    });

                // Apply changes
                if self.dirty {
                    self.dirty = false;
                    self.sound.set_volume(self.cfg.volume);
                    if let Some(tray) = &self.tray {
                        tray.set_checked(self.cfg.paused);
                    }
                    let _ = self.cfg.save();
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Reset settings").clicked() {
                        let paused = self.cfg.paused;
                        let previous_device = self.cfg.audio_device.clone();
                        self.cfg = Config::default();
                        self.cfg.paused = paused;
                        if self.cfg.audio_device != previous_device {
                            self.sound = SoundPlayer::new(&self.sounds_dir, &self.cfg.audio_device);
                        }
                        self.dirty = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Done").clicked() {
                            self.settings_open = false;
                            let _ = self.cfg.save();
                        }
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
            });
    }

    /// Small live preview of the current frame
    fn preview(&self, ui: &mut egui::Ui, width: f32) {
        let h = width * self.aspect;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, h), egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, 8.0, egui::Color32::from_rgb(18, 20, 21));
        if let Some(tex) = &self.texture {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(tex.id(), rect, uv, egui::Color32::WHITE);
        }
    }

    fn apply_video_options_if_changed(&mut self, ctx: &egui::Context) {
        let width_changed = (self.cfg.pet_width - self.applied_width).abs() >= 0.5;
        let chroma_changed = self.cfg.chroma_key != self.applied_chroma_key;
        if !self.video_inited || (!width_changed && !chroma_changed) {
            return;
        }
        let logical_w = self.cfg.pet_width;
        let logical_h = logical_w * self.aspect;

        if let Some(pet) = &mut self.pet {
            let old_size = pet.size;
            pet.size = egui::Vec2::new(logical_w, logical_h);
            // Keep the bottom edge in place when resizing, including mid-flight.
            pet.pos.y += old_size.y - logical_h;
            pet.pos.x = pet.pos.x.clamp(0.0, (pet.screen.x - logical_w).max(0.0));
            pet.pos.y = pet.pos.y.clamp(0.0, pet.floor_y_pub().max(0.0));
        }

        let video_path = crate::config::assets_dir().join("video").join("pet.mp4");
        if video_path.exists() {
            match self.build_decoder(&video_path) {
                Ok(player) => {
                    self.video = Some(player);
                    self.video_needs_refresh = true;
                    self.frame_accum = 0.0;
                }
                Err(e) => eprintln!("Failed to rebuild decoder: {e:#}"),
            }
        }
        // Avoid retrying a failed decoder rebuild every UI frame.
        self.applied_width = logical_w;
        self.applied_chroma_key = self.cfg.chroma_key;
        if width_changed {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                logical_w, logical_h,
            )));
        }
    }
}

/// Section heading
fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(10.0);
    ui.label(
        egui::RichText::new(title.to_uppercase())
            .small()
            .strong()
            .color(egui::Color32::from_rgb(120, 220, 160)),
    );
    ui.add_space(2.0);
    ui.separator();
}

fn row_check(ui: &mut egui::Ui, label: &str, value: &mut bool, dirty: &mut bool) {
    ui.label(label);
    if ui.checkbox(value, "").changed() {
        *dirty = true;
    }
    ui.end_row();
}

fn row_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    suffix: &str,
    dirty: &mut bool,
) {
    ui.label(label);
    let mut slider = egui::Slider::new(value, range);
    if !suffix.is_empty() {
        slider = slider.suffix(suffix);
    }
    if ui.add(slider).changed() {
        *dirty = true;
    }
    ui.end_row();
}

/// Dark theme with a green accent (matching the meme)
fn setup_style(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.all_styles_mut(|style| {
        style.visuals = egui::Visuals::dark();
        let panel = egui::Color32::from_rgb(30, 33, 35);
        style.visuals.panel_fill = panel;
        style.visuals.window_fill = panel;
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(20, 22, 23);
        style.visuals.faint_bg_color = egui::Color32::from_rgb(40, 43, 46);
        let accent = egui::Color32::from_rgb(74, 222, 128);
        style.visuals.hyperlink_color = accent;
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(34, 110, 66);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, accent);
        style.spacing.item_spacing = egui::vec2(10.0, 9.0);
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.visuals.window_corner_radius = egui::CornerRadius::same(10);
    });
}
