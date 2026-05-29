use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use sysinfo::{Pid, System};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State,
};
use uuid::Uuid;

mod db;
mod security;
mod state;
mod utils;

use security::validation::{
    is_allowed_project_type, validate_package_manager, validate_project_path, validate_project_type,
};
use state::{AppState, ManagedProcesses};
use utils::{
    network::{find_free_port, is_port_free, network_url as build_network_url},
    paths::default_data_dir,
    time::now,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Project {
    id: String,
    name: String,
    path: String,
    project_type: String,
    port: Option<u16>,
    command: Option<String>,
    status: String,
    use_turbopack: bool,
    trusted: bool,
    trusted_at: Option<String>,
    trusted_runtime: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Settings {
    #[serde(default = "default_language")]
    language: String,
    #[serde(default)]
    onboarding_completed: bool,
    projects_folder: String,
    sandboxes_folder: String,
    package_manager: String,
    port_start: u16,
    port_end: u16,
    open_preview_automatically: bool,
    start_minimized: bool,
    launch_on_startup: bool,
    use_bundled_node: bool,
    node_path: String,
    npm_path: String,
    pnpm_path: String,
    yarn_path: String,
    bun_path: String,
    php_path: String,
    git_path: String,
    use_turbopack: bool,
    clear_next_before_start: bool,
    enable_network_preview: bool,
    enable_https: bool,
    default_next_port: u16,
    default_device: String,
    desktop_width: u16,
    laptop_width: u16,
    tablet_width: u16,
    mobile_width: u16,
    custom_width: u16,
    auto_reload_preview: bool,
    open_external_browser_on_start: bool,
    environment_variables: String,
    hosts: String,
    ssl_certificates: String,
    proxy_rules: String,
    process_timeout: u32,
    log_retention: u32,
}

#[derive(Debug, Clone, Serialize)]
struct DashboardData {
    running_projects: usize,
    stopped_projects: usize,
    used_ports: Vec<u16>,
    node_version: String,
    npm_version: String,
    pnpm_version: String,
    git_version: String,
    php_version: String,
    runtime_status: String,
    recent_errors: Vec<LogEntry>,
    recent_projects: Vec<Project>,
}

#[derive(Debug, Clone, Serialize)]
struct ServerProcess {
    project_id: String,
    project_name: String,
    project_type: String,
    pid: u32,
    port: u16,
    url: String,
    network_url: String,
    status: String,
    command: String,
    cwd: String,
    started_at: String,
    memory_usage_mb: Option<f32>,
    cpu_usage: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
struct LogEntry {
    id: i64,
    project_id: Option<String>,
    project_name: Option<String>,
    level: String,
    message: String,
    created_at: String,
}

#[derive(Debug, Clone, Serialize)]
struct PortInfo {
    port: u16,
    available: bool,
    pid: Option<u32>,
    project_id: Option<String>,
    project_name: Option<String>,
    external: bool,
}

#[derive(Debug, Clone, Serialize)]
struct TemplateInfo {
    id: String,
    name: String,
    project_type: String,
    built_in: bool,
    path: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct TemplateManifest {
    name: String,
    #[serde(rename = "type")]
    project_type: String,
    version: Option<String>,
    author: Option<String>,
    description: Option<String>,
    #[serde(rename = "defaultPort")]
    default_port: Option<u16>,
    #[serde(rename = "packageManager")]
    package_manager: Option<String>,
    #[serde(rename = "requiresInstall")]
    requires_install: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
struct DiagnosticItem {
    name: String,
    status: String,
    version: String,
    path: String,
    error: String,
}

#[derive(Debug, Clone, Serialize)]
struct RuntimeInfo {
    name: String,
    found: bool,
    version: Option<String>,
    path: Option<String>,
    source: String,
    last_checked_at: String,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ProjectDoctorCheck {
    label: String,
    status: String,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct ProjectDoctorReport {
    project_id: String,
    project_name: String,
    checks: Vec<ProjectDoctorCheck>,
}

#[derive(Debug, Clone, Serialize)]
struct HostingCompatibilityReport {
    project_id: String,
    project_name: String,
    checks: Vec<ProjectDoctorCheck>,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db_path = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| default_data_dir())
                .join("local-dev-studio.sqlite");
            if let Some(parent) = db_path.parent() {
                fs::create_dir_all(parent)?;
            }
            db::migrations::run_migrations(&db_path).map_err(|error| error.to_string())?;
            seed_defaults(&db_path).map_err(|error| error.to_string())?;
            app.manage(AppState {
                db_path: db_path.clone(),
                processes: Arc::new(Mutex::new(ManagedProcesses::default())),
            });
            create_tray(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            dashboard,
            list_projects,
            add_project,
            remove_project,
            start_project,
            stop_project,
            trust_project,
            reset_project_trust,
            start_all_projects,
            stop_all_projects,
            open_path,
            open_in_code,
            open_external_url,
            clear_project_cache,
            network_url,
            list_servers,
            list_ports,
            release_port,
            list_logs,
            clear_logs,
            export_logs,
            get_settings,
            save_settings,
            list_templates,
            create_from_template,
            create_sandbox,
            duplicate_template,
            delete_template,
            import_template_zip,
            export_template_zip,
            detect_project_type,
            check_runtime,
            check_all_runtimes,
            project_doctor,
            hosting_compatibility_check,
            diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("error while running Local Dev Studio");
}

fn connect(db_path: &Path) -> Result<Connection, String> {
    Connection::open(db_path).map_err(|error| error.to_string())
}

#[allow(dead_code)]
fn init_database(db_path: &Path) -> Result<(), String> {
    let conn = connect(db_path)?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            project_type TEXT NOT NULL,
            port INTEGER,
            command TEXT,
            status TEXT NOT NULL,
            use_turbopack INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS processes (
            project_id TEXT PRIMARY KEY,
            pid INTEGER NOT NULL,
            command TEXT NOT NULL,
            cwd TEXT NOT NULL,
            port INTEGER NOT NULL,
            started_at TEXT NOT NULL,
            status TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id TEXT,
            level TEXT NOT NULL,
            message TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS templates (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            project_type TEXT NOT NULL,
            built_in INTEGER NOT NULL,
            path TEXT
        );
        CREATE TABLE IF NOT EXISTS sandboxes (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            template_id TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS ports (
            port INTEGER PRIMARY KEY,
            project_id TEXT,
            pid INTEGER,
            status TEXT NOT NULL
        );
        ",
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn seed_defaults(db_path: &Path) -> Result<(), String> {
    let conn = connect(db_path)?;
    let defaults = default_settings();
    let value = serde_json::to_string(&defaults).map_err(|error| error.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('settings', ?1)",
        params![value],
    )
    .map_err(|error| error.to_string())?;
    for (id, name, project_type) in [
        ("next-app-router", "Next.js App Router", "next"),
        ("next-tailwind", "Next.js + Tailwind", "next"),
        ("vite-react", "Vite React", "vite"),
        ("static-html", "Static HTML/CSS/JS", "static"),
        ("php-template", "PHP Template", "php"),
    ] {
        conn.execute(
            "INSERT OR IGNORE INTO templates (id, name, project_type, built_in, path) VALUES (?1, ?2, ?3, 1, NULL)",
            params![id, name, project_type],
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn default_settings() -> Settings {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("C:\\"));
    let base = home.join("LocalDevStudio");
    Settings {
        language: "ru".to_string(),
        onboarding_completed: false,
        projects_folder: base.join("projects").to_string_lossy().to_string(),
        sandboxes_folder: base.join("sandboxes").to_string_lossy().to_string(),
        package_manager: "pnpm".to_string(),
        port_start: 3000,
        port_end: 3999,
        open_preview_automatically: true,
        start_minimized: false,
        launch_on_startup: false,
        use_bundled_node: true,
        node_path: "".to_string(),
        npm_path: "".to_string(),
        pnpm_path: "".to_string(),
        yarn_path: "".to_string(),
        bun_path: "".to_string(),
        php_path: "".to_string(),
        git_path: "".to_string(),
        use_turbopack: false,
        clear_next_before_start: false,
        enable_network_preview: true,
        enable_https: false,
        default_next_port: 3000,
        default_device: "Desktop".to_string(),
        desktop_width: 1440,
        laptop_width: 1280,
        tablet_width: 768,
        mobile_width: 390,
        custom_width: 980,
        auto_reload_preview: true,
        open_external_browser_on_start: false,
        environment_variables: "".to_string(),
        hosts: "".to_string(),
        ssl_certificates: "".to_string(),
        proxy_rules: "".to_string(),
        process_timeout: 60,
        log_retention: 14,
    }
}

fn default_language() -> String {
    "ru".to_string()
}

fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let language = app
        .path()
        .app_data_dir()
        .ok()
        .and_then(|path| read_settings_at(&path.join("local-dev-studio.sqlite")).ok())
        .map(|settings| settings.language)
        .unwrap_or_else(default_language);
    let open = MenuItem::with_id(
        app,
        "open",
        tray_label(&language, "open"),
        true,
        None::<&str>,
    )?;
    let start_all = MenuItem::with_id(
        app,
        "start_all",
        tray_label(&language, "start_all"),
        true,
        None::<&str>,
    )?;
    let stop_all = MenuItem::with_id(
        app,
        "stop_all",
        tray_label(&language, "stop_all"),
        true,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(
        app,
        "settings",
        tray_label(&language, "settings"),
        true,
        None::<&str>,
    )?;
    let exit = MenuItem::with_id(
        app,
        "exit",
        tray_label(&language, "exit"),
        true,
        None::<&str>,
    )?;
    let menu = Menu::with_items(app, &[&open, &start_all, &stop_all, &settings, &exit])?;
    TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" | "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "start_all" => {
                let state = app.state::<AppState>();
                let _ = start_all_projects_inner(&state);
            }
            "stop_all" => {
                let state = app.state::<AppState>();
                let _ = stop_all_projects_inner(&state);
            }
            "exit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}

fn tray_label(language: &str, key: &str) -> &'static str {
    match (language, key) {
        ("ru", "open") => "Открыть панель",
        ("ru", "start_all") => "Запустить все",
        ("ru", "stop_all") => "Остановить все",
        ("ru", "settings") => "Настройки",
        ("ru", "exit") => "Выход",
        (_, "open") => "Open Dashboard",
        (_, "start_all") => "Start All",
        (_, "stop_all") => "Stop All",
        (_, "settings") => "Settings",
        (_, "exit") => "Exit",
        _ => "Local Dev Studio",
    }
}

#[tauri::command]
fn dashboard(state: State<AppState>) -> Result<DashboardData, String> {
    let projects = list_projects(state.clone())?;
    let running_projects = projects
        .iter()
        .filter(|project| project.status == "running")
        .count();
    let stopped_projects = projects
        .iter()
        .filter(|project| project.status != "running")
        .count();
    let used_ports = list_ports(state.clone())?
        .into_iter()
        .filter(|port| !port.available)
        .map(|port| port.port)
        .collect::<Vec<_>>();
    let settings = get_settings(state.clone())?;
    let node_version = version_for(resolve_runtime("node", &settings), &["-v"]);
    let npm_version = version_for(resolve_runtime("npm", &settings), &["-v"]);
    let pnpm_version = version_for(resolve_runtime("pnpm", &settings), &["-v"]);
    let git_version = version_for(resolve_runtime("git", &settings), &["--version"]);
    let php_version = version_for(resolve_runtime("php", &settings), &["-v"]);
    let runtime_status = if node_version != "Not found" {
        "Ready"
    } else {
        "Node.js not found"
    }
    .to_string();
    let recent_errors = list_logs(
        Some("".to_string()),
        Some("error".to_string()),
        None,
        state.clone(),
    )?
    .into_iter()
    .take(5)
    .collect();
    let recent_projects = projects.into_iter().take(5).collect();
    Ok(DashboardData {
        running_projects,
        stopped_projects,
        used_ports,
        node_version,
        npm_version,
        pnpm_version,
        git_version,
        php_version,
        runtime_status,
        recent_errors,
        recent_projects,
    })
}

#[tauri::command]
fn diagnostics(state: State<AppState>) -> Result<Vec<DiagnosticItem>, String> {
    let settings = get_settings(state.clone())?;
    let mut items = Vec::new();
    for (name, args) in [
        ("node", vec!["-v"]),
        ("npm", vec!["-v"]),
        ("pnpm", vec!["-v"]),
        ("yarn", vec!["-v"]),
        ("bun", vec!["--version"]),
        ("git", vec!["--version"]),
        ("php", vec!["-v"]),
        ("cargo", vec!["--version"]),
    ] {
        items.push(runtime_diagnostic(name, &settings, &args));
    }
    items.push(path_diagnostic(
        "PATH",
        env::var("PATH").unwrap_or_default(),
        true,
    ));
    items.push(path_diagnostic(
        "Projects folder",
        settings.projects_folder.clone(),
        Path::new(&settings.projects_folder).is_dir(),
    ));
    items.push(path_diagnostic(
        "Sandboxes folder",
        settings.sandboxes_folder.clone(),
        Path::new(&settings.sandboxes_folder).is_dir(),
    ));
    items.push(path_diagnostic(
        "SQLite data",
        state.db_path.to_string_lossy().to_string(),
        state.db_path.exists(),
    ));
    let data_parent = state
        .db_path
        .parent()
        .map(|path| path.is_dir())
        .unwrap_or(false);
    items.push(path_diagnostic(
        "App data folder",
        state
            .db_path
            .parent()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
        data_parent,
    ));
    Ok(items)
}

#[tauri::command]
fn check_runtime(name: String, state: State<AppState>) -> Result<RuntimeInfo, String> {
    let settings = get_settings(state)?;
    runtime_info(&name, &settings)
}

#[tauri::command]
fn check_all_runtimes(state: State<AppState>) -> Result<Vec<RuntimeInfo>, String> {
    let settings = get_settings(state)?;
    ["node", "npm", "pnpm", "yarn", "bun", "php", "git"]
        .into_iter()
        .map(|name| runtime_info(name, &settings))
        .collect()
}

fn runtime_info(name: &str, settings: &Settings) -> Result<RuntimeInfo, String> {
    let args = runtime_version_args(name)?;
    let program = resolve_runtime(name, settings);
    let source = runtime_source(name, settings, &program);
    match Command::new(&program).args(args).output() {
        Ok(output) if output.status.success() => Ok(RuntimeInfo {
            name: name.to_string(),
            found: true,
            version: Some(
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("Ready")
                    .to_string(),
            ),
            path: Some(program),
            source,
            last_checked_at: now(),
            error: None,
        }),
        Ok(output) => Ok(RuntimeInfo {
            name: name.to_string(),
            found: false,
            version: None,
            path: Some(program),
            source,
            last_checked_at: now(),
            error: Some(
                String::from_utf8_lossy(&output.stderr)
                    .lines()
                    .next()
                    .unwrap_or("Runtime returned an error")
                    .to_string(),
            ),
        }),
        Err(error) => Ok(RuntimeInfo {
            name: name.to_string(),
            found: false,
            version: None,
            path: Some(program),
            source,
            last_checked_at: now(),
            error: Some(error.to_string()),
        }),
    }
}

fn runtime_version_args(name: &str) -> Result<&'static [&'static str], String> {
    match name {
        "node" | "npm" | "pnpm" | "yarn" => Ok(&["-v"]),
        "bun" => Ok(&["--version"]),
        "php" => Ok(&["-v"]),
        "git" => Ok(&["--version"]),
        _ => Err("Unsupported runtime.".to_string()),
    }
}

fn runtime_diagnostic(name: &str, settings: &Settings, args: &[&str]) -> DiagnosticItem {
    let program = resolve_runtime(name, settings);
    match Command::new(&program).args(args).output() {
        Ok(output) if output.status.success() => DiagnosticItem {
            name: name.to_string(),
            status: "OK".to_string(),
            version: String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("Ready")
                .to_string(),
            path: program,
            error: String::new(),
        },
        Ok(output) => DiagnosticItem {
            name: name.to_string(),
            status: "Warning".to_string(),
            version: String::new(),
            path: program,
            error: String::from_utf8_lossy(&output.stderr)
                .lines()
                .next()
                .unwrap_or("Command returned an error")
                .to_string(),
        },
        Err(error) => DiagnosticItem {
            name: name.to_string(),
            status: "Missing".to_string(),
            version: String::new(),
            path: program,
            error: error.to_string(),
        },
    }
}

fn path_diagnostic(name: &str, path: String, ok: bool) -> DiagnosticItem {
    DiagnosticItem {
        name: name.to_string(),
        status: if ok { "OK" } else { "Warning" }.to_string(),
        version: String::new(),
        path,
        error: if ok {
            String::new()
        } else {
            "Path is not available.".to_string()
        },
    }
}

#[tauri::command]
fn list_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    projects_from_db(&state.db_path)
}

fn projects_from_db(db_path: &Path) -> Result<Vec<Project>, String> {
    let conn = connect(db_path)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, path, project_type, port, command, status, use_turbopack, trusted, trusted_at, trusted_runtime, created_at, updated_at
             FROM projects ORDER BY updated_at DESC",
        )
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], project_from_row)
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn add_project(path: String, state: State<AppState>) -> Result<Project, String> {
    let project_path = PathBuf::from(path.trim());
    validate_project_path(&project_path)?;
    let project_type = detect_project_type(project_path.to_string_lossy().to_string())?;
    let timestamp = now();
    let name = project_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("Project")
        .to_string();
    let project = Project {
        id: format!("project_{}", Uuid::new_v4().simple()),
        name,
        path: project_path.to_string_lossy().to_string(),
        project_type,
        port: None,
        command: None,
        status: "stopped".to_string(),
        use_turbopack: false,
        trusted: false,
        trusted_at: None,
        trusted_runtime: None,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    };
    upsert_project(&state.db_path, &project)?;
    insert_log(&state.db_path, Some(&project.id), "info", "Project added")?;
    Ok(project)
}

#[tauri::command]
fn remove_project(id: String, state: State<AppState>) -> Result<(), String> {
    let _ = stop_project(id.clone(), state.clone());
    let conn = connect(&state.db_path)?;
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
fn trust_project(id: String, state: State<AppState>) -> Result<Project, String> {
    let mut project = get_project(&state.db_path, &id)?;
    let settings = read_settings_at(&state.db_path)?;
    project.trusted = true;
    project.trusted_at = Some(now());
    project.trusted_runtime = Some(trust_runtime_for_project(&project, &settings));
    project.updated_at = now();
    upsert_project(&state.db_path, &project)?;
    insert_log(
        &state.db_path,
        Some(&project.id),
        "warning",
        "Project marked as trusted by user",
    )?;
    Ok(project)
}

#[tauri::command]
fn reset_project_trust(id: String, state: State<AppState>) -> Result<Project, String> {
    let mut project = get_project(&state.db_path, &id)?;
    project.trusted = false;
    project.trusted_at = None;
    project.trusted_runtime = None;
    project.updated_at = now();
    upsert_project(&state.db_path, &project)?;
    insert_log(
        &state.db_path,
        Some(&project.id),
        "warning",
        "Project trust status reset",
    )?;
    Ok(project)
}

#[tauri::command]
fn project_doctor(id: String, state: State<AppState>) -> Result<ProjectDoctorReport, String> {
    let project = get_project(&state.db_path, &id)?;
    let settings = read_settings_at(&state.db_path)?;
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
            let manager = package_manager(&settings);
            checks.push(doctor_check(
                "package manager",
                Command::new(resolve_runtime(&manager, &settings))
                    .arg(if manager == "bun" { "--version" } else { "-v" })
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false),
                &format!("{} is available.", manager),
                &format!("{} is not available.", manager),
            ));
            checks.push(doctor_check(
                "Node.js",
                Command::new(resolve_runtime("node", &settings))
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
                Command::new(resolve_runtime("php", &settings))
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
    Ok(ProjectDoctorReport {
        project_id: project.id,
        project_name: project.name,
        checks,
    })
}

fn doctor_check(label: &str, ok: bool, success: &str, failure: &str) -> ProjectDoctorCheck {
    ProjectDoctorCheck {
        label: label.to_string(),
        status: if ok { "ok" } else { "warning" }.to_string(),
        message: if ok { success } else { failure }.to_string(),
    }
}

fn package_json_has_script(path: &Path, script: &str) -> bool {
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

#[tauri::command]
fn hosting_compatibility_check(
    id: String,
    state: State<AppState>,
) -> Result<HostingCompatibilityReport, String> {
    let project = get_project(&state.db_path, &id)?;
    validate_project_path(Path::new(&project.path))?;
    let root = PathBuf::from(&project.path);
    let files = collect_hosting_text_files(&root, 500)?;
    let checks = vec![
        doctor_check(
            "Hosting project type",
            matches!(project.project_type.as_str(), "php" | "static"),
            "Project type is suitable for shared hosting checks.",
            "Shared hosting checks are focused on PHP and static projects.",
        ),
        doctor_check(
            "Entrypoint",
            root.join("index.php").is_file()
                || root.join("public").join("index.php").is_file()
                || root.join("index.html").is_file(),
            "index.php or index.html found.",
            "No index.php or index.html entrypoint found.",
        ),
        doctor_check(
            "Scannable files",
            !files.is_empty(),
            "Text files were found for compatibility scan.",
            "No text files were found for compatibility scan.",
        ),
        hosting_scan_check(
            "Absolute Windows paths",
            !files
                .iter()
                .any(|(_, content)| contains_windows_path(content)),
            "No absolute Windows paths found.",
            "Absolute Windows paths found. Replace them with relative or server paths.",
        ),
        hosting_scan_check(
            "localhost URLs",
            !files.iter().any(|(_, content)| contains_localhost(content)),
            "No localhost URLs found.",
            "localhost URLs found. Replace them before upload.",
        ),
        hosting_scan_check(
            "Mixed content",
            !files.iter().any(|(_, content)| content.contains("http://")),
            "No http:// references found.",
            "http:// references found. Check mixed-content risk for HTTPS hosting.",
        ),
        doctor_check(
            ".htaccess",
            root.join(".htaccess").is_file() || project.project_type != "php",
            ".htaccess is present or not required for this project type.",
            ".htaccess is missing. Confirm rewrite rules if this PHP project needs routing.",
        ),
        doctor_check(
            "Writable folders",
            root.join("uploads").is_dir()
                || root.join("storage").is_dir()
                || project.project_type != "php",
            "Writable folders look present or not required.",
            "No common writable folder found. Check uploads/storage needs.",
        ),
    ];
    Ok(HostingCompatibilityReport {
        project_id: project.id,
        project_name: project.name,
        checks,
    })
}

fn hosting_scan_check(label: &str, ok: bool, success: &str, failure: &str) -> ProjectDoctorCheck {
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

fn contains_windows_path(content: &str) -> bool {
    content.as_bytes().windows(3).any(|window| {
        window[0].is_ascii_alphabetic()
            && window[1] == b':'
            && (window[2] == b'\\' || window[2] == b'/')
    })
}

fn contains_localhost(content: &str) -> bool {
    let lower = content.to_lowercase();
    lower.contains("localhost") || lower.contains("127.0.0.1")
}

#[tauri::command]
fn detect_project_type(path: String) -> Result<String, String> {
    let root = PathBuf::from(path);
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
    Ok("unknown".to_string())
}

#[tauri::command]
fn start_project(id: String, state: State<AppState>) -> Result<Project, String> {
    start_project_inner(&state, &id)
}

fn start_project_inner(state: &AppState, id: &str) -> Result<Project, String> {
    if state
        .processes
        .lock()
        .map_err(|_| "Process lock failed")?
        .children
        .contains_key(id)
    {
        return get_project(&state.db_path, id);
    }
    let mut project = get_project(&state.db_path, id)?;
    validate_project_path(Path::new(&project.path))?;
    validate_project_type(&project.project_type)?;
    let settings = read_settings_at(&state.db_path)?;
    let trust_runtime = trust_runtime_for_project(&project, &settings);
    if !project.trusted {
        return Err(format!(
            "Project is not trusted yet. Detected command runtime: {}. Use Trust Project before starting local scripts.",
            trust_runtime
        ));
    }
    let port = match project.port {
        Some(port) if is_port_free(port) => port,
        Some(port) => {
            insert_log(
                &state.db_path,
                Some(&project.id),
                "warning",
                &format!(
                    "Configured port {} is occupied. Selecting a free port.",
                    port
                ),
            )?;
            find_free_port(settings.port_start, settings.port_end)?
        }
        None => find_free_port(settings.port_start, settings.port_end)?,
    };
    if settings.clear_next_before_start && project.project_type == "next" {
        clear_cache_at(Path::new(&project.path))?;
    }
    project.status = "starting".to_string();
    project.port = Some(port);
    upsert_project(&state.db_path, &project)?;
    let command_spec = build_command(&project, &settings, port)?;
    insert_log(
        &state.db_path,
        Some(&project.id),
        "server",
        &format!("Starting: {}", command_spec.display),
    )?;
    let env_vars = parse_environment_variables(&settings.environment_variables)?;
    if !env_vars.is_empty() {
        let keys = env_vars
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        insert_log(
            &state.db_path,
            Some(&project.id),
            "server",
            &format!("Environment keys: {}", keys),
        )?;
    }
    let mut command = Command::new(&command_spec.program);
    command.args(&command_spec.args);
    command.current_dir(&project.path);
    command.env("PATH", runtime_path(&settings));
    for (key, value) in env_vars {
        command.env(key, value);
    }
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.stdin(Stdio::null());
    let mut child = command
        .spawn()
        .map_err(|error| friendly_spawn_error(&project, &command_spec.display, error))?;
    let pid = child.id();
    stream_child_logs(&state.db_path, &project, child.stdout.take(), "server");
    stream_child_logs(&state.db_path, &project, child.stderr.take(), "error");
    state
        .processes
        .lock()
        .map_err(|_| "Process lock failed")?
        .children
        .insert(project.id.clone(), child);
    let started_at = now();
    let conn = connect(&state.db_path)?;
    conn.execute(
        "INSERT OR REPLACE INTO processes (project_id, pid, command, cwd, port, started_at, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'starting')",
        params![
            project.id,
            pid,
            command_spec.display,
            project.path,
            port,
            started_at
        ],
    )
    .map_err(|error| error.to_string())?;
    project.command = Some(command_spec.display);
    project.updated_at = now();
    upsert_project(&state.db_path, &project)?;
    monitor_project_startup(
        state.db_path.clone(),
        project.id.clone(),
        pid,
        port,
        settings.process_timeout,
    );
    if settings.open_external_browser_on_start {
        let _ = open::that(format!("http://localhost:{}", port));
    }
    Ok(project)
}

#[tauri::command]
fn stop_project(id: String, state: State<AppState>) -> Result<Project, String> {
    stop_project_inner(&state, &id)
}

fn stop_project_inner(state: &AppState, id: &str) -> Result<Project, String> {
    let mut project = get_project(&state.db_path, id)?;
    project.status = "stopping".to_string();
    upsert_project(&state.db_path, &project)?;
    if let Some(mut child) = state
        .processes
        .lock()
        .map_err(|_| "Process lock failed")?
        .children
        .remove(id)
    {
        let _ = child.kill();
        let _ = child.wait();
    } else if let Some(pid) = stored_pid(&state.db_path, id)? {
        kill_process_tree(pid);
    }
    let conn = connect(&state.db_path)?;
    conn.execute("DELETE FROM processes WHERE project_id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    project.status = "stopped".to_string();
    project.updated_at = now();
    upsert_project(&state.db_path, &project)?;
    insert_log(
        &state.db_path,
        Some(&project.id),
        "server",
        "Project stopped",
    )?;
    Ok(project)
}

#[tauri::command]
fn start_all_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    start_all_projects_inner(&state)
}

fn start_all_projects_inner(state: &AppState) -> Result<Vec<Project>, String> {
    insert_log(&state.db_path, None, "server", "Start All requested")?;
    let projects = projects_from_db(&state.db_path)?;
    let mut started = Vec::new();
    for project in projects {
        if matches!(project.status.as_str(), "running" | "starting") {
            continue;
        }
        match start_project_inner(state, &project.id) {
            Ok(project) => started.push(project),
            Err(error) => {
                let _ = insert_log(
                    &state.db_path,
                    Some(&project.id),
                    "error",
                    &format!("Start All failed: {}", user_error(&error)),
                );
            }
        }
    }
    Ok(started)
}

#[tauri::command]
fn stop_all_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    stop_all_projects_inner(&state)
}

fn stop_all_projects_inner(state: &AppState) -> Result<Vec<Project>, String> {
    insert_log(&state.db_path, None, "server", "Stop All requested")?;
    let projects = projects_from_db(&state.db_path)?;
    let mut stopped = Vec::new();
    for project in projects {
        if matches!(project.status.as_str(), "stopped" | "idle") {
            continue;
        }
        match stop_project_inner(state, &project.id) {
            Ok(project) => stopped.push(project),
            Err(error) => {
                let _ = insert_log(
                    &state.db_path,
                    Some(&project.id),
                    "error",
                    &format!("Stop All failed: {}", user_error(&error)),
                );
            }
        }
    }
    Ok(stopped)
}

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    open::that(path).map_err(|error| error.to_string())
}

#[tauri::command]
fn open_in_code(path: String) -> Result<(), String> {
    Command::new("code")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|_| {
            "VS Code command 'code' was not found. Install VS Code shell command and try again."
                .to_string()
        })
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    open::that(url).map_err(|error| error.to_string())
}

#[tauri::command]
fn clear_project_cache(id: String, state: State<AppState>) -> Result<(), String> {
    let project = get_project(&state.db_path, &id)?;
    clear_cache_at(Path::new(&project.path))?;
    insert_log(
        &state.db_path,
        Some(&project.id),
        "build",
        "Next.js cache folders cleared",
    )?;
    Ok(())
}

fn clear_cache_at(root: &Path) -> Result<(), String> {
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

#[tauri::command]
fn list_servers(state: State<AppState>) -> Result<Vec<ServerProcess>, String> {
    let conn = connect(&state.db_path)?;
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut stmt = conn
        .prepare(
            "SELECT p.project_id, pr.name, pr.project_type, p.pid, p.port, p.command, p.cwd, p.started_at, p.status
             FROM processes p JOIN projects pr ON pr.id = p.project_id ORDER BY p.started_at DESC",
        )
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let pid_u32: u32 = row.get(3)?;
            let process = sys.process(Pid::from_u32(pid_u32));
            Ok((
                process.is_some(),
                ServerProcess {
                    project_id: row.get(0)?,
                    project_name: row.get(1)?,
                    project_type: row.get(2)?,
                    pid: pid_u32,
                    port: row.get::<_, u16>(4)?,
                    url: format!("http://localhost:{}", row.get::<_, u16>(4)?),
                    network_url: network_url(row.get::<_, u16>(4)?).unwrap_or_else(|_| {
                        format!(
                            "http://127.0.0.1:{}",
                            row.get::<_, u16>(4).unwrap_or_default()
                        )
                    }),
                    status: row.get(8)?,
                    command: row.get(5)?,
                    cwd: row.get(6)?,
                    started_at: row.get(7)?,
                    memory_usage_mb: process.map(|p| p.memory() as f32 / 1024.0 / 1024.0),
                    cpu_usage: process.map(|p| p.cpu_usage()),
                },
            ))
        })
        .map_err(|error| error.to_string())?;
    let mut servers = Vec::new();
    for row in rows {
        let (alive, server) = row.map_err(|error| error.to_string())?;
        if alive {
            servers.push(server);
        } else {
            let _ = mark_project_stopped(&state.db_path, &server.project_id);
            let _ = insert_log(
                &state.db_path,
                Some(&server.project_id),
                "warning",
                "Removed stale process record",
            );
        }
    }
    Ok(servers)
}

#[tauri::command]
fn network_url(port: u16) -> Result<String, String> {
    Ok(build_network_url(port))
}

#[tauri::command]
fn list_ports(state: State<AppState>) -> Result<Vec<PortInfo>, String> {
    let settings = get_settings(state.clone())?;
    let servers = list_servers(state)?;
    let mut result = Vec::new();
    for port in settings.port_start..=settings.port_end.min(settings.port_start + 120) {
        if let Some(server) = servers.iter().find(|server| server.port == port) {
            result.push(PortInfo {
                port,
                available: false,
                pid: Some(server.pid),
                project_id: Some(server.project_id.clone()),
                project_name: Some(server.project_name.clone()),
                external: false,
            });
        } else {
            result.push(PortInfo {
                port,
                available: is_port_free(port),
                pid: None,
                project_id: None,
                project_name: None,
                external: !is_port_free(port),
            });
        }
    }
    Ok(result)
}

#[tauri::command]
fn release_port(port: u16, state: State<AppState>) -> Result<(), String> {
    let servers = list_servers(state.clone())?;
    if let Some(server) = servers.into_iter().find(|server| server.port == port) {
        stop_project(server.project_id, state)?;
        Ok(())
    } else {
        Err("Port is used by an external process. Local Dev Studio will not kill external processes automatically.".to_string())
    }
}

#[tauri::command]
fn list_logs(
    project_id: Option<String>,
    level: Option<String>,
    search: Option<String>,
    state: State<AppState>,
) -> Result<Vec<LogEntry>, String> {
    let conn = connect(&state.db_path)?;
    let project_filter = project_id.filter(|value| !value.is_empty());
    let level_filter = level.filter(|value| !value.is_empty());
    let search_filter = search
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{}%", value));
    let mut stmt = conn
        .prepare(
            "SELECT l.id, l.project_id, p.name, l.level, l.message, l.created_at
             FROM logs l LEFT JOIN projects p ON p.id = l.project_id
             WHERE (?1 IS NULL OR l.project_id = ?1)
               AND (?2 IS NULL OR l.level = ?2)
               AND (?3 IS NULL OR l.message LIKE ?3)
             ORDER BY l.id DESC LIMIT 500",
        )
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(
            params![project_filter, level_filter, search_filter],
            |row| {
                Ok(LogEntry {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    project_name: row.get(2)?,
                    level: row.get(3)?,
                    message: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn clear_logs(state: State<AppState>) -> Result<(), String> {
    connect(&state.db_path)?
        .execute("DELETE FROM logs", [])
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn export_logs(state: State<AppState>) -> Result<String, String> {
    let logs = list_logs(None, None, None, state.clone())?;
    let path = default_data_dir().join(format!("logs-{}.txt", Utc::now().format("%Y%m%d-%H%M%S")));
    let body = logs
        .into_iter()
        .map(|log| {
            format!(
                "{} [{}] {} {}",
                log.created_at,
                log.level,
                log.project_name.unwrap_or_default(),
                log.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&path, body).map_err(|error| error.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Result<Settings, String> {
    read_settings_at(&state.db_path)
}

fn read_settings_at(db_path: &Path) -> Result<Settings, String> {
    let conn = connect(db_path)?;
    let value: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'settings'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    value
        .and_then(|value| serde_json::from_str(&value).ok())
        .ok_or_else(|| "Settings record is missing or invalid.".to_string())
}

#[tauri::command]
fn save_settings(settings: Settings, state: State<AppState>) -> Result<Settings, String> {
    if settings.port_start > settings.port_end {
        return Err("Default port range is invalid: start must be lower than end.".to_string());
    }
    validate_package_manager(&settings.package_manager)?;
    fs::create_dir_all(&settings.projects_folder)
        .map_err(|error| format!("Cannot create projects folder: {}", error))?;
    fs::create_dir_all(&settings.sandboxes_folder)
        .map_err(|error| format!("Cannot create sandboxes folder: {}", error))?;
    let value = serde_json::to_string(&settings).map_err(|error| error.to_string())?;
    connect(&state.db_path)?
        .execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('settings', ?1)",
            params![value],
        )
        .map_err(|error| error.to_string())?;
    apply_launch_on_startup(&settings)?;
    prune_logs(&state.db_path, settings.log_retention)?;
    Ok(settings)
}

#[tauri::command]
fn list_templates(state: State<AppState>) -> Result<Vec<TemplateInfo>, String> {
    let conn = connect(&state.db_path)?;
    let mut stmt = conn
        .prepare("SELECT id, name, project_type, built_in, path FROM templates ORDER BY built_in DESC, name")
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(TemplateInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                project_type: row.get(2)?,
                built_in: row.get::<_, i64>(3)? == 1,
                path: row.get(4)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn create_sandbox(template_id: String, state: State<AppState>) -> Result<Project, String> {
    let settings = get_settings(state.clone())?;
    fs::create_dir_all(&settings.sandboxes_folder).map_err(|error| error.to_string())?;
    let index = fs::read_dir(&settings.sandboxes_folder)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("sandbox-"))
        .count()
        + 1;
    let name = format!("sandbox-{:03}", index);
    let project = create_project_from_template(
        &template_id,
        Some(name),
        PathBuf::from(&settings.sandboxes_folder),
        state.clone(),
    )?;
    let started = start_project(project.id.clone(), state.clone())?;
    let conn = connect(&state.db_path)?;
    conn.execute(
        "INSERT OR REPLACE INTO sandboxes (id, project_id, template_id, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![format!("sandbox_{}", Uuid::new_v4().simple()), started.id, template_id, now()],
    )
    .map_err(|error| error.to_string())?;
    Ok(started)
}

#[tauri::command]
fn create_from_template(
    template_id: String,
    name: Option<String>,
    state: State<AppState>,
) -> Result<Project, String> {
    let settings = get_settings(state.clone())?;
    fs::create_dir_all(&settings.projects_folder).map_err(|error| error.to_string())?;
    create_project_from_template(
        &template_id,
        name,
        PathBuf::from(settings.projects_folder),
        state,
    )
}

fn create_project_from_template(
    template_id: &str,
    name: Option<String>,
    base: PathBuf,
    state: State<AppState>,
) -> Result<Project, String> {
    let template = get_template(&state.db_path, template_id)?;
    let project_name =
        name.unwrap_or_else(|| template.name.to_lowercase().replace([' ', '+', '.'], "-"));
    let target = unique_path(&base, &project_name);
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;
    if template.built_in {
        write_builtin_template(template_id, &target)?;
    } else if let Some(path) = template.path {
        let template_root = PathBuf::from(path);
        let files_root = template_root.join("files");
        copy_dir_all(
            if files_root.is_dir() {
                &files_root
            } else {
                &template_root
            },
            &target,
        )?;
    }
    add_project(target.to_string_lossy().to_string(), state)
}

#[tauri::command]
fn duplicate_template(template_id: String, state: State<AppState>) -> Result<TemplateInfo, String> {
    let template = get_template(&state.db_path, &template_id)?;
    let id = format!("template_{}", Uuid::new_v4().simple());
    let name = format!("{} Copy", template.name);
    connect(&state.db_path)?
        .execute(
            "INSERT INTO templates (id, name, project_type, built_in, path) VALUES (?1, ?2, ?3, 0, ?4)",
            params![id, name, template.project_type, template.path],
        )
        .map_err(|error| error.to_string())?;
    Ok(TemplateInfo {
        id,
        name,
        project_type: template.project_type,
        built_in: false,
        path: template.path,
    })
}

#[tauri::command]
fn delete_template(template_id: String, state: State<AppState>) -> Result<(), String> {
    let template = get_template(&state.db_path, &template_id)?;
    if template.built_in {
        return Err("Built-in templates cannot be deleted.".to_string());
    }
    connect(&state.db_path)?
        .execute("DELETE FROM templates WHERE id = ?1", params![template_id])
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn import_template_zip(zip_path: String, state: State<AppState>) -> Result<TemplateInfo, String> {
    let source = PathBuf::from(zip_path.trim());
    if source
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("zip"))
        != Some(true)
    {
        return Err("Selected file must be a .zip archive.".to_string());
    }
    if !source.exists() {
        return Err("ZIP file was not found.".to_string());
    }
    if !source.is_file() {
        return Err("Selected ZIP path is not a file.".to_string());
    }
    let metadata = fs::metadata(&source).map_err(|error| error.to_string())?;
    if metadata.len() > 100 * 1024 * 1024 {
        return Err("ZIP archive is too large. Maximum supported size is 100 MB.".to_string());
    }
    let templates_root = default_data_dir().join("templates");
    fs::create_dir_all(&templates_root).map_err(|error| error.to_string())?;
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("template");
    let target = unique_path(&templates_root, stem);
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;
    if let Err(error) = extract_zip(&source, &target) {
        let _ = fs::remove_dir_all(&target);
        return Err(error);
    }
    let manifest = read_template_manifest(&target)?;
    if !is_allowed_project_type(&manifest.project_type) {
        let _ = fs::remove_dir_all(&target);
        return Err("Template manifest has an unsupported project type.".to_string());
    }
    if let Some(package_manager) = &manifest.package_manager {
        validate_package_manager(package_manager)?;
    }
    let id = format!("template_{}", Uuid::new_v4().simple());
    let name = manifest.name;
    let project_type = manifest.project_type;
    connect(&state.db_path)?
        .execute(
            "INSERT INTO templates (id, name, project_type, built_in, path) VALUES (?1, ?2, ?3, 0, ?4)",
            params![id, name, project_type, target.to_string_lossy().to_string()],
        )
        .map_err(|error| error.to_string())?;
    Ok(TemplateInfo {
        id,
        name,
        project_type,
        built_in: false,
        path: Some(target.to_string_lossy().to_string()),
    })
}

#[tauri::command]
fn export_template_zip(template_id: String, state: State<AppState>) -> Result<String, String> {
    let template = get_template(&state.db_path, &template_id)?;
    if template.built_in {
        return Err(
            "Built-in templates are generated from code and are not exported as ZIP files."
                .to_string(),
        );
    }
    let source = template
        .path
        .ok_or_else(|| "Template has no source folder.".to_string())?;
    let target = default_data_dir().join(format!("{}.zip", template.name.replace(' ', "-")));
    zip_dir(Path::new(&source), &target)?;
    Ok(target.to_string_lossy().to_string())
}

fn get_template(db_path: &Path, id: &str) -> Result<TemplateInfo, String> {
    connect(db_path)?
        .query_row(
            "SELECT id, name, project_type, built_in, path FROM templates WHERE id = ?1",
            params![id],
            |row| {
                Ok(TemplateInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    project_type: row.get(2)?,
                    built_in: row.get::<_, i64>(3)? == 1,
                    path: row.get(4)?,
                })
            },
        )
        .map_err(|_| "Template not found.".to_string())
}

struct CommandSpec {
    program: String,
    args: Vec<String>,
    display: String,
}

fn shell_install_then_run(program: &str, run_args: &[String], display: &str) -> CommandSpec {
    let install = shell_quote(program).to_string() + " install";
    let run = std::iter::once(shell_quote(program))
        .chain(run_args.iter().map(|arg| shell_quote(arg)))
        .collect::<Vec<_>>()
        .join(" ");
    let command_line = format!("{} && {}", install, run);
    if cfg!(windows) {
        CommandSpec {
            program: "cmd".to_string(),
            args: vec!["/C".to_string(), command_line],
            display: display.to_string(),
        }
    } else {
        CommandSpec {
            program: "sh".to_string(),
            args: vec!["-c".to_string(), command_line],
            display: display.to_string(),
        }
    }
}

fn shell_quote(value: &str) -> String {
    if cfg!(windows) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn build_command(project: &Project, settings: &Settings, port: u16) -> Result<CommandSpec, String> {
    match project.project_type.as_str() {
        "next" => {
            ensure_file(Path::new(&project.path).join("package.json"), "package.json not found. Next.js projects must have package.json in the root.")?;
            let package_manager = package_manager(settings);
            let program = resolve_runtime(package_manager.as_str(), settings);
            let mut args = package_runner_args(package_manager.as_str(), "next", &["dev"]);
            if project.use_turbopack || settings.use_turbopack {
                args.push("--turbopack".to_string());
            }
            args.extend(["-H", "0.0.0.0", "-p", &port.to_string()].iter().map(|value| value.to_string()));
            if !Path::new(&project.path).join("node_modules").exists() {
                return Ok(shell_install_then_run(&program, &args, &format!("{} install && {} {}", program, program, args.join(" "))));
            }
            Ok(CommandSpec { display: format!("{} {}", program, args.join(" ")), program, args })
        }
        "vite" | "astro" => {
            ensure_file(Path::new(&project.path).join("package.json"), "package.json not found. Vite/Astro projects must have package.json in the root.")?;
            let package_manager = package_manager(settings);
            let program = resolve_runtime(package_manager.as_str(), settings);
            let mut args = package_dev_args(package_manager.as_str());
            args.extend(["--host", "0.0.0.0", "--port", &port.to_string()].iter().map(|value| value.to_string()));
            if !Path::new(&project.path).join("node_modules").exists() {
                return Ok(shell_install_then_run(&program, &args, &format!("{} install && {} {}", program, program, args.join(" "))));
            }
            Ok(CommandSpec { display: format!("{} {}", program, args.join(" ")), program, args })
        }
        "php" => {
            let php = resolve_runtime("php", settings);
            let args = vec!["-S".into(), format!("0.0.0.0:{}", port), "-t".into(), project.path.clone()];
            Ok(CommandSpec { display: format!("{} {}", php, args.join(" ")), program: php, args })
        }
        "static" => {
            let node = resolve_runtime("node", settings);
            let script = "const http=require('http'),fs=require('fs'),path=require('path');const root=path.resolve(process.cwd());const mime={'.html':'text/html','.css':'text/css','.js':'text/javascript','.json':'application/json','.png':'image/png','.jpg':'image/jpeg','.svg':'image/svg+xml'};http.createServer((req,res)=>{let p=decodeURIComponent(req.url.split('?')[0]);if(p==='/' )p='/index.html';let f=path.resolve(root,'.'+p);if(!f.startsWith(root+path.sep)&&f!==root){res.writeHead(403);return res.end('Forbidden')}fs.readFile(f,(e,d)=>{if(e){res.writeHead(404);res.end('Not found')}else{res.writeHead(200,{'Content-Type':mime[path.extname(f)]||'application/octet-stream'});res.end(d)}})}).listen(PORT,'0.0.0.0');";
            let args = vec!["-e".into(), script.replace("PORT", &port.to_string())];
            Ok(CommandSpec { display: format!("{} -e <static-server> port {}", node, port), program: node, args })
        }
        _ => Err("Unknown project type. Add package.json, index.html, index.php or a supported config file.".to_string()),
    }
}

fn resolve_runtime(name: &str, settings: &Settings) -> String {
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

fn runtime_source(name: &str, settings: &Settings, resolved: &str) -> String {
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

fn runtime_path(settings: &Settings) -> String {
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

fn version_for(program: String, args: &[&str]) -> String {
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

fn friendly_spawn_error(project: &Project, command: &str, error: std::io::Error) -> String {
    match project.project_type.as_str() {
        "next" => format!("Не удалось запустить Next.js проект. Проверь, установлен ли pnpm, существует ли package.json в корне проекта, и доступна ли команда: {}. Детали: {}", command, error),
        "vite" | "astro" => format!("Не удалось запустить dev server. Проверь pnpm install и script dev в package.json. Детали: {}", error),
        "php" => format!("PHP не найден или не запускается. Укажи PHP path в Settings > Runtime. Детали: {}", error),
        "static" => format!("Не удалось запустить static server через Node.js. Проверь Node path или bundled runtime. Детали: {}", error),
        _ => format!("spawn failed: {}", error),
    }
}

fn stream_child_logs(
    db_path: &Path,
    project: &Project,
    pipe: Option<impl std::io::Read + Send + 'static>,
    level: &str,
) {
    let Some(pipe) = pipe else {
        return;
    };
    let db_path = db_path.to_path_buf();
    let project_id = project.id.clone();
    let level = level.to_string();
    thread::spawn(move || {
        let reader = BufReader::new(pipe);
        for line in reader.lines().map_while(Result::ok) {
            let _ = insert_log(&db_path, Some(&project_id), &level, &line);
        }
    });
}

fn get_project(db_path: &Path, id: &str) -> Result<Project, String> {
    connect(db_path)?
        .query_row(
            "SELECT id, name, path, project_type, port, command, status, use_turbopack, trusted, trusted_at, trusted_runtime, created_at, updated_at FROM projects WHERE id = ?1",
            params![id],
            project_from_row,
        )
        .map_err(|_| "Project not found.".to_string())
}

fn project_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        project_type: row.get(3)?,
        port: row.get(4)?,
        command: row.get(5)?,
        status: row.get(6)?,
        use_turbopack: row.get::<_, i64>(7)? == 1,
        trusted: row.get::<_, i64>(8)? == 1,
        trusted_at: row.get(9)?,
        trusted_runtime: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn upsert_project(db_path: &Path, project: &Project) -> Result<(), String> {
    connect(db_path)?
        .execute(
            "INSERT OR REPLACE INTO projects (id, name, path, project_type, port, command, status, use_turbopack, trusted, trusted_at, trusted_runtime, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                project.id,
                project.name,
                project.path,
                project.project_type,
                project.port,
                project.command,
                project.status,
                if project.use_turbopack { 1 } else { 0 },
                if project.trusted { 1 } else { 0 },
                project.trusted_at,
                project.trusted_runtime,
                project.created_at,
                project.updated_at
            ],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn trust_runtime_for_project(project: &Project, settings: &Settings) -> String {
    match project.project_type.as_str() {
        "next" | "vite" | "astro" => package_manager(settings),
        "php" => "php".to_string(),
        "static" => "node".to_string(),
        _ => "unknown".to_string(),
    }
}

fn insert_log(
    db_path: &Path,
    project_id: Option<&str>,
    level: &str,
    message: &str,
) -> Result<(), String> {
    connect(db_path)?
        .execute(
            "INSERT INTO logs (project_id, level, message, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![project_id, level, message, now()],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn ensure_file(path: PathBuf, message: &str) -> Result<(), String> {
    if path.exists() {
        Ok(())
    } else {
        Err(message.to_string())
    }
}

fn user_error(error: &str) -> String {
    error.lines().next().unwrap_or(error).to_string()
}

fn package_manager(settings: &Settings) -> String {
    match settings.package_manager.trim().to_lowercase().as_str() {
        "npm" => "npm".to_string(),
        "yarn" => "yarn".to_string(),
        "bun" => "bun".to_string(),
        _ => "pnpm".to_string(),
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

fn parse_environment_variables(raw: &str) -> Result<Vec<(String, String)>, String> {
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

fn monitor_project_startup(
    db_path: PathBuf,
    project_id: String,
    pid: u32,
    port: u16,
    timeout_seconds: u32,
) {
    thread::spawn(move || {
        let timeout = Duration::from_secs(timeout_seconds.max(1) as u64);
        let started = Instant::now();
        while started.elapsed() < timeout {
            if !process_exists(pid) {
                let _ = update_process_status(&db_path, &project_id, "error");
                let _ = update_project_status(&db_path, &project_id, "error");
                let _ = insert_log(
                    &db_path,
                    Some(&project_id),
                    "error",
                    "Process exited before the server became ready",
                );
                return;
            }
            if !is_port_free(port) {
                let _ = update_process_status(&db_path, &project_id, "running");
                let _ = update_project_status(&db_path, &project_id, "running");
                let _ = insert_log(
                    &db_path,
                    Some(&project_id),
                    "server",
                    &format!("Server ready on port {}", port),
                );
                return;
            }
            thread::sleep(Duration::from_millis(350));
        }
        let _ = update_process_status(&db_path, &project_id, "error");
        let _ = update_project_status(&db_path, &project_id, "error");
        let _ = insert_log(
            &db_path,
            Some(&project_id),
            "error",
            &format!(
                "Server did not open port {} within {} seconds.",
                port,
                timeout_seconds.max(1)
            ),
        );
    });
}

fn process_exists(pid: u32) -> bool {
    let mut sys = System::new_all();
    sys.refresh_all();
    sys.process(Pid::from_u32(pid)).is_some()
}

fn update_process_status(db_path: &Path, project_id: &str, status: &str) -> Result<(), String> {
    connect(db_path)?
        .execute(
            "UPDATE processes SET status = ?2 WHERE project_id = ?1",
            params![project_id, status],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn update_project_status(db_path: &Path, project_id: &str, status: &str) -> Result<(), String> {
    connect(db_path)?
        .execute(
            "UPDATE projects SET status = ?2, updated_at = ?3 WHERE id = ?1",
            params![project_id, status, now()],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn stored_pid(db_path: &Path, project_id: &str) -> Result<Option<u32>, String> {
    connect(db_path)?
        .query_row(
            "SELECT pid FROM processes WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())
}

fn kill_process_tree(pid: u32) {
    if cfg!(windows) {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    } else {
        let _ = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .status();
    }
}

fn mark_project_stopped(db_path: &Path, project_id: &str) -> Result<(), String> {
    connect(db_path)?
        .execute(
            "DELETE FROM processes WHERE project_id = ?1",
            params![project_id],
        )
        .map_err(|error| error.to_string())?;
    connect(db_path)?
        .execute(
            "UPDATE projects SET status = 'stopped', updated_at = ?2 WHERE id = ?1",
            params![project_id, now()],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn apply_launch_on_startup(settings: &Settings) -> Result<(), String> {
    if !cfg!(windows) {
        return Ok(());
    }
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let status = if settings.launch_on_startup {
        Command::new("reg")
            .args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "LocalDevStudio",
                "/t",
                "REG_SZ",
                "/d",
            ])
            .arg(exe.to_string_lossy().to_string())
            .arg("/f")
            .status()
    } else {
        Command::new("reg")
            .args([
                "delete",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "LocalDevStudio",
                "/f",
            ])
            .status()
    };
    status.map(|_| ()).map_err(|error| error.to_string())
}

fn prune_logs(db_path: &Path, retention_days: u32) -> Result<(), String> {
    if retention_days == 0 {
        return Ok(());
    }
    let cutoff = (Utc::now() - chrono::Duration::days(retention_days as i64)).to_rfc3339();
    connect(db_path)?
        .execute("DELETE FROM logs WHERE created_at < ?1", params![cutoff])
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn unique_path(base: &Path, name: &str) -> PathBuf {
    let mut candidate = base.join(name);
    let mut index = 2;
    while candidate.exists() {
        candidate = base.join(format!("{}-{}", name, index));
        index += 1;
    }
    candidate
}

fn write_builtin_template(template_id: &str, target: &Path) -> Result<(), String> {
    match template_id {
        "next-app-router" | "next-tailwind" => {
            fs::create_dir_all(target.join("app")).map_err(|error| error.to_string())?;
            fs::write(
                target.join("package.json"),
                next_package(template_id == "next-tailwind"),
            )
            .map_err(|error| error.to_string())?;
            fs::write(target.join("app").join("page.tsx"), "export default function Page() {\n  return <main><h1>Local Dev Studio Next.js Sandbox</h1></main>;\n}\n").map_err(|error| error.to_string())?;
            fs::write(target.join("app").join("layout.tsx"), "import type { ReactNode } from 'react';\n\nexport default function RootLayout({ children }: { children: ReactNode }) {\n  return <html lang=\"en\"><body>{children}</body></html>;\n}\n").map_err(|error| error.to_string())?;
            fs::write(
                target.join("next.config.mjs"),
                "const nextConfig = {};\nexport default nextConfig;\n",
            )
            .map_err(|error| error.to_string())?;
        }
        "vite-react" => {
            fs::create_dir_all(target.join("src")).map_err(|error| error.to_string())?;
            fs::write(target.join("package.json"), vite_package())
                .map_err(|error| error.to_string())?;
            fs::write(
                target.join("index.html"),
                "<div id=\"root\"></div><script type=\"module\" src=\"/src/main.tsx\"></script>\n",
            )
            .map_err(|error| error.to_string())?;
            fs::write(target.join("src").join("main.tsx"), "import React from 'react';\nimport ReactDOM from 'react-dom/client';\nReactDOM.createRoot(document.getElementById('root')!).render(<h1>Vite React Sandbox</h1>);\n").map_err(|error| error.to_string())?;
            fs::write(target.join("vite.config.ts"), "import { defineConfig } from 'vite';\nimport react from '@vitejs/plugin-react';\nexport default defineConfig({ plugins: [react()] });\n").map_err(|error| error.to_string())?;
        }
        "static-html" => {
            fs::create_dir_all(target.join("css")).map_err(|error| error.to_string())?;
            fs::write(target.join("index.html"), "<!doctype html><html><head><link rel=\"stylesheet\" href=\"css/style.css\"></head><body><h1>Static HTML Sandbox</h1><script src=\"js/app.js\"></script></body></html>\n").map_err(|error| error.to_string())?;
            fs::create_dir_all(target.join("js")).map_err(|error| error.to_string())?;
            fs::write(
                target.join("css").join("style.css"),
                "body{font-family:Segoe UI,Arial,sans-serif;margin:40px;}\n",
            )
            .map_err(|error| error.to_string())?;
            fs::write(
                target.join("js").join("app.js"),
                "console.log('Static sandbox ready');\n",
            )
            .map_err(|error| error.to_string())?;
        }
        "php-template" => {
            fs::write(
                target.join("index.php"),
                "<?php echo '<h1>PHP Sandbox</h1>'; ?>\n",
            )
            .map_err(|error| error.to_string())?;
        }
        _ => return Err("Unknown built-in template.".to_string()),
    }
    Ok(())
}

fn next_package(tailwind: bool) -> String {
    let extra = if tailwind {
        ",\n    \"tailwindcss\": \"latest\",\n    \"postcss\": \"latest\",\n    \"autoprefixer\": \"latest\""
    } else {
        ""
    };
    format!("{{\n  \"scripts\": {{ \"dev\": \"next dev\" }},\n  \"dependencies\": {{\n    \"next\": \"latest\",\n    \"react\": \"latest\",\n    \"react-dom\": \"latest\"{}\n  }},\n  \"devDependencies\": {{ \"typescript\": \"latest\", \"@types/react\": \"latest\", \"@types/node\": \"latest\" }}\n}}\n", extra)
}

fn vite_package() -> &'static str {
    "{\n  \"scripts\": { \"dev\": \"vite\" },\n  \"dependencies\": { \"@vitejs/plugin-react\": \"latest\", \"vite\": \"latest\", \"typescript\": \"latest\", \"react\": \"latest\", \"react-dom\": \"latest\", \"@types/react\": \"latest\", \"@types/react-dom\": \"latest\" },\n  \"devDependencies\": {}\n}\n"
}

fn copy_dir_all(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        let next_target = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &next_target)?;
        } else {
            fs::copy(entry.path(), next_target).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn zip_dir(source: &Path, target: &Path) -> Result<(), String> {
    let file = fs::File::create(target).map_err(|error| error.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    fn walk(
        zip: &mut zip::ZipWriter<fs::File>,
        source: &Path,
        base: &Path,
        options: zip::write::SimpleFileOptions,
    ) -> Result<(), String> {
        for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            let name = path
                .strip_prefix(base)
                .map_err(|error| error.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            if path.is_dir() {
                zip.add_directory(format!("{}/", name), options)
                    .map_err(|error| error.to_string())?;
                walk(zip, &path, base, options)?;
            } else {
                zip.start_file(name, options)
                    .map_err(|error| error.to_string())?;
                let mut file = fs::File::open(&path).map_err(|error| error.to_string())?;
                std::io::copy(&mut file, &mut *zip).map_err(|error| error.to_string())?;
            }
        }
        Ok(())
    }
    walk(&mut zip, source, source, options)?;
    zip.finish().map_err(|error| error.to_string())?;
    Ok(())
}

fn extract_zip(source: &Path, target: &Path) -> Result<(), String> {
    let file = fs::File::open(source).map_err(|error| error.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| error.to_string())?;
    validate_template_zip_archive(&mut archive)?;
    let root = target.canonicalize().map_err(|error| error.to_string())?;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|error| error.to_string())?;
        let Some(enclosed) = file.enclosed_name().map(|path| path.to_path_buf()) else {
            return Err("ZIP contains an unsafe path.".to_string());
        };
        let out_path = root.join(enclosed);
        if !out_path.starts_with(&root) {
            return Err("ZIP contains an unsafe path.".to_string());
        }
        if file.is_dir() {
            fs::create_dir_all(&out_path).map_err(|error| error.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            let mut output = fs::File::create(&out_path).map_err(|error| error.to_string())?;
            std::io::copy(&mut file, &mut output).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn validate_template_zip_archive(archive: &mut zip::ZipArchive<fs::File>) -> Result<(), String> {
    if archive.is_empty() {
        return Err("ZIP archive is empty.".to_string());
    }
    if archive.len() > 2_000 {
        return Err(
            "ZIP archive contains too many files. Maximum supported file count is 2000."
                .to_string(),
        );
    }
    let mut total_uncompressed = 0_u64;
    let mut has_manifest = false;
    let mut has_files_dir = false;
    let mut has_useful_file = false;
    for index in 0..archive.len() {
        let file = archive.by_index(index).map_err(|error| error.to_string())?;
        let Some(enclosed) = file.enclosed_name().map(|path| path.to_path_buf()) else {
            return Err("ZIP contains an unsafe path.".to_string());
        };
        if enclosed.is_absolute()
            || enclosed
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err("ZIP contains an unsafe path.".to_string());
        }
        total_uncompressed = total_uncompressed.saturating_add(file.size());
        if total_uncompressed > 250 * 1024 * 1024 {
            return Err("ZIP archive expands to more than 250 MB.".to_string());
        }
        let normalized = enclosed.to_string_lossy().replace('\\', "/");
        if normalized == "template.json" && file.is_file() {
            has_manifest = true;
        }
        if normalized == "files/" || normalized.starts_with("files/") {
            has_files_dir = true;
        }
        if normalized.starts_with("files/") && file.is_file() {
            has_useful_file = true;
        }
    }
    if !has_manifest {
        return Err("Template manifest is missing. Add template.json at the ZIP root.".to_string());
    }
    if !has_files_dir {
        return Err("Template ZIP must contain a files/ directory.".to_string());
    }
    if !has_useful_file {
        return Err("Template ZIP does not contain useful files inside files/.".to_string());
    }
    Ok(())
}

fn read_template_manifest(target: &Path) -> Result<TemplateManifest, String> {
    let manifest_path = target.join("template.json");
    if !manifest_path.is_file() {
        return Err("Template manifest is missing. Add template.json at the ZIP root.".to_string());
    }
    let manifest = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("Cannot read template.json: {}", error))?;
    let manifest: TemplateManifest = serde_json::from_str(&manifest)
        .map_err(|error| format!("Invalid template.json: {}", error))?;
    if manifest.name.trim().is_empty() {
        return Err("Template manifest name is required.".to_string());
    }
    if !target.join("files").is_dir() {
        return Err("Template ZIP must contain a files/ directory.".to_string());
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    fn temp_project(name: &str) -> PathBuf {
        let root = env::temp_dir().join(format!(
            "local-dev-studio-test-{}-{}",
            name,
            Uuid::new_v4().simple()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn parse_environment_variables_accepts_key_value_lines() {
        let vars = parse_environment_variables(
            "NEXT_PUBLIC_API=http://localhost:3000\n# comment\nPORT=3000",
        )
        .unwrap();
        assert_eq!(
            vars[0],
            (
                "NEXT_PUBLIC_API".to_string(),
                "http://localhost:3000".to_string()
            )
        );
        assert_eq!(vars[1], ("PORT".to_string(), "3000".to_string()));
    }

    #[test]
    fn parse_environment_variables_rejects_invalid_lines() {
        assert!(parse_environment_variables("NO_VALUE").is_err());
        assert!(parse_environment_variables("BAD KEY=value").is_err());
        assert!(parse_environment_variables("1BAD=value").is_err());
        assert!(parse_environment_variables("lower=value").is_err());
    }

    #[test]
    fn validate_project_path_requires_existing_directory() {
        let root = temp_project("path");
        validate_project_path(&root).unwrap();
        fs::remove_dir_all(&root).unwrap();
        assert!(validate_project_path(&root).is_err());
    }

    #[test]
    fn detect_project_type_detects_next_and_static() {
        let next = temp_project("next");
        fs::write(
            next.join("package.json"),
            r#"{"dependencies":{"next":"latest"}}"#,
        )
        .unwrap();
        assert_eq!(
            detect_project_type(next.to_string_lossy().to_string()).unwrap(),
            "next"
        );
        fs::remove_dir_all(next).unwrap();

        let static_site = temp_project("static");
        fs::write(static_site.join("index.html"), "<h1>Static</h1>").unwrap();
        assert_eq!(
            detect_project_type(static_site.to_string_lossy().to_string()).unwrap(),
            "static"
        );
        fs::remove_dir_all(static_site).unwrap();
    }

    #[test]
    fn find_free_port_returns_available_port() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let occupied = listener.local_addr().unwrap().port();
        let port = find_free_port(occupied.saturating_add(1), occupied.saturating_add(20)).unwrap();
        assert!(port > occupied);
    }

    #[test]
    fn validate_project_type_rejects_unknown() {
        assert!(validate_project_type("next").is_ok());
        assert!(validate_project_type("unknown").is_err());
    }

    #[test]
    fn runtime_version_args_accept_supported_runtimes() {
        assert_eq!(runtime_version_args("node").unwrap(), ["-v"]);
        assert_eq!(runtime_version_args("bun").unwrap(), ["--version"]);
        assert!(runtime_version_args("powershell").is_err());
    }

    #[test]
    fn package_json_has_script_detects_dev_script() {
        let root = temp_project("scripts");
        let package_json = root.join("package.json");
        fs::write(
            &package_json,
            r#"{"scripts":{"dev":"vite","build":"vite build"}}"#,
        )
        .unwrap();
        assert!(package_json_has_script(&package_json, "dev"));
        assert!(!package_json_has_script(&package_json, "preview"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn trust_runtime_for_project_uses_project_type() {
        let mut project = Project {
            id: "p1".to_string(),
            name: "Project".to_string(),
            path: "C:\\Projects\\demo".to_string(),
            project_type: "php".to_string(),
            port: None,
            command: None,
            status: "stopped".to_string(),
            use_turbopack: false,
            trusted: false,
            trusted_at: None,
            trusted_runtime: None,
            created_at: now(),
            updated_at: now(),
        };
        let settings = default_settings();
        assert_eq!(trust_runtime_for_project(&project, &settings), "php");
        project.project_type = "static".to_string();
        assert_eq!(trust_runtime_for_project(&project, &settings), "node");
    }

    #[test]
    fn hosting_scanner_detects_localhost_and_windows_paths() {
        assert!(contains_localhost("const api = 'http://localhost:3000';"));
        assert!(contains_windows_path(
            r#"include "C:\Users\demo\file.php";"#
        ));
        assert!(!contains_windows_path("/var/www/html/index.php"));
    }

    fn temp_zip(name: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "local-dev-studio-test-{}-{}.zip",
            name,
            Uuid::new_v4().simple()
        ))
    }

    fn write_zip(path: &Path, entries: &[(&str, &str)]) {
        use std::io::Write;
        let file = fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        for (name, content) in entries {
            zip.start_file(*name, options).unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }

    #[test]
    fn template_zip_validation_rejects_empty_archive() {
        let zip_path = temp_zip("empty");
        write_zip(&zip_path, &[]);
        let file = fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        assert!(validate_template_zip_archive(&mut archive).is_err());
        fs::remove_file(zip_path).unwrap();
    }

    #[test]
    fn template_zip_validation_rejects_zip_slip() {
        let zip_path = temp_zip("zipslip");
        write_zip(
            &zip_path,
            &[
                ("template.json", r#"{"name":"Bad","type":"static"}"#),
                ("files/index.html", "<h1>OK</h1>"),
                ("../evil.txt", "evil"),
            ],
        );
        let file = fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        assert!(validate_template_zip_archive(&mut archive).is_err());
        fs::remove_file(zip_path).unwrap();
    }

    #[test]
    fn template_zip_validation_accepts_manifest_and_files() {
        let zip_path = temp_zip("valid");
        write_zip(
            &zip_path,
            &[
                ("template.json", r#"{"name":"Static","type":"static"}"#),
                ("files/index.html", "<h1>OK</h1>"),
            ],
        );
        let file = fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        validate_template_zip_archive(&mut archive).unwrap();
        fs::remove_file(zip_path).unwrap();
    }
}
