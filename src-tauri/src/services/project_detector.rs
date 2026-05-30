use std::{fs, path::Path};

pub(crate) fn detect_project_type_at(root: &Path) -> Result<String, String> {
    let package_json = root.join("package.json");
    let package = fs::read_to_string(&package_json).unwrap_or_default();
    let has_dep = |name: &str| package.contains(&format!("\"{}\"", name));
    if has_dep("next")
        || root.join("next.config.js").exists()
        || root.join("next.config.mjs").exists()
        || root.join("next.config.ts").exists()
        || root.join("app").exists()
        || root.join("pages").exists()
    {
        return Ok("next".to_string());
    }
    if has_dep("vite")
        || root.join("vite.config.js").exists()
        || root.join("vite.config.ts").exists()
    {
        return Ok("vite".to_string());
    }
    if has_dep("astro") || root.join("astro.config.mjs").exists() {
        return Ok("astro".to_string());
    }
    if root.join("index.php").exists() || root.join("composer.json").exists() {
        return Ok("php".to_string());
    }
    if root.join("index.html").exists()
        || root.join("assets").exists()
        || root.join("css").exists()
        || root.join("js").exists()
    {
        return Ok("static".to_string());
    }
    if package_json.is_file() && package_json_has_script(&package_json, "dev") {
        return Ok("node".to_string());
    }
    Ok("unknown".to_string())
}

pub(crate) fn package_json_has_script(path: &Path, script: &str) -> bool {
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    value
        .get("scripts")
        .and_then(|scripts| scripts.get(script))
        .and_then(|script| script.as_str())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

pub(crate) fn ensure_node_modules(project_path: &str, package_manager: &str) -> Result<(), String> {
    if Path::new(project_path).join("node_modules").is_dir() {
        return Ok(());
    }
    let lockfile = detect_lockfile(Path::new(project_path)).unwrap_or_else(|| {
        format!(
            "{} lockfile",
            match package_manager {
                "npm" => "npm",
                "yarn" => "Yarn",
                "bun" => "Bun",
                _ => "pnpm",
            }
        )
    });
    Err(format!(
        "Dependencies are missing. Detected {}. Run Install dependencies before starting this project.",
        lockfile
    ))
}

pub(crate) fn detect_lockfile(project_path: &Path) -> Option<String> {
    [
        ("pnpm-lock.yaml", "pnpm-lock.yaml"),
        ("package-lock.json", "package-lock.json"),
        ("yarn.lock", "yarn.lock"),
        ("bun.lockb", "bun.lockb"),
        ("bun.lock", "bun.lock"),
    ]
    .iter()
    .find(|(file, _)| project_path.join(file).is_file())
    .map(|(_, label)| (*label).to_string())
}
