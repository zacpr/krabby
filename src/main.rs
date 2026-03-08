mod config;
mod docker;
mod error;
mod export;
mod ui;

#[cfg(feature = "tray")]
mod tray;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use iced::{window, Subscription, Size, Task};
use ui::app::{ContainerManagerApp, Message};

#[cfg(feature = "tray")]
use tray::{SystemTray, TrayMessage};

// Global tray receiver - needs to be accessible from subscription
#[cfg(feature = "tray")]
lazy_static::lazy_static! {
    static ref TRAY_RECEIVER: Arc<Mutex<Option<std::sync::mpsc::Receiver<TrayMessage>>>> = 
        Arc::new(Mutex::new(None));
}

fn main() -> iced::Result {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();
    
    // Ensure single instance
    let instance = single_instance::SingleInstance::new("krabby_single_instance")
        .expect("Failed to check single instance");
    
    if !instance.is_single() {
        eprintln!("Krabby is already running!");
        std::process::exit(1);
    }
    
    // Initialize system tray if feature is enabled
    #[cfg(feature = "tray")]
    {
        // Create system tray (ksni handles D-Bus registration automatically)
        let (tray, tray_receiver) = SystemTray::new();
        *TRAY_RECEIVER.lock().unwrap() = Some(tray_receiver);
        
        // Store tray to keep it alive
        let _tray = tray;
        println!("System tray initialized");
    }
    
    // Run the iced application with software rendering for compatibility
    iced::application(
        ContainerManagerApp::title,
        update,
        ContainerManagerApp::view,
    )
    .subscription(subscription)
    .theme(ContainerManagerApp::theme)
    .window(window::Settings {
        size: Size::new(1200.0, 800.0),
        position: window::Position::Centered,
        min_size: Some(Size::new(800.0, 600.0)),
        decorations: true,
        transparent: false,
        visible: true,
        ..Default::default()
    })
    .antialiasing(false) // Disable antialiasing for compatibility
    .run_with(ContainerManagerApp::new)
}

// Wrapper update function to handle window visibility
fn update(app: &mut ContainerManagerApp, message: Message) -> Task<Message> {
    match &message {
        Message::ShowWindow => {
            // Show and focus the latest window
            return window::get_latest().and_then(|id| {
                window::gain_focus(id)
            });
        }
        Message::HideWindow => {
            // Just minimize - proper hide isn't supported well in this iced version
            // User can close the window to hide it
            return Task::none();
        }
        _ => {}
    }
    
    app.update(message)
}

fn subscription(app: &ContainerManagerApp) -> Subscription<Message> {
    let app_subscription = app.subscription();
    let tray_sub = tray_subscription(app);
    
    Subscription::batch([app_subscription, tray_sub])
}

#[cfg(feature = "tray")]
fn tray_subscription(_state: &ContainerManagerApp) -> Subscription<Message> {
    iced::time::every(Duration::from_millis(100)).map(|_| {
        // Check for tray messages
        if let Ok(guard) = TRAY_RECEIVER.lock() {
            if let Some(ref receiver) = *guard {
                match receiver.try_recv() {
                    Ok(TrayMessage::Show) => Message::ShowWindow,
                    Ok(TrayMessage::Quit) => Message::Exit,
                    Err(_) => Message::Tick,
                }
            } else {
                Message::Tick
            }
        } else {
            Message::Tick
        }
    })
}

#[cfg(not(feature = "tray"))]
fn tray_subscription(_state: &ContainerManagerApp) -> Subscription<Message> {
    // No tray support, return empty subscription
    Subscription::none()
}
