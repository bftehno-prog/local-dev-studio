# Security

## Command Execution

Local Dev Studio does not execute arbitrary commands from the UI. Commands are built from allow-listed project types and package managers.

## Trusted Project Mode

Projects start as untrusted. Before a project can run local scripts, the user must mark it as trusted from the Projects page. The trust decision is stored in SQLite with the trusted timestamp and runtime family, and every trust/reset action is written to logs.

Allowed project types:

- `next`
- `vite`
- `astro`
- `php`
- `static`

Allowed package managers:

- `npm`
- `pnpm`
- `yarn`
- `bun`

## Project Paths

Project paths must exist, be directories and resolve through canonicalization. The app rejects path traversal and Windows system/user root folders such as `C:\`, `C:\Windows`, `C:\Program Files`, `C:\Users`, the current user profile root and AppData.

## ZIP Import

ZIP imports require `.zip` files with `template.json` and a `files/` directory. Extraction uses enclosed path checks so archive entries cannot escape the target directory. The importer also enforces archive size, file count, uncompressed size, project type and package manager allow-lists.

## Environment Variables

Environment variables must use `KEY=value`. Keys must use uppercase letters, numbers and underscores and cannot start with a number. Values are passed to child processes, but logs include only variable names.

## CSP

The Tauri CSP allows the application itself, Tauri IPC, local preview servers and local iframe preview. Avoid broad remote origins.

## Network Preview Risks

LAN preview exposes the dev server to other devices on the network. Use it only on trusted networks.
