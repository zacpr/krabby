use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: Theme,
    pub columns: Vec<ColumnConfig>,
    pub auto_refresh_interval: u64, // seconds
    pub enable_notifications: bool,
    pub check_image_updates: bool,
    pub window_geometry: Option<WindowGeometry>,
    pub docker_socket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub accent_color: String,
    pub background_color: String,
    pub text_color: String,
    pub border_radius: f32,
    pub enable_animations: bool,
    pub enable_effects: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnConfig {
    pub name: String,
    pub visible: bool,
    pub width: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            columns: vec![
                ColumnConfig { name: "name".to_string(), visible: true, width: 200 },
                ColumnConfig { name: "image".to_string(), visible: true, width: 250 },
                ColumnConfig { name: "status".to_string(), visible: true, width: 120 },
                ColumnConfig { name: "state".to_string(), visible: true, width: 100 },
                ColumnConfig { name: "ports".to_string(), visible: true, width: 150 },
                ColumnConfig { name: "created".to_string(), visible: false, width: 150 },
                ColumnConfig { name: "id".to_string(), visible: false, width: 100 },
            ],
            auto_refresh_interval: 5,
            enable_notifications: true,
            check_image_updates: false,
            window_geometry: None,
            docker_socket: None,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "midnight".to_string(),
            accent_color: "#7c3aed".to_string(), // violet-600
            background_color: "#0f0f23".to_string(), // deep midnight
            text_color: "#e2e8f0".to_string(), // slate-200
            border_radius: 12.0,
            enable_animations: true,
            enable_effects: true,
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("krabby")
    }
    
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }
    
    pub fn load() -> crate::error::Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: AppConfig = toml::from_str(&content)
                .map_err(|e| crate::error::ContainerError::Config(e.to_string()))?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> crate::error::Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::error::ContainerError::Config(e.to_string()))?;
        std::fs::write(Self::config_path(), content)?;
        Ok(())
    }
}

pub const THEMES: &[(&str, &str, &str, &str)] = &[
    ("midnight", "#0f0f23", "#e2e8f0", "#7c3aed"),
    ("ocean", "#0c4a6e", "#e0f2fe", "#0ea5e9"),
    ("forest", "#064e3b", "#d1fae5", "#10b981"),
    ("rose", "#881337", "#ffe4e6", "#f43f5e"),
    ("amber", "#451a03", "#fef3c7", "#f59e0b"),
];
