use std::collections::HashMap;

use crate::docker::ContainerInfo;
use crate::error::Result;

pub fn generate_compose(containers: &[ContainerInfo]) -> Result<String> {
    let mut services = HashMap::new();
    
    for container in containers {
        let service = container_to_service(container);
        let service_name = sanitize_service_name(&container.name);
        services.insert(service_name, service);
    }
    
    let compose = serde_yaml::to_string(&ComposeFile {
        version: "3.8".to_string(),
        services,
    })?;
    
    Ok(compose)
}

#[derive(serde::Serialize)]
struct ComposeFile {
    version: String,
    services: HashMap<String, ComposeServiceOutput>,
}

#[derive(serde::Serialize, Default)]
struct ComposeServiceOutput {
    image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    container_name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ports: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    volumes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "environment")]
    env: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    restart: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    labels: HashMap<String, String>,
}

fn container_to_service(container: &ContainerInfo) -> ComposeServiceOutput {
    let ports = container.ports.iter()
        .filter_map(|p| {
            p.public_port.map(|pub_port| {
                format!("{}:{}/{}", pub_port, p.private_port, p.protocol)
            })
        })
        .collect();
    
    let volumes = container.mounts.iter()
        .map(|m| format!("{}:{}", m.source, m.destination))
        .collect();
    
    let command = if container.command.is_empty() {
        None
    } else {
        Some(container.command.clone())
    };
    
    ComposeServiceOutput {
        image: container.image.clone(),
        container_name: Some(container.name.clone()),
        ports,
        volumes,
        env: container.env.clone(),
        restart: container.restart_policy.clone(),
        command,
        labels: container.labels.clone(),
    }
}

fn sanitize_service_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sanitize_service_name() {
        assert_eq!(sanitize_service_name("my-app"), "my-app");
        assert_eq!(sanitize_service_name("My App"), "my_app");
        assert_eq!(sanitize_service_name("app/container"), "app_container");
    }
}
