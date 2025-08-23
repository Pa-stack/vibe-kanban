use anyhow::Result;
use std::{fs, path::Path};

pub struct Artifacts;

impl Artifacts {
    pub fn ensure_dir(dir: &Path) -> Result<()> { fs::create_dir_all(dir)?; Ok(()) }

    pub fn write_touched_files(dir: &Path, files: &[String]) -> Result<()> {
        let path = dir.join("touched_files.txt");
        let content = files.join("\n");
        fs::write(path, content)?; Ok(())
    }

    pub fn write_dep_snapshot(dir: &Path, snapshot: &str) -> Result<()> {
        let path = dir.join("dep_snapshot.txt");
        fs::write(path, snapshot)?; Ok(())
    }

    pub fn write_kpi_json(dir: &Path, json: &serde_json::Value) -> Result<()> {
        let path = dir.join("kpi.json");
        fs::write(path, serde_json::to_vec_pretty(json)?)?; Ok(())
    }

    pub fn write_snippets_log(dir: &Path, content: &str) -> Result<()> {
        let path = dir.join("snippets.log");
        fs::write(path, content)?; Ok(())
    }
}
