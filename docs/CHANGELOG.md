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

### Fixed

- Project startup remains non-blocking while status is monitored in the background.
- Process log streaming avoids the clippy `Lines::flatten()` pitfall.
- Project start is less likely to appear frozen because dependency installation is a separate background action.
