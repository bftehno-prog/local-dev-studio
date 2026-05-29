use std::path::{Component, Path};

const ALLOWED_PROJECT_TYPES: &[&str] = &["next", "vite", "astro", "php", "static"];
const ALLOWED_PACKAGE_MANAGERS: &[&str] = &["npm", "pnpm", "yarn", "bun"];

pub(crate) fn validate_project_path(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("Project path is empty.".to_string());
    }
    let raw = path.to_string_lossy();
    if raw.contains('\0') {
        return Err("Project path contains invalid characters.".to_string());
    }
    if path.components().any(|component| matches!(component, Component::ParentDir)) {
        return Err("Project path must not contain parent-directory traversal.".to_string());
    }
    if !path.exists() {
        return Err("Project folder does not exist. Check the path and try again.".to_string());
    }
    if !path.is_dir() {
        return Err("Selected path is not a folder.".to_string());
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("Cannot resolve project path: {}", error))?;
    reject_system_directory(&canonical)?;
    Ok(())
}

pub(crate) fn validate_project_type(project_type: &str) -> Result<(), String> {
    if ALLOWED_PROJECT_TYPES.contains(&project_type) {
        Ok(())
    } else {
        Err("Unsupported project type. Supported types: next, vite, astro, php, static.".to_string())
    }
}

pub(crate) fn validate_package_manager(package_manager: &str) -> Result<(), String> {
    let normalized = package_manager.trim().to_lowercase();
    if ALLOWED_PACKAGE_MANAGERS.contains(&normalized.as_str()) {
        Ok(())
    } else {
        Err("Unsupported package manager. Supported package managers: npm, pnpm, yarn, bun.".to_string())
    }
}

fn reject_system_directory(path: &Path) -> Result<(), String> {
    if !cfg!(windows) {
        return Ok(());
    }
    let normalized = path.to_string_lossy().replace('/', "\\").to_lowercase();
    let system_roots = [
        "c:\\windows",
        "c:\\program files",
        "c:\\program files (x86)",
        "c:\\programdata\\microsoft",
    ];
    if system_roots.iter().any(|root| normalized == *root || normalized.starts_with(&format!("{}\\", root))) {
        return Err("Projects cannot be launched from Windows system folders.".to_string());
    }
    Ok(())
}
