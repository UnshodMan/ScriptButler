use std::path::PathBuf;

use serde_json::json;

use crate::output::{self, CliError};

const SKILL: &str = include_str!("../../../skills/script-butler/SKILL.md");

pub fn run(install: Option<PathBuf>, json: bool) -> Result<(), CliError> {
    match install {
        Some(root) => {
            let dir = root.join("script-butler");
            std::fs::create_dir_all(&dir).map_err(|e| CliError::Io(e.to_string()))?;
            let path = dir.join("SKILL.md");
            std::fs::write(&path, SKILL).map_err(|e| CliError::Io(e.to_string()))?;
            if json {
                output::print_json(json!({
                    "ok": true,
                    "data": { "path": path.to_string_lossy() }
                }));
            } else {
                println!("Installed skill to {}", path.display());
            }
        }
        None => print!("{SKILL}"),
    }
    Ok(())
}
