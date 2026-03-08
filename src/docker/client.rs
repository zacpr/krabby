use bollard::container::{ListContainersOptions, StartContainerOptions, StopContainerOptions, RestartContainerOptions, RemoveContainerOptions, StatsOptions, Stats};
use bollard::models::ContainerSummary;
use bollard::Docker;
use futures::stream::StreamExt;
use std::collections::HashMap;

use super::models::*;
use crate::error::{ContainerError, Result};

pub struct DockerClient {
    docker: Docker,
}

impl DockerClient {
    pub fn new() -> Result<Self> {
        let docker = Docker::connect_with_socket_defaults()
            .map_err(|e| ContainerError::Docker(e.into()))?;
        Ok(Self { docker })
    }
    
    pub fn new_with_socket(socket_path: &str) -> Result<Self> {
        let docker = Docker::connect_with_socket(socket_path, 120, bollard::API_DEFAULT_VERSION)
            .map_err(|e| ContainerError::Docker(e.into()))?;
        Ok(Self { docker })
    }
    
    /// Fast list - only basic info from list_containers, no per-container API calls
    pub async fn list_containers_fast(&self, all: bool) -> Result<Vec<ContainerInfo>> {
        let options: ListContainersOptions<String> = ListContainersOptions {
            all,
            ..Default::default()
        };
        
        let containers = self.docker.list_containers(Some(options)).await?;
        let mut result = Vec::with_capacity(containers.len());
        
        for container in containers {
            // Fast path: use only the data from list_containers, no extra API calls
            let info = Self::summarize_to_info_fast(container)?;
            result.push(info);
        }
        
        Ok(result)
    }
    
    /// Full list with details - async fetch stats in background
    pub async fn list_containers(&self, all: bool) -> Result<Vec<ContainerInfo>> {
        // Get basic info fast
        let mut containers = self.list_containers_fast(all).await?;
        
        // Fetch stats for running containers concurrently (but don't block UI)
        let running_ids: Vec<String> = containers
            .iter()
            .filter(|c| c.state == ContainerState::Running)
            .map(|c| c.id.clone())
            .collect();
        
        // Fetch all stats concurrently
        let mut stats_futures = Vec::new();
        for id in &running_ids {
            let docker = self.docker.clone();
            let id = id.clone();
            stats_futures.push(async move {
                let options = StatsOptions { stream: false, ..Default::default() };
                let mut stream = docker.stats(&id, Some(options));
                if let Some(Ok(stats)) = stream.next().await {
                    (id, Self::extract_stats(&stats))
                } else {
                    (id, (None, None))
                }
            });
        }
        
        // Wait for all stats in parallel
        let stats_results: Vec<(String, (Option<CpuStats>, Option<MemoryStats>))> = 
            futures::future::join_all(stats_futures).await;
        
        // Apply stats to containers
        let stats_map: HashMap<String, (Option<CpuStats>, Option<MemoryStats>)> = 
            stats_results.into_iter().collect();
        
        for container in &mut containers {
            if let Some((cpu, mem)) = stats_map.get(&container.id) {
                container.cpu_stats = cpu.clone();
                container.memory_stats = mem.clone();
            }
        }
        
        Ok(containers)
    }
    
    /// Fast conversion - no API calls
    fn summarize_to_info_fast(summary: ContainerSummary) -> Result<ContainerInfo> {
        let id = summary.id.unwrap_or_default();
        let short_id = id.chars().take(12).collect();
        
        let names = summary.names.unwrap_or_default();
        let name = names.first()
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_else(|| id.clone());
        
        let image = summary.image.unwrap_or_default();
        let image_id = summary.image_id.unwrap_or_default();
        let status = summary.status.unwrap_or_default();
        let state = ContainerState::from(summary.state.as_deref().unwrap_or("unknown"));
        
        let created = summary.created
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
            .unwrap_or_else(chrono::Utc::now);
        
        let ports = summary.ports.unwrap_or_default()
            .into_iter()
            .map(|p| PortMapping {
                private_port: p.private_port,
                public_port: p.public_port.map(|p| p as u16),
                protocol: p.typ.map(|t| format!("{:?}", t)).unwrap_or_else(|| "tcp".to_string()),
                ip: p.ip,
            })
            .collect();
        
        // Note: command, env, mounts, labels, network_mode, restart_policy 
        // are NOT available from list_containers - we'd need inspect_container
        // For fast loading, we skip these or use empty defaults
        
        Ok(ContainerInfo {
            id,
            short_id,
            name,
            image,
            image_id,
            status,
            state,
            created,
            ports,
            mounts: Vec::new(), // Not available in list_containers
            env: Vec::new(),    // Not available in list_containers  
            labels: summary.labels.unwrap_or_default(),
            command: String::new(), // Not available in list_containers
            cpu_stats: None,
            memory_stats: None,
            network_mode: None,
            restart_policy: None,
        })
    }
    
    fn extract_stats(stats: &Stats) -> (Option<CpuStats>, Option<MemoryStats>) {
        let cpu_stats = Self::calculate_cpu_percent(stats);
        let mem_stats = Self::calculate_memory_stats(stats);
        (cpu_stats, mem_stats)
    }
    
    fn calculate_cpu_percent(stats: &Stats) -> Option<CpuStats> {
        let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
            - stats.precpu_stats.cpu_usage.total_usage as f64;
        let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
            - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
        
        if system_delta > 0.0 && cpu_delta > 0.0 {
            let cpu_count = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;
            let usage_percent = (cpu_delta / system_delta) * cpu_count * 100.0;
            Some(CpuStats { usage_percent })
        } else {
            None
        }
    }
    
    fn calculate_memory_stats(stats: &Stats) -> Option<MemoryStats> {
        let usage = stats.memory_stats.usage.unwrap_or(0);
        let limit = stats.memory_stats.limit.unwrap_or(1);
        
        if limit > 0 {
            let usage_mb = usage / (1024 * 1024);
            let limit_mb = limit / (1024 * 1024);
            let usage_percent = (usage as f64 / limit as f64) * 100.0;
            
            Some(MemoryStats {
                usage_mb,
                limit_mb,
                usage_percent,
            })
        } else {
            None
        }
    }
    
    /// Get full container details (slow - uses inspect_container)
    pub async fn get_container_details(&self, id: &str) -> Result<ContainerInfo> {
        let summary = ContainerSummary {
            id: Some(id.to_string()),
            ..Default::default()
        };
        
        let mut info = Self::summarize_to_info_fast(summary)?;
        
        // Get detailed info
        if let Ok(details) = self.docker.inspect_container(id, None).await {
            let config = details.config;
            let host_config = details.host_config;
            
            info.command = config.as_ref()
                .and_then(|c| c.cmd.as_ref())
                .map(|c| c.join(" "))
                .unwrap_or_default();
            
            info.env = config.as_ref()
                .and_then(|c| c.env.clone())
                .unwrap_or_default();
            
            info.mounts = details.mounts
                .unwrap_or_default()
                .into_iter()
                .map(|m| Mount {
                    source: m.source.unwrap_or_default(),
                    destination: m.destination.unwrap_or_default(),
                    mode: m.mode.unwrap_or_else(|| "rw".to_string()),
                    mount_type: m.typ.map(|t| format!("{:?}", t)).unwrap_or_else(|| "bind".to_string()),
                })
                .collect();
            
            info.network_mode = host_config.as_ref()
                .and_then(|h| h.network_mode.clone());
            
            info.restart_policy = host_config.as_ref()
                .and_then(|h| h.restart_policy.as_ref())
                .and_then(|r| r.name.clone())
                .map(|n| format!("{:?}", n).to_lowercase());
        }
        
        // Get stats if running
        if info.state == ContainerState::Running {
            let options = StatsOptions { stream: false, ..Default::default() };
            let mut stream = self.docker.stats(id, Some(options));
            if let Some(Ok(stats)) = stream.next().await {
                (info.cpu_stats, info.memory_stats) = Self::extract_stats(&stats);
            }
        }
        
        Ok(info)
    }
    
    pub async fn start_container(&self, id: &str) -> Result<()> {
        self.docker.start_container(id, None::<StartContainerOptions<String>>).await?;
        Ok(())
    }
    
    pub async fn stop_container(&self, id: &str, timeout: Option<i64>) -> Result<()> {
        let options = StopContainerOptions {
            t: timeout.unwrap_or(10),
        };
        self.docker.stop_container(id, Some(options)).await?;
        Ok(())
    }
    
    pub async fn restart_container(&self, id: &str, timeout: Option<i64>) -> Result<()> {
        let options = RestartContainerOptions {
            t: timeout.unwrap_or(10) as isize,
        };
        self.docker.restart_container(id, Some(options)).await?;
        Ok(())
    }
    
    pub async fn remove_container(&self, id: &str, force: bool) -> Result<()> {
        let options = RemoveContainerOptions {
            force,
            v: false,
            link: false,
        };
        self.docker.remove_container(id, Some(options)).await?;
        Ok(())
    }
    
    pub async fn get_container_logs(&self, id: &str, tail: usize) -> Result<String> {
        use bollard::container::LogsOptions;
        
        let options = LogsOptions {
            stdout: true,
            stderr: true,
            tail: format!("{}", tail),
            timestamps: false,
            follow: false,
            ..Default::default()
        };
        
        let mut stream = self.docker.logs(id, Some(options));
        let mut logs = String::new();
        
        while let Some(chunk) = stream.next().await {
            if let Ok(chunk) = chunk {
                logs.push_str(&String::from_utf8_lossy(&chunk.into_bytes()));
            }
        }
        
        Ok(logs)
    }
    
    pub async fn ping(&self) -> Result<String> {
        let version = self.docker.version().await?;
        Ok(format!("Docker API Version: {:?}", version.api_version))
    }
}

impl Clone for DockerClient {
    fn clone(&self) -> Self {
        Self { 
            docker: self.docker.clone() 
        }
    }
}
