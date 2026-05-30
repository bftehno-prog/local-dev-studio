use std::{fs, path::Path};

pub(crate) fn clear_cache_at(root: &Path) -> Result<(), String> {
    for folder in [".next", "node_modules/.cache", ".turbo"] {
        let target = root.join(folder);
        if target.exists() {
            ensure_child_path(root, &target)?;
            fs::remove_dir_all(&target)
                .map_err(|error| format!("Failed to remove {}: {}", target.display(), error))?;
        }
    }
    Ok(())
}

fn ensure_child_path(root: &Path, target: &Path) -> Result<(), String> {
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let target_parent = target.parent().unwrap_or(&root);
    let target_parent = target_parent.canonicalize().unwrap_or(root.clone());
    if !target_parent.starts_with(&root) {
        return Err("Refusing to delete a path outside the project folder.".to_string());
    }
    Ok(())
}
