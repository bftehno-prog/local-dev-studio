use std::{path::PathBuf, process::Command};

use crate::security::validation::validate_project_type;
use crate::services::project_detector::package_json_has_script;
use crate::services::runtime_resolver::resolve_runtime;
use crate::utils::network::is_port_free;
use crate::{Project, ProjectDoctorCheck, ProjectDoctorReport, Settings};

pub(crate) fn build_project_doctor_report(
    project: &Project,
    settings: &Settings,
    package_manager: &str,
) -> ProjectDoctorReport {
    let root = PathBuf::from(&project.path);
    let mut checks = Vec::new();
    checks.push(doctor_check(
        "Project path",
        root.is_dir(),
        "Project directory is available.",
        "Project directory is missing.",
    ));
    checks.push(doctor_check(
        "Project type",
        validate_project_type(&project.project_type).is_ok(),
        "Project type is supported.",
        "Project type is unsupported.",
    ));
    checks.push(doctor_check(
        "Trusted project",
        project.trusted,
        "Project is trusted.",
        "Project is not trusted yet.",
    ));
    if let Some(port) = project.port {
        checks.push(doctor_check(
            "Configured port",
            is_port_free(port),
            &format!("Port {} is free.", port),
            &format!("Port {} is occupied.", port),
        ));
    }
    match project.project_type.as_str() {
        "next" | "vite" | "astro" => {
            let package_json = root.join("package.json");
            checks.push(doctor_check(
                "package.json",
                package_json.is_file(),
                "package.json found.",
                "package.json is missing.",
            ));
            checks.push(doctor_check(
                "dev script",
                package_json_has_script(&package_json, "dev"),
                "dev script found.",
                "No dev script found in package.json.",
            ));
            checks.push(doctor_check(
                "node_modules",
                root.join("node_modules").is_dir(),
                "node_modules exists.",
                "node_modules is missing.",
            ));
            checks.push(doctor_check(
                "package manager",
                Command::new(resolve_runtime(package_manager, settings))
                    .arg(if package_manager == "bun" {
                        "--version"
                    } else {
                        "-v"
                    })
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false),
                &format!("{} is available.", package_manager),
                &format!("{} is not available.", package_manager),
            ));
            checks.push(doctor_check(
                "Node.js",
                Command::new(resolve_runtime("node", settings))
                    .arg("-v")
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false),
                "Node.js is available.",
                "Node.js is not available.",
            ));
        }
        "php" => {
            checks.push(doctor_check(
                "index.php",
                root.join("index.php").is_file() || root.join("public").join("index.php").is_file(),
                "PHP entrypoint found.",
                "index.php was not found.",
            ));
            checks.push(doctor_check(
                "PHP",
                Command::new(resolve_runtime("php", settings))
                    .arg("-v")
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false),
                "PHP is available.",
                "PHP is not available.",
            ));
        }
        "static" => {
            checks.push(doctor_check(
                "index.html",
                root.join("index.html").is_file(),
                "index.html found.",
                "index.html was not found.",
            ));
        }
        _ => {}
    }
    ProjectDoctorReport {
        project_id: project.id.clone(),
        project_name: project.name.clone(),
        checks,
    }
}

fn doctor_check(label: &str, ok: bool, success: &str, failure: &str) -> ProjectDoctorCheck {
    ProjectDoctorCheck {
        label: label.to_string(),
        status: if ok { "ok" } else { "warning" }.to_string(),
        message: if ok { success } else { failure }.to_string(),
    }
}
