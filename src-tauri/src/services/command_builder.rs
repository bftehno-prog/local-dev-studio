use std::path::Path;

use crate::models::{Project, Settings};
use crate::services::project_detector::{ensure_node_modules, package_json_has_script};
use crate::services::runtime_resolver::resolve_runtime;

pub(crate) struct CommandSpec {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) display: String,
}

pub(crate) fn build_command(
    project: &Project,
    settings: &Settings,
    port: u16,
) -> Result<CommandSpec, String> {
    match project.project_type.as_str() {
        "next" => {
            ensure_required_file(
                &Path::new(&project.path).join("package.json"),
                "package.json not found. Next.js projects must have package.json in the root.",
            )?;
            let package_manager = project_package_manager(project, settings);
            let program = resolve_runtime(package_manager.as_str(), settings);
            let mut args = package_runner_args(package_manager.as_str(), "next", &["dev"]);
            if project.use_turbopack || settings.use_turbopack {
                args.push("--turbopack".to_string());
            }
            args.extend(
                ["-H", "0.0.0.0", "-p", &port.to_string()]
                    .iter()
                    .map(|value| value.to_string()),
            );
            ensure_node_modules(&project.path, package_manager.as_str())?;
            Ok(CommandSpec {
                display: format!("{} {}", program, args.join(" ")),
                program,
                args,
            })
        }
        "vite" | "astro" | "node" => {
            let package_json = Path::new(&project.path).join("package.json");
            ensure_required_file(
                &package_json,
                "package.json not found. Vite/Astro/Node projects must have package.json in the root.",
            )?;
            if !package_json_has_script(&package_json, "dev") {
                return Err(
                    "No dev script found in package.json. Add a dev script before starting this project."
                        .to_string(),
                );
            }
            let package_manager = project_package_manager(project, settings);
            let program = resolve_runtime(package_manager.as_str(), settings);
            let mut args = package_dev_args(package_manager.as_str());
            args.extend(
                ["--host", "0.0.0.0", "--port", &port.to_string()]
                    .iter()
                    .map(|value| value.to_string()),
            );
            ensure_node_modules(&project.path, package_manager.as_str())?;
            Ok(CommandSpec {
                display: format!("{} {}", program, args.join(" ")),
                program,
                args,
            })
        }
        "php" => {
            let php = resolve_runtime("php", settings);
            let args = vec![
                "-S".into(),
                format!("0.0.0.0:{}", port),
                "-t".into(),
                project.path.clone(),
            ];
            Ok(CommandSpec {
                display: format!("{} {}", php, args.join(" ")),
                program: php,
                args,
            })
        }
        "static" => {
            let node = resolve_runtime("node", settings);
            let script = "const http=require('http'),fs=require('fs'),path=require('path');const root=path.resolve(process.cwd());const mime={'.html':'text/html','.css':'text/css','.js':'text/javascript','.json':'application/json','.png':'image/png','.jpg':'image/jpeg','.svg':'image/svg+xml'};http.createServer((req,res)=>{let p=decodeURIComponent(req.url.split('?')[0]);if(p==='/' )p='/index.html';let f=path.resolve(root,'.'+p);if(!f.startsWith(root+path.sep)&&f!==root){res.writeHead(403);return res.end('Forbidden')}fs.readFile(f,(e,d)=>{if(e){res.writeHead(404);res.end('Not found')}else{res.writeHead(200,{'Content-Type':mime[path.extname(f)]||'application/octet-stream'});res.end(d)}})}).listen(PORT,'0.0.0.0');";
            let args = vec!["-e".into(), script.replace("PORT", &port.to_string())];
            Ok(CommandSpec {
                display: format!("{} -e <static-server> port {}", node, port),
                program: node,
                args,
            })
        }
        _ => Err(
            "Unknown project type. Add package.json, index.html, index.php or a supported config file."
                .to_string(),
        ),
    }
}

pub(crate) fn package_manager(settings: &Settings) -> String {
    match settings.package_manager.trim().to_lowercase().as_str() {
        "npm" => "npm".to_string(),
        "yarn" => "yarn".to_string(),
        "bun" => "bun".to_string(),
        _ => "pnpm".to_string(),
    }
}

fn project_package_manager(project: &Project, settings: &Settings) -> String {
    project
        .package_manager
        .as_deref()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| matches!(value.as_str(), "npm" | "pnpm" | "yarn" | "bun"))
        .unwrap_or_else(|| package_manager(settings))
}

pub(crate) fn parse_environment_variables(raw: &str) -> Result<Vec<(String, String)>, String> {
    let mut vars = Vec::new();
    for (index, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!(
                "Invalid environment variable on line {}. Use KEY=value.",
                index + 1
            ));
        };
        let key = key.trim();
        if !is_valid_env_key(key) {
            return Err(format!(
                "Invalid environment variable name on line {}.",
                index + 1
            ));
        }
        let value = value.trim();
        if value.chars().any(|ch| ch.is_control() && ch != '\t') {
            return Err(format!(
                "Invalid environment variable value on line {}.",
                index + 1
            ));
        }
        vars.push((key.to_string(), value.to_string()));
    }
    Ok(vars)
}

fn ensure_required_file(path: &Path, message: &str) -> Result<(), String> {
    if path.exists() {
        Ok(())
    } else {
        Err(message.to_string())
    }
}

fn package_runner_args(manager: &str, binary: &str, base_args: &[&str]) -> Vec<String> {
    match manager {
        "npm" => ["exec", binary, "--"]
            .iter()
            .chain(base_args.iter())
            .map(|value| value.to_string())
            .collect(),
        "yarn" => [binary]
            .iter()
            .chain(base_args.iter())
            .map(|value| value.to_string())
            .collect(),
        "bun" => ["x", binary]
            .iter()
            .chain(base_args.iter())
            .map(|value| value.to_string())
            .collect(),
        _ => [binary]
            .iter()
            .chain(base_args.iter())
            .map(|value| value.to_string())
            .collect(),
    }
}

fn package_dev_args(manager: &str) -> Vec<String> {
    match manager {
        "npm" => vec!["run".into(), "dev".into(), "--".into()],
        "yarn" => vec!["dev".into()],
        "bun" => vec!["run".into(), "dev".into(), "--".into()],
        _ => vec!["dev".into(), "--".into()],
    }
}

fn is_valid_env_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_uppercase()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_uppercase() || ch.is_ascii_digit())
}
