pub fn detect_mime(bytes: &[u8], filename: &str) -> Option<String> {
    // Basic by extension first
    if let Some(ext) = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
    {
        let m = match ext.as_str() {
            "pdf" => Some("application/pdf"),
            "txt" => Some("text/plain"),
            "md" => Some("text/markdown"),
            "json" => Some("application/json"),
            "csv" => Some("text/csv"),
            "png" => Some("image/png"),
            "jpg" | "jpeg" => Some("image/jpeg"),
            _ => None,
        };
        if let Some(m) = m { return Some(m.to_string()); }
    }
    // Minimal sniff
    if bytes.len() >= 4 {
        let magic = &bytes[..4];
        if magic == b"%PDF" { return Some("application/pdf".into()); }
        if magic == [0x89, b'P', b'N', b'G'] { return Some("image/png".into()); }
        if &magic[..3] == b"\xFF\xD8\xFF" { return Some("image/jpeg".into()); }
    }
    None
}
