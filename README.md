# Local Dev Studio

Local Dev Studio is a Windows desktop app built with Tauri 2, React, TypeScript, Rust and SQLite. It manages local dev servers for Next.js, Vite, Astro, static HTML/CSS/JS and PHP projects.

## Requirements

- Node.js LTS or newer
- pnpm 10
- Rust stable with Cargo
- Git
- PHP only when PHP projects are used

The app is prepared for bundled runtime files in `src-tauri/binaries/node.exe` and `src-tauri/binaries/pnpm.cmd`. If those files are absent, it falls back to the system runtime configured in Settings.

## Development

```bash
corepack enable
corepack prepare pnpm@10.0.0 --activate
pnpm install
pnpm tauri dev
```

## Windows Installer

```bash
pnpm tauri build
```

The installer is produced by Tauri under `src-tauri/target/release/bundle`.

## Implemented Features

- Dashboard with project counts, occupied ports and runtime versions.
- SQLite storage for projects, settings, processes, logs, templates, sandboxes and ports.
- Project manager with add, remove, open folder, open VS Code, start, stop, restart, cache clear and live preview.
- Project type detection for Next.js, Vite, Astro, static HTML and PHP.
- Real process launch for Next.js, Vite/Astro, PHP and static HTML.
- Automatic free-port search in the configured range.
- Live preview iframe with device widths, external browser open and QR code.
- Live preview server selector, manual URL field, iframe reload, local/network URLs and network QR code.
- Live preview fit/actual scaling modes with server health display.
- English/Russian interface localization with a saved language switch in Settings.
- English/Russian tray menu labels.
- Sandbox creation from built-in templates.
- Template list, ZIP import by path, duplication, delete for user templates and export for folder-backed user templates.
- Server list with PID, port, URL, status and memory/CPU sampling.
- Stale process cleanup, startup port readiness checks and Windows process-tree stop.
- Port manager with managed-port release and warning for external processes.
- Log center with filters, search, clear and `.txt` export.
- Runtime, package manager, environment variable, Next.js, Preview and Advanced settings.
- Tray icon with dashboard/settings/exit entries.

## Current Limitations

- ZIP import currently accepts a typed file path; a native file picker would be a useful next UI improvement.
- Tray menu entries `Start All` and `Stop All` are present in the native menu, but bulk actions are currently exposed in the Servers screen.
- Bundled Node and pnpm binaries are not included in this repository. Put `node.exe` and `pnpm.cmd` in `src-tauri/binaries` to ship a self-contained runtime.
- Rust and Cargo must be installed locally to run `pnpm tauri dev` and `pnpm tauri build`.

## Notes

Local Dev Studio never accepts arbitrary shell commands from the UI. Commands are assembled by the backend from the detected project type and saved settings.
