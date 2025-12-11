use std::fs;
use std::path::Path;

fn main() {
    // Sync version from version.txt to tauri.conf.json before build
    sync_version();
    
    tauri_build::build()
}

fn sync_version() {
    let version_file = Path::new("../version.txt");
    let config_file = Path::new("tauri.conf.json");
    
    // Read version from version.txt
    let version = match fs::read_to_string(version_file) {
        Ok(v) => v.trim().to_string(),
        Err(e) => {
            println!("cargo:warning=Could not read version.txt: {}", e);
            return;
        }
    };
    
    if version.is_empty() {
        println!("cargo:warning=version.txt is empty");
        return;
    }
    
    // Read tauri.conf.json
    let config_content = match fs::read_to_string(config_file) {
        Ok(c) => c,
        Err(e) => {
            println!("cargo:warning=Could not read tauri.conf.json: {}", e);
            return;
        }
    };
    
    // Find and replace version using simple string matching
    // Look for "version": "x.x.x" pattern (after $schema line)
    let mut updated_config = String::new();
    let mut version_updated = false;
    
    for line in config_content.lines() {
        if !version_updated && line.contains("\"version\"") && line.contains(":") {
            // Extract indentation
            let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
            updated_config.push_str(&format!("{}\"version\": \"{}\",\n", indent, version));
            version_updated = true;
        } else {
            updated_config.push_str(line);
            updated_config.push('\n');
        }
    }
    
    // Remove trailing newline if original didn't have one
    if !config_content.ends_with('\n') && updated_config.ends_with('\n') {
        updated_config.pop();
    }
    
    // Only write if changed
    if updated_config != config_content && version_updated {
        if let Err(e) = fs::write(config_file, &updated_config) {
            println!("cargo:warning=Could not write tauri.conf.json: {}", e);
            return;
        }
        println!("cargo:warning=Synced tauri.conf.json version to: {}", version);
    }
    
    // Tell cargo to rerun if version.txt changes
    println!("cargo:rerun-if-changed=../version.txt");
}
