//! Tray icon with a menu. Created on the main thread before eframe starts.

use anyhow::Result;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub struct Tray {
    pub settings_id: MenuId,
    pub toggle_pause_id: MenuId,
    pub quit_id: MenuId,
    _tray: TrayIcon,
    pub check_pause: CheckMenuItem,
}

impl Tray {
    pub fn new(icon_rgba: Vec<u8>, icon_size: (u32, u32)) -> Result<Self> {
        let icon = Icon::from_rgba(icon_rgba, icon_size.0, icon_size.1)?;

        let check_pause = CheckMenuItem::new("Pause", true, false, None);
        let settings = MenuItem::new("Settings", true, None);
        let quit = MenuItem::new("Quit", true, None);

        let menu = Menu::new();
        menu.append_items(&[&check_pause, &settings, &quit])?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Hood Irony Desktop Pet")
            .with_icon(icon)
            .build()?;

        Ok(Self {
            settings_id: settings.id().clone(),
            toggle_pause_id: check_pause.id().clone(),
            quit_id: quit.id().clone(),
            _tray: tray,
            check_pause,
        })
    }

    /// Poll menu events.
    pub fn poll_events(&self) -> Option<TrayEvent> {
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.settings_id {
                return Some(TrayEvent::Settings);
            } else if event.id == self.toggle_pause_id {
                return Some(TrayEvent::PauseToggle);
            } else if event.id == self.quit_id {
                return Some(TrayEvent::Quit);
            }
        }
        None
    }

    pub fn set_checked(&self, checked: bool) {
        self.check_pause.set_checked(checked);
    }
}

pub enum TrayEvent {
    Settings,
    PauseToggle,
    Quit,
}
