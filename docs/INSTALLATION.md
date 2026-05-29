# Installation

## Requirements

- Windows 10 or newer.
- Node.js 22 LTS or newer.
- pnpm 10 via Corepack.
- Rust stable with Cargo.
- Git.
- PHP only for PHP projects.

## Setup

```bash
corepack enable
corepack prepare pnpm@10.0.0 --activate
pnpm install
```

## First Run

```bash
pnpm tauri dev
```

If Tauri cannot find Rust, install Rust stable from rustup and restart the terminal.

## Common Installation Errors

| Error | Fix |
| --- | --- |
| `pnpm not found` | Enable Corepack or install pnpm globally. |
| `cargo not found` | Install Rust stable and reopen the shell. |
| WebView missing | Install Microsoft Edge WebView2 Runtime. |
