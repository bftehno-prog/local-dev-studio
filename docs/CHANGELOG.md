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
- Frontend shared API, hooks, preview and reusable data-view modules.
- Rust `state`, `security` and `utils` modules.

### Changed

- Polling is split by data type instead of loading all data every two seconds.
- Network preview IP detection no longer depends on Google DNS.
- README now includes production readiness details.
- `App.tsx` and `src-tauri/src/lib.rs` are smaller after incremental module extraction.
- Package manager settings are validated against an allow-list before saving.

### Fixed

- Project startup remains non-blocking while status is monitored in the background.
