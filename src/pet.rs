//! Pet behavior state machine: Idle / Walk / Sound / Dragged / Drop

use crate::config::Config;
use rand::RngExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Idle,
    Walk,
    Sound,
    Dragged,
    /// Falling after being dropped
    Drop,
}

pub struct Pet {
    /// Window top-left position in logical points
    pub pos: egui::Vec2,
    /// Horizontal velocity (px/s), sign = direction
    pub vx: f32,
    /// Vertical velocity used while falling
    pub vy: f32,
    pub state: State,
    /// Timer until next state change
    pub timer: f32,
    /// Timer until the next random sound
    pub sound_timer: f32,
    /// Screen size in logical points
    pub screen: egui::Vec2,
    /// Pet window size in logical points
    pub size: egui::Vec2,
    /// Offset of the cursor inside the window when the drag started
    grab: egui::Vec2,
}

impl Pet {
    pub fn new(size: egui::Vec2, screen: egui::Vec2) -> Self {
        let mut rng = rand::rng();
        let dir: f32 = if rng.random_bool(0.5) { 1.0 } else { -1.0 };
        Self {
            pos: egui::Vec2::new(screen.x * 0.5 - size.x * 0.5, screen.y - size.y - 48.0),
            vx: 40.0 * dir,
            vy: 0.0,
            state: State::Idle,
            timer: 1.0,
            sound_timer: rng.random_range(8.0..20.0), // first sound early, for testing
            screen,
            size,
            grab: egui::Vec2::ZERO,
        }
    }

    pub fn walk_speed(&self, cfg: &Config) -> f32 {
        cfg.walk_speed
    }

    /// Start dragging. `local` is the cursor position inside the window.
    pub fn begin_drag(&mut self, local: egui::Vec2) {
        self.state = State::Dragged;
        self.grab = local;
        self.vx = 0.0;
        self.vy = 0.0;
    }

    /// Update position while dragging.
    ///
    /// egui reports the cursor position *inside the window*, not on the screen.
    /// With the window's top-left at `pos`, the global cursor is `pos + local`.
    /// Keeping the grab offset constant: `pos = pos + local - grab`.
    pub fn drag_update(&mut self, local: egui::Vec2) {
        if self.state == State::Dragged {
            self.pos += local - self.grab;
        }
    }

    pub fn release(&mut self) {
        if self.state == State::Dragged {
            self.state = State::Drop;
            self.vy = 0.0;
        }
    }

    fn floor_y(&self) -> f32 {
        self.screen.y - self.size.y
    }

    pub fn floor_y_pub(&self) -> f32 {
        self.floor_y()
    }

    pub fn play_sound(&mut self, duration: f32, cfg: &Config) {
        if cfg.stop_on_sound {
            self.state = State::Sound;
            self.timer = duration.max(0.5);
        }
    }

    /// Advance the logic by `dt` seconds.
    /// Returns true when a random sound should be played.
    pub fn update(&mut self, dt: f32, cfg: &Config) -> bool {
        if cfg.paused {
            return false;
        }

        let mut play_sound = false;

        // The sound timer ticks unless we are dragging
        if self.state != State::Dragged {
            self.sound_timer -= dt;
            if self.sound_timer <= 0.0 {
                let mut rng = rand::rng();
                self.sound_timer = rng.random_range(cfg.sound_interval_min..cfg.sound_interval_max);
                play_sound = true;
            }
        }

        match self.state {
            State::Idle => {
                self.timer -= dt;
                if self.timer <= 0.0 {
                    let mut rng = rand::rng();
                    self.state = State::Walk;
                    self.vx = self.walk_speed(cfg) * if rng.random_bool(0.5) { 1.0 } else { -1.0 };
                }
            }
            State::Walk => {
                self.pos.x += self.vx * dt;
                // Turn around at the screen edges
                if self.pos.x <= 0.0 {
                    self.pos.x = 0.0;
                    self.vx = self.vx.abs();
                } else if self.pos.x + self.size.x >= self.screen.x {
                    self.pos.x = self.screen.x - self.size.x;
                    self.vx = -self.vx.abs();
                }
                // Occasional random pause
                let mut rng = rand::rng();
                if rng.random_bool(dt as f64 * 0.05) {
                    self.state = State::Idle;
                    self.timer = rng.random_range(1.0..4.0);
                }
            }
            State::Sound => {
                self.timer -= dt;
                if self.timer <= 0.0 {
                    self.state = State::Walk;
                }
            }
            State::Dragged => {
                // Position is driven by the cursor
            }
            State::Drop => {
                self.vy += 2000.0 * dt; // gravity
                self.pos.y += self.vy * dt;
                if self.pos.y >= self.floor_y() {
                    self.pos.y = self.floor_y();
                    self.vy = 0.0;
                    self.state = State::Idle;
                    self.timer = 1.0;
                }
            }
        }

        play_sound
    }

    /// Whether the animation should play right now.
    /// When idle/sound/dragged the video freezes, so the animation matches motion.
    pub fn animating(&self) -> bool {
        matches!(self.state, State::Walk | State::Drop)
    }

    /// Facing direction: 1 = right, -1 = left
    pub fn facing(&self) -> f32 {
        if self.vx >= 0.0 {
            1.0
        } else {
            -1.0
        }
    }
}
