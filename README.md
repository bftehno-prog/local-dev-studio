# Local Dev Studio

Local Dev Studio is a Windows desktop app built with Tauri 2, React, TypeScript, Rust and SQLite. It manages local dev servers for Next.js, Vite, Astro, static HTML/CSS/JS and PHP projects.

> Screenshot placeholder: Dashboard
> Screenshot placeholder: Projects
> Screenshot placeholder: Live Preview
> Screenshot placeholder: Settings

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
pnpm typecheck
pnpm test
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
- Native folder picker for adding existing projects.
- Project type detection for Next.js, Vite, Astro, static HTML and PHP.
- Real process launch for Next.js, Vite/Astro, PHP and static HTML.
- Safe command construction with project type and package manager allow-lists.
- Project path and environment variable validation before process launch.
- Automatic free-port search in the configured range.
- Live preview iframe with device widths, external browser open and QR code.
- Live preview server selector, manual URL field, iframe reload, local/network URLs and network QR code.
- Live preview fit/actual scaling modes with server health display.
- English/Russian interface localization with a saved language switch in Settings.
- English/Russian tray menu labels.
- Sandbox creation from built-in templates.
- Template list, ZIP import by path or native picker, ZIP drag and drop, duplication, delete for user templates and export for folder-backed user templates.
- Server list with PID, port, URL, status and memory/CPU sampling.
- Stale process cleanup, startup port readiness checks and Windows process-tree stop.
- Port manager with managed-port release and warning for external processes.
- Log center with filters, search, clear and `.txt` export.
- Runtime, package manager, environment variable, Next.js, Preview, Advanced and Diagnostics settings.
- Tray icon with dashboard/settings/start all/stop all/exit entries.
- GitHub Actions check and release workflows.
- Basic Rust and React UI tests.

## Supported Project Types

- Next.js
- Vite
- Astro
- PHP
- Static HTML/CSS/JS

## Live Preview and Network Preview

Live Preview embeds local project URLs in the app, supports refresh/copy/open-in-browser actions, and can show a LAN URL with QR code when network preview is enabled. LAN IP detection uses Windows network configuration and falls back to `127.0.0.1`.

## Project Structure

- `src/` React frontend.
- `src/app/` navigation and frontend constants.
- `src/components/ui/` shared UI primitives and UI tests.
- `src-tauri/src/` Rust backend.
- `src-tauri/src/db/` SQLite migrations.
- `docs/` Markdown documentation.
- `docs-site/` static HTML/CSS documentation.

## Current Limitations

- HTTPS preview, proxy rules, hosts and SSL certificate controls are marked as in development and disabled in the UI.
- Auto-update is documented in `docs/updater.md`, but not exposed in the UI until signing keys and release hosting are configured.
- Bundled Node and pnpm binaries are not included in this repository. Put `node.exe` and `pnpm.cmd` in `src-tauri/binaries` to ship a self-contained runtime.
- Rust and Cargo must be installed locally to run `pnpm tauri dev` and `pnpm tauri build`.

## Release

```bash
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test
pnpm build
cd src-tauri
cargo check
cargo test
cd ..
pnpm tauri build
```

Windows installers are generated under `src-tauri/target/release/bundle`.

## Security Model

Local Dev Studio does not accept arbitrary shell commands from the UI. Commands are assembled by the Rust backend from:

- a supported project type: `next`, `vite`, `astro`, `php`, `static`;
- a supported package manager: `npm`, `pnpm`, `yarn`, `bun`;
- a validated project directory;
- validated `KEY=value` environment variables.

Environment variable values are passed to child processes but only keys are written to logs.

## Troubleshooting

- If a project does not start, open Logs and check the project-specific `server` and `error` entries.
- If the preview is blank, verify the server URL in Live Preview and use the external browser button.
- If a port is occupied by an external process, Local Dev Studio will not kill it automatically.
- If installer build fails with access denied, close the running `local-dev-studio.exe` and build again.

## Documentation

- [Markdown docs](docs/README.md)
- [Static HTML docs](docs-site/index.html)
- [Roadmap](docs/ROADMAP.md)
- [Changelog](docs/CHANGELOG.md)
- [Updater notes](docs/updater.md)

## Roadmap

See [docs/ROADMAP.md](docs/ROADMAP.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

No open-source license has been selected yet. Treat the repository as all rights reserved until a license is added.

## Production Readiness Checklist

- [x] CSP configured.
- [x] Installer builds on Windows.
- [x] Tray Start All / Stop All actions work through backend commands.
- [x] Native folder and ZIP pickers are available.
- [x] Unsafe or unfinished controls are disabled and marked as in development.
- [x] Backend and frontend tests are configured.
- [x] GitHub Actions check and release workflows are present.
- [ ] Tauri updater is fully configured with production signing keys.
- [ ] Bundled Node/pnpm runtime is shipped with release artifacts.

## Notes

Local Dev Studio never accepts arbitrary shell commands from the UI. Commands are assembled by the backend from the detected project type and saved settings.
