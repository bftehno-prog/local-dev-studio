use rusqlite::{params, Connection, OptionalExtension};
use std::{
    env, fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};
use sysinfo::{Pid, System};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State,
};
use uuid::Uuid;

mod db;
mod models;
mod security;
mod services;
mod state;
mod utils;

use models::{
    default_language, CreateProjectRequest, DashboardData, DiagnosticItem,
    HostingCompatibilityReport, LogEntry, PortInfo, Project, ProjectDoctorReport, RuntimeInfo,
    ServerProcess, Settings, TemplateInfo, TemplateManifest, UpdateProjectRequest,
};
use security::validation::{
    is_allowed_project_type, validate_package_manager, validate_project_path, validate_project_type,
};
use services::command_builder::{build_command, package_manager, parse_environment_variables};
use services::hosting_compatibility::build_hosting_compatibility_report;
use services::log_service::{
    append_log as insert_log, clear_logs as clear_logs_at, export_logs as export_logs_at,
    list_logs as list_logs_at, prune_logs,
};
use services::process_manager::{
    kill_process_tree, mark_project_stopped, monitor_project_startup, stored_pid,
    update_project_status,
};
use services::project_cache::clear_cache_at;
use services::project_detector::detect_project_type_at;
use services::project_doctor::build_project_doctor_report;
use services::runtime_resolver::{
    resolve_runtime, runtime_path, runtime_source, runtime_version_args, version_for,
};
use state::{AppState, ManagedProcesses};
use utils::{
    network::{find_free_port, is_port_free, network_url as build_network_url},
    paths::default_data_dir,
    time::now,
};

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
            get_project,
            create_project,
            import_project,
            update_project,
            delete_project,
            add_project,
            remove_project,
            start_project,
            stop_project,
            trust_project,
            reset_project_trust,
            install_project_dependencies,
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

#[tauri::command]
fn get_project(id: String, state: State<AppState>) -> Result<Project, String> {
    project_by_id(&state.db_path, &id)
}

fn projects_from_db(db_path: &Path) -> Result<Vec<Project>, String> {
    let conn = connect(db_path)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, path, project_type, port, command, status, package_manager, use_docker, dev_port, proxy_port, last_started_at, last_error, use_turbopack, trusted, trusted_at, trusted_runtime, created_at, updated_at
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
        package_manager: None,
        use_docker: false,
        dev_port: None,
        proxy_port: None,
        last_started_at: None,
        last_error: None,
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
fn import_project(path: String, state: State<AppState>) -> Result<Project, String> {
    add_project(path, state)
}

#[tauri::command]
fn create_project(
    request: CreateProjectRequest,
    state: State<AppState>,
) -> Result<Project, String> {
    let name = request.name.trim();
    if name.is_empty() {
        return Err("Project name is required.".to_string());
    }
    let project_type = normalize_created_project_type(&request.project_type)?;
    let package_manager = request
        .package_manager
        .as_deref()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty());
    if let Some(manager) = &package_manager {
        validate_package_manager(manager)?;
    }
    let base = PathBuf::from(request.path.trim());
    if base.as_os_str().is_empty() {
        return Err("Project path is required.".to_string());
    }
    let target = if base.exists() && base.is_dir() {
        unique_path(&base, &safe_project_folder_name(name))
    } else {
        base
    };
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;
    if target
        .read_dir()
        .map_err(|error| error.to_string())?
        .next()
        .is_some()
    {
        return Err("Target project folder is not empty.".to_string());
    }
    write_project_wizard_template(&request.project_type, &project_type, name, &target)?;
    validate_project_path(&target)?;
    let timestamp = now();
    let project = Project {
        id: format!("project_{}", Uuid::new_v4().simple()),
        name: name.to_string(),
        path: target.to_string_lossy().to_string(),
        project_type,
        port: None,
        command: None,
        status: "stopped".to_string(),
        package_manager,
        use_docker: request.use_docker,
        dev_port: None,
        proxy_port: None,
        last_started_at: None,
        last_error: None,
        use_turbopack: false,
        trusted: true,
        trusted_at: Some(timestamp.clone()),
        trusted_runtime: None,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    };
    upsert_project(&state.db_path, &project)?;
    insert_log(
        &state.db_path,
        Some(&project.id),
        "info",
        "Project created from wizard",
    )?;
    if request.auto_install
        && matches!(
            project.project_type.as_str(),
            "next" | "vite" | "astro" | "node"
        )
    {
        return install_project_dependencies(project.id.clone(), state);
    }
    if request.auto_start {
        return start_project(project.id.clone(), state);
    }
    Ok(project)
}

#[tauri::command]
fn update_project(
    request: UpdateProjectRequest,
    state: State<AppState>,
) -> Result<Project, String> {
    let mut project = project_by_id(&state.db_path, &request.id)?;
    if let Some(name) = request.name {
        let name = name.trim();
        if name.is_empty() {
            return Err("Project name is required.".to_string());
        }
        project.name = name.to_string();
    }
    if let Some(package_manager) = request.package_manager {
        let package_manager = package_manager.trim().to_lowercase();
        if package_manager.is_empty() {
            project.package_manager = None;
        } else {
            validate_package_manager(&package_manager)?;
            project.package_manager = Some(package_manager);
        }
    }
    if let Some(use_docker) = request.use_docker {
        project.use_docker = use_docker;
    }
    if request.dev_port.is_some() {
        project.dev_port = request.dev_port;
        project.port = request.dev_port;
    }
    if request.proxy_port.is_some() {
        project.proxy_port = request.proxy_port;
    }
    project.updated_at = now();
    upsert_project(&state.db_path, &project)?;
    insert_log(
        &state.db_path,
        Some(&project.id),
        "info",
        "Project settings updated",
    )?;
    Ok(project)
}

#[tauri::command]
fn delete_project(id: String, state: State<AppState>) -> Result<(), String> {
    remove_project(id, state)
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
    let mut project = project_by_id(&state.db_path, &id)?;
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
    let mut project = project_by_id(&state.db_path, &id)?;
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
    let project = project_by_id(&state.db_path, &id)?;
    let settings = read_settings_at(&state.db_path)?;
    Ok(build_project_doctor_report(
        &project,
        &settings,
        &package_manager(&settings),
    ))
}

#[tauri::command]
fn hosting_compatibility_check(
    id: String,
    state: State<AppState>,
) -> Result<HostingCompatibilityReport, String> {
    let project = project_by_id(&state.db_path, &id)?;
    validate_project_path(Path::new(&project.path))?;
    build_hosting_compatibility_report(&project)
}

#[tauri::command]
fn detect_project_type(path: String) -> Result<String, String> {
    let root = PathBuf::from(path);
    detect_project_type_at(&root)
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
        return project_by_id(&state.db_path, id);
    }
    let mut project = project_by_id(&state.db_path, id)?;
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
    project.dev_port = Some(port);
    project.last_error = None;
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
    project.last_started_at = Some(started_at);
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
fn install_project_dependencies(id: String, state: State<AppState>) -> Result<Project, String> {
    let mut project = project_by_id(&state.db_path, &id)?;
    validate_project_path(Path::new(&project.path))?;
    if !matches!(
        project.project_type.as_str(),
        "next" | "vite" | "astro" | "node"
    ) {
        return Err("Dependency installation is only available for Node.js projects.".to_string());
    }
    if !project.trusted {
        return Err(
            "Project is not trusted yet. Trust the project before installing dependencies."
                .to_string(),
        );
    }
    ensure_file(
        Path::new(&project.path).join("package.json"),
        "package.json not found. Cannot install dependencies.",
    )?;
    let settings = read_settings_at(&state.db_path)?;
    let manager = project
        .package_manager
        .clone()
        .unwrap_or_else(|| package_manager(&settings));
    let program = resolve_runtime(&manager, &settings);
    let args = vec!["install".to_string()];
    let display = format!("{} install", program);
    insert_log(
        &state.db_path,
        Some(&project.id),
        "build",
        &format!("Installing dependencies: {}", display),
    )?;
    let mut command = Command::new(&program);
    command.args(&args);
    command.current_dir(&project.path);
    command.env("PATH", runtime_path(&settings));
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.stdin(Stdio::null());
    let mut child = command.spawn().map_err(|error| {
        format!(
            "Could not start dependency installation with {}. Check Settings > Runtime and make sure the package manager is installed. Details: {}",
            manager, error
        )
    })?;
    stream_child_logs(&state.db_path, &project, child.stdout.take(), "build");
    stream_child_logs(&state.db_path, &project, child.stderr.take(), "error");
    project.status = "installing".to_string();
    project.updated_at = now();
    upsert_project(&state.db_path, &project)?;
    let db_path = state.db_path.clone();
    let project_id = project.id.clone();
    thread::spawn(move || match child.wait() {
        Ok(status) if status.success() => {
            let _ = update_project_status(&db_path, &project_id, "stopped");
            let _ = insert_log(
                &db_path,
                Some(&project_id),
                "build",
                "Dependency installation completed.",
            );
        }
        Ok(status) => {
            let _ = update_project_status(&db_path, &project_id, "error");
            let _ = insert_log(
                &db_path,
                Some(&project_id),
                "error",
                &format!("Dependency installation failed with status {}.", status),
            );
        }
        Err(error) => {
            let _ = update_project_status(&db_path, &project_id, "error");
            let _ = insert_log(
                &db_path,
                Some(&project_id),
                "error",
                &format!("Dependency installation failed: {}", error),
            );
        }
    });
    Ok(project)
}

#[tauri::command]
fn stop_project(id: String, state: State<AppState>) -> Result<Project, String> {
    stop_project_inner(&state, &id)
}

fn stop_project_inner(state: &AppState, id: &str) -> Result<Project, String> {
    let mut project = project_by_id(&state.db_path, id)?;
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
    let project = project_by_id(&state.db_path, &id)?;
    clear_cache_at(Path::new(&project.path))?;
    insert_log(
        &state.db_path,
        Some(&project.id),
        "build",
        "Next.js cache folders cleared",
    )?;
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
    list_logs_at(&state.db_path, project_id, level, search)
}

#[tauri::command]
fn clear_logs(state: State<AppState>) -> Result<(), String> {
    clear_logs_at(&state.db_path)
}

#[tauri::command]
fn export_logs(state: State<AppState>) -> Result<String, String> {
    export_logs_at(&state.db_path, &default_data_dir())
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

fn friendly_spawn_error(project: &Project, command: &str, error: std::io::Error) -> String {
    match project.project_type.as_str() {
        "next" => format!("Не удалось запустить Next.js проект. Проверь, установлен ли pnpm, существует ли package.json в корне проекта, и доступна ли команда: {}. Детали: {}", command, error),
        "vite" | "astro" => format!("Не удалось запустить dev server. Проверь pnpm install и script dev в package.json. Детали: {}", error),
        "node" => format!("Не удалось запустить Node.js проект. Проверь package.json, script dev и установленный package manager. Детали: {}", error),
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

fn project_by_id(db_path: &Path, id: &str) -> Result<Project, String> {
    connect(db_path)?
        .query_row(
            "SELECT id, name, path, project_type, port, command, status, package_manager, use_docker, dev_port, proxy_port, last_started_at, last_error, use_turbopack, trusted, trusted_at, trusted_runtime, created_at, updated_at FROM projects WHERE id = ?1",
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
        package_manager: row.get(7)?,
        use_docker: row.get::<_, i64>(8)? == 1,
        dev_port: row.get(9)?,
        proxy_port: row.get(10)?,
        last_started_at: row.get(11)?,
        last_error: row.get(12)?,
        use_turbopack: row.get::<_, i64>(13)? == 1,
        trusted: row.get::<_, i64>(14)? == 1,
        trusted_at: row.get(15)?,
        trusted_runtime: row.get(16)?,
        created_at: row.get(17)?,
        updated_at: row.get(18)?,
    })
}

fn upsert_project(db_path: &Path, project: &Project) -> Result<(), String> {
    connect(db_path)?
        .execute(
            "INSERT OR REPLACE INTO projects (id, name, path, project_type, port, command, status, package_manager, use_docker, dev_port, proxy_port, last_started_at, last_error, use_turbopack, trusted, trusted_at, trusted_runtime, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            params![
                project.id,
                project.name,
                project.path,
                project.project_type,
                project.port,
                project.command,
                project.status,
                project.package_manager,
                if project.use_docker { 1 } else { 0 },
                project.dev_port,
                project.proxy_port,
                project.last_started_at,
                project.last_error,
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
        "next" | "vite" | "astro" | "node" => package_manager(settings),
        "php" => "php".to_string(),
        "static" => "node".to_string(),
        _ => "unknown".to_string(),
    }
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

fn unique_path(base: &Path, name: &str) -> PathBuf {
    let mut candidate = base.join(name);
    let mut index = 2;
    while candidate.exists() {
        candidate = base.join(format!("{}-{}", name, index));
        index += 1;
    }
    candidate
}

fn normalize_created_project_type(project_type: &str) -> Result<String, String> {
    let normalized = project_type.trim().to_lowercase();
    let project_type = match normalized.as_str() {
        "vite-react" | "vite-vanilla" | "vite" => "vite",
        "next.js" | "next" => "next",
        "astro" => "astro",
        "static-html" | "static" => "static",
        "empty-node" | "node" => "node",
        "php-basic" | "php" => "php",
        _ => return Err(
            "Unsupported project type. Choose Vite, Next.js, Astro, Static HTML, Node.js or PHP."
                .to_string(),
        ),
    };
    validate_project_type(project_type)?;
    Ok(project_type.to_string())
}

fn safe_project_folder_name(name: &str) -> String {
    let sanitized = name
        .trim()
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            ch if ch.is_control() => '-',
            ch => ch,
        })
        .collect::<String>()
        .trim_matches([' ', '.'])
        .to_string();
    if sanitized.is_empty() {
        "project".to_string()
    } else {
        sanitized
    }
}

fn write_project_wizard_template(
    requested_type: &str,
    project_type: &str,
    name: &str,
    target: &Path,
) -> Result<(), String> {
    match requested_type.trim().to_lowercase().as_str() {
        "vite-react" | "vite" if project_type == "vite" => {
            fs::create_dir_all(target.join("src")).map_err(|error| error.to_string())?;
            fs::write(target.join("package.json"), vite_package())
                .map_err(|error| error.to_string())?;
            fs::write(
                target.join("index.html"),
                "<div id=\"root\"></div><script type=\"module\" src=\"/src/main.tsx\"></script>\n",
            )
            .map_err(|error| error.to_string())?;
            fs::write(
                target.join("src").join("main.tsx"),
                format!(
                    "import React from 'react';\nimport ReactDOM from 'react-dom/client';\nimport './style.css';\n\nReactDOM.createRoot(document.getElementById('root')!).render(<h1>{}</h1>);\n",
                    name
                ),
            )
            .map_err(|error| error.to_string())?;
            fs::write(
                target.join("src").join("style.css"),
                "body{font-family:Inter,Segoe UI,Arial,sans-serif;margin:40px;background:#111;color:#f8fafc;}\n",
            )
            .map_err(|error| error.to_string())?;
            fs::write(target.join("vite.config.ts"), "import { defineConfig } from 'vite';\nimport react from '@vitejs/plugin-react';\nexport default defineConfig({ plugins: [react()] });\n").map_err(|error| error.to_string())?;
        }
        "vite-vanilla" => {
            fs::write(target.join("package.json"), vite_vanilla_package())
                .map_err(|error| error.to_string())?;
            fs::write(
                target.join("index.html"),
                format!(
                    "<!doctype html><html><head><title>{}</title><link rel=\"stylesheet\" href=\"/src/style.css\"></head><body><main><h1>{}</h1></main><script type=\"module\" src=\"/src/main.js\"></script></body></html>\n",
                    name, name
                ),
            )
            .map_err(|error| error.to_string())?;
            fs::create_dir_all(target.join("src")).map_err(|error| error.to_string())?;
            fs::write(
                target.join("src").join("main.js"),
                "console.log('Vite vanilla project ready');\n",
            )
            .map_err(|error| error.to_string())?;
            fs::write(
                target.join("src").join("style.css"),
                "body{font-family:Inter,Segoe UI,Arial,sans-serif;margin:40px;background:#111;color:#f8fafc;}\n",
            )
            .map_err(|error| error.to_string())?;
        }
        "next" | "next.js" => {
            fs::create_dir_all(target.join("app")).map_err(|error| error.to_string())?;
            fs::write(target.join("package.json"), next_package(false))
                .map_err(|error| error.to_string())?;
            fs::write(
                target.join("app").join("page.tsx"),
                format!(
                    "export default function Page() {{\n  return <main><h1>{}</h1></main>;\n}}\n",
                    name
                ),
            )
            .map_err(|error| error.to_string())?;
            fs::write(target.join("app").join("layout.tsx"), "import type { ReactNode } from 'react';\n\nexport default function RootLayout({ children }: { children: ReactNode }) {\n  return <html lang=\"en\"><body>{children}</body></html>;\n}\n").map_err(|error| error.to_string())?;
            fs::write(
                target.join("next.config.mjs"),
                "const nextConfig = {};\nexport default nextConfig;\n",
            )
            .map_err(|error| error.to_string())?;
        }
        "astro" => {
            fs::create_dir_all(target.join("src").join("pages"))
                .map_err(|error| error.to_string())?;
            fs::write(target.join("package.json"), astro_package())
                .map_err(|error| error.to_string())?;
            fs::write(
                target.join("src").join("pages").join("index.astro"),
                format!(
                    "---\nconst title = '{}';\n---\n<html><head><title>{{title}}</title></head><body><main><h1>{{title}}</h1></main></body></html>\n",
                    name
                ),
            )
            .map_err(|error| error.to_string())?;
        }
        "static-html" | "static" => {
            fs::write(target.join("index.html"), format!("<!doctype html><html><head><title>{}</title><link rel=\"stylesheet\" href=\"styles.css\"></head><body><main><h1>{}</h1></main><script src=\"script.js\"></script></body></html>\n", name, name)).map_err(|error| error.to_string())?;
            fs::write(
                target.join("styles.css"),
                "body{font-family:Inter,Segoe UI,Arial,sans-serif;margin:40px;background:#111;color:#f8fafc;}\n",
            )
            .map_err(|error| error.to_string())?;
            fs::write(
                target.join("script.js"),
                "console.log('Static project ready');\n",
            )
            .map_err(|error| error.to_string())?;
        }
        "empty-node" | "node" => {
            fs::write(target.join("package.json"), node_package())
                .map_err(|error| error.to_string())?;
            fs::write(target.join("server.js"), "const http = require('http');\nconst port = process.env.PORT || 3000;\nhttp.createServer((_, res) => {\n  res.writeHead(200, { 'content-type': 'text/html' });\n  res.end('<h1>Node.js project ready</h1>');\n}).listen(port, '0.0.0.0', () => console.log(`Node server ready on ${port}`));\n").map_err(|error| error.to_string())?;
        }
        "php-basic" | "php" => {
            fs::create_dir_all(target.join("assets").join("css"))
                .map_err(|error| error.to_string())?;
            fs::create_dir_all(target.join("assets").join("js"))
                .map_err(|error| error.to_string())?;
            fs::write(
                target.join("index.php"),
                format!("<?php $title = '{}'; ?><!doctype html><html><head><title><?= $title ?></title><link rel=\"stylesheet\" href=\"assets/css/style.css\"></head><body><main><h1><?= $title ?></h1></main><script src=\"assets/js/app.js\"></script></body></html>\n", name),
            )
            .map_err(|error| error.to_string())?;
            fs::write(
                target.join("assets").join("css").join("style.css"),
                "body{font-family:Inter,Segoe UI,Arial,sans-serif;margin:40px;background:#111;color:#f8fafc;}\n",
            )
            .map_err(|error| error.to_string())?;
            fs::write(
                target.join("assets").join("js").join("app.js"),
                "console.log('PHP project ready');\n",
            )
            .map_err(|error| error.to_string())?;
        }
        _ => return Err("Unsupported project template.".to_string()),
    }
    Ok(())
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

fn vite_vanilla_package() -> &'static str {
    "{\n  \"scripts\": { \"dev\": \"vite\" },\n  \"dependencies\": { \"vite\": \"latest\" },\n  \"devDependencies\": {}\n}\n"
}

fn astro_package() -> &'static str {
    "{\n  \"scripts\": { \"dev\": \"astro dev\" },\n  \"dependencies\": { \"astro\": \"latest\" },\n  \"devDependencies\": {}\n}\n"
}

fn node_package() -> &'static str {
    "{\n  \"scripts\": { \"dev\": \"node server.js\" },\n  \"dependencies\": {},\n  \"devDependencies\": {}\n}\n"
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
    use crate::services::project_detector::{ensure_node_modules, package_json_has_script};
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
    fn validate_project_type_accepts_known_types() {
        assert!(validate_project_type("next").is_ok());
        assert!(validate_project_type("node").is_ok());
        assert!(validate_project_type("unknown").is_ok());
        assert!(validate_project_type("rails").is_err());
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
    fn detect_lockfile_prefers_project_lockfile() {
        let root = temp_project("lockfile");
        fs::write(root.join("pnpm-lock.yaml"), "lockfile").unwrap();
        assert_eq!(
            services::project_detector::detect_lockfile(&root).unwrap(),
            "pnpm-lock.yaml"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ensure_node_modules_requires_explicit_install() {
        let root = temp_project("missing-modules");
        let error = ensure_node_modules(root.to_str().unwrap(), "pnpm").unwrap_err();
        assert!(error.contains("Dependencies are missing"));
        fs::create_dir_all(root.join("node_modules")).unwrap();
        ensure_node_modules(root.to_str().unwrap(), "pnpm").unwrap();
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
            package_manager: None,
            use_docker: false,
            dev_port: None,
            proxy_port: None,
            last_started_at: None,
            last_error: None,
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
        assert!(services::hosting_compatibility::contains_localhost(
            "const api = 'http://localhost:3000';"
        ));
        assert!(services::hosting_compatibility::contains_windows_path(
            r#"include "C:\Users\demo\file.php";"#
        ));
        assert!(!services::hosting_compatibility::contains_windows_path(
            "/var/www/html/index.php"
        ));
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
