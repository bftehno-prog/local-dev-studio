use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{HostingCompatibilityReport, Project, ProjectDoctorCheck};

pub(crate) fn build_hosting_compatibility_report(
    project: &Project,
) -> Result<HostingCompatibilityReport, String> {
    let root = PathBuf::from(&project.path);
    let files = collect_hosting_text_files(&root, 500)?;
    let checks = vec![
        hosting_check(
            "Hosting project type",
            matches!(project.project_type.as_str(), "php" | "static"),
            "Project type is suitable for shared hosting checks.",
            "Shared hosting checks are focused on PHP and static projects.",
        ),
        hosting_check(
            "Entrypoint",
            root.join("index.php").is_file()
                || root.join("public").join("index.php").is_file()
                || root.join("index.html").is_file(),
            "index.php or index.html found.",
            "No index.php or index.html entrypoint found.",
        ),
        hosting_check(
            "Scannable files",
            !files.is_empty(),
            "Text files were found for compatibility scan.",
            "No text files were found for compatibility scan.",
        ),
        hosting_check(
            "Absolute Windows paths",
            !files
                .iter()
                .any(|(_, content)| contains_windows_path(content)),
            "No absolute Windows paths found.",
            "Absolute Windows paths found. Replace them with relative or server paths.",
        ),
        hosting_check(
            "localhost URLs",
            !files.iter().any(|(_, content)| contains_localhost(content)),
            "No localhost URLs found.",
            "localhost URLs found. Replace them before upload.",
        ),
        hosting_check(
            "Mixed content",
            !files.iter().any(|(_, content)| content.contains("http://")),
            "No http:// references found.",
            "http:// references found. Check mixed-content risk for HTTPS hosting.",
        ),
        hosting_check(
            ".htaccess",
            root.join(".htaccess").is_file() || project.project_type != "php",
            ".htaccess is present or not required for this project type.",
            ".htaccess is missing. Confirm rewrite rules if this PHP project needs routing.",
        ),
        hosting_check(
            "Writable folders",
            root.join("uploads").is_dir()
                || root.join("storage").is_dir()
                || project.project_type != "php",
            "Writable folders look present or not required.",
            "No common writable folder found. Check uploads/storage needs.",
        ),
    ];
    Ok(HostingCompatibilityReport {
        project_id: project.id.clone(),
        project_name: project.name.clone(),
        checks,
    })
}

fn hosting_check(label: &str, ok: bool, success: &str, failure: &str) -> ProjectDoctorCheck {
    ProjectDoctorCheck {
        label: label.to_string(),
        status: if ok { "ok" } else { "warning" }.to_string(),
        message: if ok { success } else { failure }.to_string(),
    }
}

fn collect_hosting_text_files(
    root: &Path,
    max_files: usize,
) -> Result<Vec<(PathBuf, String)>, String> {
    fn walk(
        dir: &Path,
        files: &mut Vec<(PathBuf, String)>,
        max_files: usize,
    ) -> Result<(), String> {
        if files.len() >= max_files {
            return Ok(());
        }
        for entry in fs::read_dir(dir).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            if name == "node_modules" || name == ".git" || name == "vendor" || name == "dist" {
                continue;
            }
            if path.is_dir() {
                walk(&path, files, max_files)?;
            } else if is_hosting_text_file(&path) {
                if let Ok(content) = fs::read_to_string(&path) {
                    files.push((path, content));
                }
            }
            if files.len() >= max_files {
                break;
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    walk(root, &mut files, max_files)?;
    Ok(files)
}

fn is_hosting_text_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default(),
        "html" | "htm" | "php" | "css" | "js" | "json" | "env" | "txt" | "md" | "htaccess"
    ) || path.file_name().and_then(|value| value.to_str()) == Some(".htaccess")
}

pub(crate) fn contains_windows_path(content: &str) -> bool {
    content.as_bytes().windows(3).any(|window| {
        window[0].is_ascii_alphabetic()
            && window[1] == b':'
            && (window[2] == b'\\' || window[2] == b'/')
    })
}

pub(crate) fn contains_localhost(content: &str) -> bool {
    let lower = content.to_lowercase();
    lower.contains("localhost") || lower.contains("127.0.0.1")
}
