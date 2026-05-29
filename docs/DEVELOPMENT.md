# Development

## Commands

```bash
pnpm typecheck
pnpm test
pnpm build
pnpm tauri dev
pnpm tauri build
```

Rust checks:

```bash
cd src-tauri
cargo check
cargo test
```

## Structure

- `src/` contains React UI.
- `src/lib/` contains API, types, i18n and constants.
- `src/components/ui/` contains shared UI primitives.
- `src-tauri/src/` contains Rust commands, migrations and process logic.
- `docs/` contains Markdown docs.
- `docs-site/` contains static HTML documentation.

## Adding a Tauri Command

1. Add the Rust function with `#[tauri::command]`.
2. Register it in `tauri::generate_handler!`.
3. Add the TypeScript wrapper in `src/lib/api.ts`.
4. Add types in `src/lib/types.ts`.
5. Add tests where practical.
