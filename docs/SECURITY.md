# Security

## Command Execution

Local Dev Studio does not execute arbitrary commands from the UI. Commands are built from allow-listed project types and package managers.

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

Project paths must exist, be directories and resolve through canonicalization.

## ZIP Import

ZIP imports require `.zip` files. Extraction uses enclosed path checks so archive entries cannot escape the target directory.

## Environment Variables

Environment variables must use `KEY=value`. Values are passed to child processes, but logs include only variable names.

## CSP

The Tauri CSP allows the application itself, Tauri IPC, local preview servers and local iframe preview. Avoid broad remote origins.

## Network Preview Risks

LAN preview exposes the dev server to other devices on the network. Use it only on trusted networks.
