//! Server Scanner

use super::types::{DiscoveredServer, DiscoveryError};
use antikythera_core::logging::DiscoveryLogger;
use std::path::Path;

/// Scan a folder for MCP server binaries.
pub fn scan_folder(folder_path: impl AsRef<Path>) -> Result<Vec<DiscoveredServer>, DiscoveryError> {
    let log = DiscoveryLogger::new("discovery");
    let folder = folder_path.as_ref();

    log.info(format!(
        "Scanning servers folder | path={}",
        folder.display()
    ));

    if !folder.exists() {
        log.warn(format!(
            "Servers folder not found | path={}",
            folder.display()
        ));
        return Err(DiscoveryError::FolderNotFound {
            path: folder.to_path_buf(),
        });
    }

    if !folder.is_dir() {
        log.warn(format!(
            "Path is not a directory | path={}",
            folder.display()
        ));
        return Err(DiscoveryError::FolderNotFound {
            path: folder.to_path_buf(),
        });
    }

    let entries = std::fs::read_dir(folder).map_err(|e| {
        log.warn(format!(
            "Failed to read servers folder | path={} error={}",
            folder.display(),
            e
        ));
        DiscoveryError::ReadError { source: e }
    })?;

    let mut servers = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                log.warn(format!(
                    "Failed to read directory entry, skipping | error={}",
                    e
                ));
                continue;
            }
        };

        let path = entry.path();
        log.debug(format!("Checking file | path={}", path.display()));

        if path.is_dir() {
            log.debug(format!("Skipping directory | path={}", path.display()));
            continue;
        }

        if !is_executable(&path) {
            log.debug(format!(
                "Skipping non-executable file | path={}",
                path.display()
            ));
            continue;
        }

        let name = extract_server_name(&path);
        log.debug(format!(
            "Found MCP server binary | name={} path={}",
            name,
            path.display()
        ));

        servers.push(DiscoveredServer::new(name, path));
    }

    log.info(format!(
        "Server scan complete | count={} path={}",
        servers.len(),
        folder.display()
    ));

    Ok(servers)
}

/// Extract a server name from a binary path.
pub fn extract_server_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Check if a file is executable.
#[cfg(target_os = "windows")]
pub fn is_executable(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    matches!(ext.as_deref(), Some("exe") | Some("cmd") | Some("bat"))
}

#[cfg(all(unix, not(target_os = "windows")))]
pub fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = path.metadata() {
        let mode = metadata.permissions().mode();
        mode & 0o111 != 0
    } else {
        false
    }
}

#[cfg(not(any(target_os = "windows", unix)))]
pub fn is_executable(_path: &Path) -> bool {
    false
}
