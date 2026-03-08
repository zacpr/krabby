#![allow(dead_code)]
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContainerError {
    #[error("Docker API error: {0}")]
    Docker(#[from] bollard::errors::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Export error: {0}")]
    Export(String),
    
    #[error("Registry check error: {0}")]
    Registry(String),
    
    #[error("Tray error: {0}")]
    Tray(String),
    
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ContainerError>;
