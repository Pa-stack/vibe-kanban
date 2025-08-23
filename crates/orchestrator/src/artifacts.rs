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

pub fn simple_dep_snapshot(workdir: &Path) -> String {
    let mut out = String::new();
    let cargo_lock = workdir.join("Cargo.lock");
    if let Ok(text) = fs::read_to_string(&cargo_lock) {
        for line in text.lines() {
            if line.starts_with("name = \"") {
                out.push_str(line);
                out.push('\n');
            }
            if line.starts_with("version = \"") {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    let pkg_json = workdir.join("package.json");
    if let Ok(text) = fs::read_to_string(&pkg_json) {
        out.push_str("\n[package.json deps]\n");
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(obj) = v.get("dependencies").and_then(|d| d.as_object()) {
                for (k, _) in obj.iter() {
                    out.push_str(k);
                    out.push('\n');
                }
            }
        }
    }
    out
}
