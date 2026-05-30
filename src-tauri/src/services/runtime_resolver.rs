use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use crate::models::Settings;

pub(crate) fn runtime_version_args(name: &str) -> Result<&'static [&'static str], String> {
    match name {
        "node" | "npm" | "pnpm" | "yarn" => Ok(&["-v"]),
        "bun" => Ok(&["--version"]),
        "php" => Ok(&["-v"]),
        "git" => Ok(&["--version"]),
        _ => Err("Unsupported runtime.".to_string()),
    }
}

pub(crate) fn resolve_runtime(name: &str, settings: &Settings) -> String {
    let configured = match name {
        "node" => &settings.node_path,
        "npm" => &settings.npm_path,
        "pnpm" => &settings.pnpm_path,
        "yarn" => &settings.yarn_path,
        "bun" => &settings.bun_path,
        "php" => &settings.php_path,
        "git" => &settings.git_path,
        _ => "",
    };
    if !configured.trim().is_empty() {
        return configured.to_string();
    }
    if settings.use_bundled_node {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(root) = exe.parent().and_then(|p| p.parent()) {
                let candidate = root.join("binaries").join(match name {
                    "node" => "node.exe",
                    "pnpm" => "pnpm.cmd",
                    "npm" => "npm.cmd",
                    "yarn" => "yarn.cmd",
                    "bun" => "bun.exe",
                    other => other,
                });
                if candidate.exists() {
                    return candidate.to_string_lossy().to_string();
                }
            }
        }
    }
    for candidate in runtime_candidates(name) {
        if candidate.exists() {
            return candidate.to_string_lossy().to_string();
        }
    }
    name.to_string()
}

pub(crate) fn runtime_source(name: &str, settings: &Settings, resolved: &str) -> String {
    let configured = match name {
        "node" => &settings.node_path,
        "npm" => &settings.npm_path,
        "pnpm" => &settings.pnpm_path,
        "yarn" => &settings.yarn_path,
        "bun" => &settings.bun_path,
        "php" => &settings.php_path,
        "git" => &settings.git_path,
        _ => "",
    };
    if !configured.trim().is_empty() {
        return "custom".to_string();
    }
    if settings.use_bundled_node && Path::new(resolved).exists() {
        return "bundled".to_string();
    }
    "system".to_string()
}

fn runtime_candidates(name: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if cfg!(windows) {
        if let Some(appdata) = env::var_os("APPDATA") {
            let npm_dir = PathBuf::from(appdata).join("npm");
            match name {
                "pnpm" => candidates.push(npm_dir.join("pnpm.cmd")),
                "npm" => candidates.push(npm_dir.join("npm.cmd")),
                "yarn" => candidates.push(npm_dir.join("yarn.cmd")),
                "bun" => candidates.push(npm_dir.join("bun.cmd")),
                _ => {}
            }
        }
        if let Some(program_files) = env::var_os("ProgramFiles") {
            let node_dir = PathBuf::from(program_files).join("nodejs");
            match name {
                "node" => candidates.push(node_dir.join("node.exe")),
                "npm" => candidates.push(node_dir.join("npm.cmd")),
                "pnpm" => candidates.push(node_dir.join("pnpm.cmd")),
                "yarn" => candidates.push(node_dir.join("yarn.cmd")),
                _ => {}
            }
        }
        if let Some(local_appdata) = env::var_os("LOCALAPPDATA") {
            if name == "bun" {
                candidates.push(PathBuf::from(local_appdata).join("bun").join("bun.exe"));
            }
        }
    }
    candidates
}

pub(crate) fn runtime_path(settings: &Settings) -> String {
    let mut paths = Vec::new();
    for runtime in ["pnpm", "npm", "yarn", "bun", "node"] {
        let resolved = resolve_runtime(runtime, settings);
        let path = PathBuf::from(resolved);
        if let Some(parent) = path.parent() {
            paths.push(parent.to_path_buf());
        }
    }
    if let Some(path) = env::var_os("PATH") {
        paths.extend(env::split_paths(&path));
    }
    env::join_paths(paths)
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

pub(crate) fn version_for(program: String, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("Ready")
                .to_string()
        })
        .unwrap_or_else(|| "Not found".to_string())
}
