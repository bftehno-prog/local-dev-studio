# Changelog

## Unreleased

### Added

- CSP configuration for Tauri.
- Native folder and ZIP pickers.
- ZIP drag and drop.
- Diagnostics panel.
- GitHub Actions check and release workflows.
- SQLite schema migrations.
- Markdown and HTML documentation.
- ESLint configuration and `pnpm lint`.
- Prettier configuration and `pnpm format:check`.
- Frontend shared API, hooks, preview and reusable data-view modules.
- Rust `state`, `security` and `utils` modules.
- Runtime manager commands and Settings runtime health table.
- Dedicated Diagnostics page with copyable issue report.
- Project Doctor command and Projects page action.
- Trusted Project Mode with SQLite trust state and Projects page trust controls.
- First-run onboarding screen backed by persisted settings.
- Project Detail panel with runtime, command, preview URL, trust state, actions and recent logs.
- Shared Hosting Compatibility Checker for PHP/static projects.
- ZIP template preflight validation before extraction.
- Explicit dependency installation action for Node.js projects.
- Feature modules for Dashboard, Servers, Ports and Logs pages.
- Feature modules for Diagnostics, Templates and Sandboxes pages.
- Feature modules for Projects and Onboarding pages.
- Feature module for Settings page.
- App shell layout modules for Sidebar and Topbar.
- App entry component moved to `src/app/App.tsx`.
- Rust runtime resolver service module.
- Rust project detector service module.
- Rust shared-hosting compatibility service module.
- Rust project doctor service module.
- Rust command builder service module.
- Rust project cache service module.
- Rust models module with project, settings, log and process DTOs.
- SQLite storage migration for terminal sessions, recent files and future preview/Docker project metadata.
- Backend project wizard commands for create/import/get/update/delete project flows.
- Frontend project wizard split into typed step components.
- Rust process manager service for PID, status and process-tree lifecycle helpers.
- Rust log service for list, clear, export, append and retention helpers.
- Rust port manager service for port lists and LAN preview URL helpers.
- Server process listing moved into the Rust process manager service.

### Changed

- Polling is split by data type instead of loading all data every two seconds.
- Network preview IP detection no longer depends on Google DNS.
- README now includes production readiness details.
- `App.tsx` and `src-tauri/src/lib.rs` are smaller after incremental module extraction.
- Package manager settings are validated against an allow-list before saving.
- Settings are grouped into tabs without changing the visual language.
- CI and release workflows now run format/lint/clippy gates; tagged releases publish installers with checksums.
- Runtime diagnostics now expose source, path, version and last checked timestamp for supported tools.
- Projects can now run a basic doctor check for path, type, runtime, dev script and entrypoint readiness.
- Starting untrusted projects is blocked until the user explicitly trusts the project.
- New users see runtime, folder, port and preview checks before entering the main workspace.
- The Projects screen now exposes selected project details without leaving the existing workflow.
- PHP/static projects can be scanned for localhost URLs, Windows paths, mixed content and common shared-hosting entrypoint issues.
- Template ZIP archives are now fully checked for structure, traversal paths, file count and expanded size before any files are written.
- Starting a Node.js project no longer runs package-manager install implicitly; missing dependencies now produce a clear action-oriented error.
- `App.tsx` now delegates Dashboard, Servers, Ports and Logs screens to feature pages without changing the UI.
- `App.tsx` now delegates Diagnostics, Templates and Sandboxes screens to feature pages without changing behavior.
- `App.tsx` now delegates Projects and Onboarding screens to feature pages without changing behavior.
- `App.tsx` now delegates Settings to a feature page and no longer owns page-local form controls.
- `App.tsx` now delegates sidebar, topbar and shell composition to `src/app/layout`.
- Root `src/App.tsx` is now a compatibility re-export for the app-level component.
- Runtime resolution, runtime PATH construction and version helpers moved out of `src-tauri/src/lib.rs`.
- Project type detection, dev-script checks and dependency readiness helpers moved out of `src-tauri/src/lib.rs`.
- Shared-hosting compatibility scanning moved out of `src-tauri/src/lib.rs`.
- Project doctor report construction moved out of `src-tauri/src/lib.rs`.
- Project launch command construction and environment parsing moved out of `src-tauri/src/lib.rs`.
- Project cache cleanup moved out of `src-tauri/src/lib.rs`.
- Shared Rust data models moved out of `src-tauri/src/lib.rs`.
- Project records now carry package manager, Docker, dev/proxy port and last-run metadata.
- Process startup monitoring and stop helpers moved out of `src-tauri/src/lib.rs`.
- Log storage operations moved out of `src-tauri/src/lib.rs`.
- Port list construction moved out of `src-tauri/src/lib.rs`.
- Server process SQL, stale cleanup and CPU/memory sampling moved out of `src-tauri/src/lib.rs`.

### Fixed

- Project startup remains non-blocking while status is monitored in the background.
- Process log streaming avoids the clippy `Lines::flatten()` pitfall.
- Project start is less likely to appear frozen because dependency installation is a separate background action.
