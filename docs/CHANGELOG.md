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

### Changed

- Polling is split by data type instead of loading all data every two seconds.
- Network preview IP detection no longer depends on Google DNS.
- README now includes production readiness details.

### Fixed

- Project startup remains non-blocking while status is monitored in the background.
