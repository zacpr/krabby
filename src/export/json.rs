use std::path::Path;

use crate::docker::ContainerInfo;
use crate::error::Result;

pub fn export_to_json(containers: &[ContainerInfo], path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(containers)?;
    std::fs::write(path, json)?;
    Ok(())
}
