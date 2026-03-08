use std::path::Path;

use crate::docker::ContainerInfo;
use crate::error::Result;

pub fn export_to_csv(containers: &[ContainerInfo], path: &Path) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    
    // Write headers
    writer.write_record(&[
        "ID",
        "Name",
        "Image",
        "State",
        "Status",
        "Created",
        "Ports",
        "Command",
    ])?;
    
    for container in containers {
        let ports = container.ports.iter()
            .map(|p| {
                if let Some(public) = p.public_port {
                    format!("{}:{}/{}", public, p.private_port, p.protocol)
                } else {
                    format!("{}/{}", p.private_port, p.protocol)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        
        writer.write_record(&[
            &container.short_id,
            &container.name,
            &container.image,
            &container.state.to_string(),
            &container.status,
            &container.created.format("%Y-%m-%d %H:%M:%S").to_string(),
            &ports,
            &container.command,
        ])?;
    }
    
    writer.flush()?;
    Ok(())
}
