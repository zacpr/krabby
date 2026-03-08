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
    
    // Spawn a background thread to hide the iced boot window from the taskbar.
    // Iced 0.13 creates an internal "winit window" (boot window) for compositor 
    // initialization that KDE Plasma shows in the taskbar despite being invisible.
    hide_boot_window_from_taskbar();
    
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
        exit_on_close_request: false, // Don't exit when window is closed - hide to tray instead
        ..Default::default()
    })
    .antialiasing(false) // Disable antialiasing for compatibility
    .run_with(ContainerManagerApp::new)
}

/// Spawns a background thread that finds and hides the iced "boot window" from the taskbar.
///
/// Iced 0.13 (via iced_winit) creates a hidden helper window with the default title "winit window"
/// during compositor initialization. While it's created with `visible: false`, KDE Plasma on
/// Wayland/XWayland still shows it in the taskbar. This function uses KWin D-Bus scripting
/// (on Wayland) or xdotool/xprop (on X11) to hide it.
fn hide_boot_window_from_taskbar() {
    std::thread::spawn(|| {
        // Give iced time to create the boot window
        std::thread::sleep(std::time::Duration::from_millis(1000));
        
        // Try KWin scripting first (works on Wayland + KDE)
        if try_hide_via_kwin_script() {
            println!("Hid boot window from taskbar via KWin scripting");
            return;
        }
        
        // Fallback: try xdotool (works on X11)
        if try_hide_via_xdotool() {
            println!("Hid boot window from taskbar via xdotool");
            return;
        }
        
        eprintln!("Could not hide iced boot window from taskbar (not critical)");
    });
}

/// Try to hide the "winit window" using KWin's D-Bus scripting API.
/// This works on KDE Plasma with Wayland.
fn try_hide_via_kwin_script() -> bool {
    // Write a temporary KWin script that finds and hides the boot window
    let script_content = r#"
var clients = workspace.windowList();
for (var i = 0; i < clients.length; i++) {
    var w = clients[i];
    if (w.caption === "winit window") {
        w.skipTaskbar = true;
        w.skipPager = true;
        w.skipSwitcher = true;
    }
}
"#;
    
    let script_path = "/tmp/krabby_hide_boot_window.js";
    if std::fs::write(script_path, script_content).is_err() {
        return false;
    }
    
    // Load the script via D-Bus
    let load_result = std::process::Command::new("qdbus")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.loadScript",
            script_path,
            "krabby_hide_boot",
        ])
        .output();
    
    let script_id = match load_result {
        Ok(result) if result.status.success() => {
            String::from_utf8_lossy(&result.stdout).trim().to_string()
        }
        _ => return false,
    };
    
    if script_id.is_empty() {
        return false;
    }
    
    // Run the script
    let run_result = std::process::Command::new("qdbus")
        .args([
            "org.kde.KWin",
            &format!("/Scripting/Script{}", script_id),
            "org.kde.kwin.Script.run",
        ])
        .output();
    
    // Small delay to let the script execute
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    // Unload the script to clean up
    let _ = std::process::Command::new("qdbus")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.unloadScript",
            "krabby_hide_boot",
        ])
        .output();
    
    // Clean up temp file
    let _ = std::fs::remove_file(script_path);
    
    run_result.map(|r| r.status.success()).unwrap_or(false)
}

/// Try to hide the "winit window" using xdotool + xprop (X11 only).
fn try_hide_via_xdotool() -> bool {
    for attempt in 0..5 {
        let output = std::process::Command::new("xdotool")
            .args(["search", "--name", "^winit window$"])
            .output();
        
        match output {
            Ok(result) if result.status.success() => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let window_ids: Vec<&str> = stdout.trim().lines().collect();
                
                if window_ids.is_empty() {
                    if attempt < 4 {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        continue;
                    }
                    return false;
                }
                
                for wid in &window_ids {
                    let wid = wid.trim();
                    if wid.is_empty() {
                        continue;
                    }
                    
                    // Set skip-taskbar and skip-pager hints via xprop
                    let _ = std::process::Command::new("xprop")
                        .args([
                            "-id", wid,
                            "-f", "_NET_WM_STATE", "32a",
                            "-set", "_NET_WM_STATE",
                            "_NET_WM_STATE_SKIP_TASKBAR,_NET_WM_STATE_SKIP_PAGER",
                        ])
                        .output();
                }
                return true;
            }
            _ => {
                if attempt < 4 {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    continue;
                }
                return false;
            }
        }
    }
    false
}

// Wrapper update function to handle window visibility
fn update(app: &mut ContainerManagerApp, message: Message) -> Task<Message> {
    match &message {
        Message::ShowWindow => {
            // Show window by changing mode to Windowed, then focus it
            return window::get_latest().and_then(|id| {
                Task::batch([
                    window::change_mode(id, window::Mode::Windowed),
                    window::gain_focus(id),
                ])
            });
        }
        Message::HideWindow => {
            // Hide window to system tray by setting mode to Hidden
            return window::get_latest().and_then(|id| {
                window::change_mode(id, window::Mode::Hidden)
            });
        }
        Message::CloseRequested(_) => {
            // Window close button was clicked - hide to tray instead of closing
            return window::get_latest().and_then(|id| {
                window::change_mode(id, window::Mode::Hidden)
            });
        }
        Message::Exit => {
            // Actually quit the application - close all windows
            return window::get_latest().and_then(|id| {
                window::close(id)
            });
        }
        _ => {}
    }
    
    app.update(message)
}

fn subscription(app: &ContainerManagerApp) -> Subscription<Message> {
    let app_subscription = app.subscription();
    let tray_sub = tray_subscription(app);
    
    // Subscribe to window close requests so we can intercept them
    let close_sub = window::close_requests().map(Message::CloseRequested);
    
    Subscription::batch([app_subscription, tray_sub, close_sub])
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
