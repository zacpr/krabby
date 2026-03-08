#![cfg(feature = "tray")]

use std::sync::{Arc, Mutex};

// Global message sender - used to communicate with the main app
lazy_static::lazy_static! {
    static ref TRAY_CALLBACK: Arc<Mutex<Option<Box<dyn Fn(TrayMessage) + Send + 'static>>>> = 
        Arc::new(Mutex::new(None));
}

#[derive(Debug, Clone)]
pub enum TrayMessage {
    Show,
    Quit,
}

pub struct SystemTray;

impl SystemTray {
    pub fn new() -> (Self, std::sync::mpsc::Receiver<TrayMessage>) {
        let (sender, receiver) = std::sync::mpsc::channel::<TrayMessage>();
        
        // Store the sender in the global callback
        let sender_clone = sender.clone();
        *TRAY_CALLBACK.lock().unwrap() = Some(Box::new(move |msg| {
            let _ = sender_clone.send(msg);
        }));
        
        // Create and spawn the tray service
        let tray_service = ksni::TrayService::new(TrayMenu);
        tray_service.spawn();
        
        println!("System tray created successfully (ksni)");
        
        (Self, receiver)
    }
}

#[derive(Debug)]
struct TrayMenu;

impl ksni::Tray for TrayMenu {
    fn id(&self) -> String {
        "krabby".into()
    }

    fn title(&self) -> String {
        "Krabby Container".into()
    }

    fn icon_name(&self) -> String {
        "krabby".into()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        use ksni::MenuItem::Separator;
        
        vec![
            StandardItem {
                label: "Show Krabby".into(),
                icon_name: "window-new".into(),
                enabled: true,
                activate: Box::new(|_| {
                    if let Ok(guard) = TRAY_CALLBACK.lock() {
                        if let Some(ref callback) = *guard {
                            callback(TrayMessage::Show);
                        }
                    }
                }),
                ..Default::default()
            }
            .into(),
            Separator.into(),
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                enabled: true,
                activate: Box::new(|_| {
                    if let Ok(guard) = TRAY_CALLBACK.lock() {
                        if let Some(ref callback) = *guard {
                            callback(TrayMessage::Quit);
                        }
                    }
                }),
                ..Default::default()
            }
            .into(),
        ]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        // Left click - show window
        if let Ok(guard) = TRAY_CALLBACK.lock() {
            if let Some(ref callback) = *guard {
                callback(TrayMessage::Show);
            }
        }
    }
}
