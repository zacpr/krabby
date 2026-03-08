#![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub short_id: String,
    pub name: String,
    pub image: String,
    pub image_id: String,
    pub status: String,
    pub state: ContainerState,
    pub created: DateTime<Utc>,
    pub ports: Vec<PortMapping>,
    pub mounts: Vec<Mount>,
    pub env: Vec<String>,
    pub labels: std::collections::HashMap<String, String>,
    pub command: String,
    pub cpu_stats: Option<CpuStats>,
    pub memory_stats: Option<MemoryStats>,
    pub network_mode: Option<String>,
    pub restart_policy: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerState {
    Running,
    Paused,
    Restarting,
    Exited,
    Dead,
    Created,
    Unknown,
}

impl From<&str> for ContainerState {
    fn from(s: &str) -> Self {
        match s {
            "running" => Self::Running,
            "paused" => Self::Paused,
            "restarting" => Self::Restarting,
            "exited" => Self::Exited,
            "dead" => Self::Dead,
            "created" => Self::Created,
            _ => Self::Unknown,
        }
    }
}

impl std::fmt::Display for ContainerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "Running"),
            Self::Paused => write!(f, "Paused"),
            Self::Restarting => write!(f, "Restarting"),
            Self::Exited => write!(f, "Stopped"),
            Self::Dead => write!(f, "Dead"),
            Self::Created => write!(f, "Created"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub protocol: String,
    pub ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mount {
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub mount_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStats {
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub usage_mb: u64,
    pub limit_mb: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUpdateInfo {
    pub image_name: String,
    pub current_digest: String,
    pub latest_digest: Option<String>,
    pub update_available: bool,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposeService {
    pub name: String,
    pub image: String,
    pub container_name: Option<String>,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub environment: Vec<String>,
    pub restart: Option<String>,
    pub command: Option<String>,
    pub labels: std::collections::HashMap<String, String>,
    pub networks: Vec<String>,
}
