use std::{fs, path::Path};

pub struct Artifacts;

impl Artifacts {
    pub fn ensure_dir(dir: &Path) -> Result<(), String> { fs::create_dir_all(dir).map_err(|e| e.to_string())?; Ok(()) }

    pub fn write_touched_files(dir: &Path, files: &[String]) -> Result<(), String> {
        let path = dir.join("touched_files.txt");
        let content = files.join("\n");
        fs::write(path, content).map_err(|e| e.to_string())?; Ok(())
    }

    pub fn write_dep_snapshot(dir: &Path, snapshot: &str) -> Result<(), String> {
        let path = dir.join("dep_snapshot.txt");
        fs::write(path, snapshot).map_err(|e| e.to_string())?; Ok(())
    }

    pub fn write_kpi_json_raw(dir: &Path, json_bytes: &[u8]) -> Result<(), String> {
        let path = dir.join("kpi.json");
        fs::write(path, json_bytes).map_err(|e| e.to_string())?; Ok(())
    }

    pub fn write_snippets_log(dir: &Path, content: &str) -> Result<(), String> {
        let path = dir.join("snippets.log");
        fs::write(path, content).map_err(|e| e.to_string())?; Ok(())
    }
}
