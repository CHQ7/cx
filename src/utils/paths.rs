use std::path::PathBuf;

/// Get the project root directory
pub fn project_root() -> PathBuf {
    // Try to find the project root by looking for Cargo.toml
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    while !path.join("Cargo.toml").exists() {
        if !path.pop() {
            break;
        }
    }

    path
}

/// Get the assets directory
pub fn assets_dir() -> PathBuf {
    project_root().join("assets")
}

/// Get the memory directory
pub fn memory_dir() -> PathBuf {
    project_root().join("memory")
}

/// Get the temp directory
pub fn temp_dir() -> PathBuf {
    project_root().join("temp")
}

/// Read global memory file
pub fn read_global_memory() -> String {
    let path = memory_dir().join("global_mem.txt");

    if !path.exists() {
        // Create default memory file
        let default = "# [Global Memory - L2]\n".to_string();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&path, &default).ok();
        return default;
    }

    std::fs::read_to_string(&path).unwrap_or_default()
}

/// Write to global memory file
pub fn write_global_memory(content: &str) -> std::io::Result<()> {
    let path = memory_dir().join("global_mem.txt");
    std::fs::write(&path, content)
}

/// Append to global memory file
pub fn append_global_memory(content: &str) -> std::io::Result<()> {
    use std::io::Write;
    let path = memory_dir().join("global_mem.txt");
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)?;
    writeln!(file, "{}", content)
}
