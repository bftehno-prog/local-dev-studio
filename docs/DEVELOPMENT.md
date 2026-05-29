# Development

## Commands

```bash
pnpm typecheck
pnpm lint
pnpm format:check
pnpm test
pnpm build
pnpm tauri dev
pnpm tauri build
```

Rust checks:

```bash
cd src-tauri
cargo check
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Structure

- `src/` contains React UI.
- `src/app/` contains app-level routing and shared app types.
- `src/features/` contains larger feature panels split out of the app shell.
- `src/shared/` contains shared API adapters, hooks and cross-feature UI helpers.
- `src/lib/` contains legacy API, types, i18n and constants.
- `src/components/ui/` contains shared UI primitives.
- `src-tauri/src/` contains Rust commands, state, security validation, utilities, migrations and process logic.
- Runtime checks are exposed through `check_runtime` and `check_all_runtimes`.
- Project readiness checks are exposed through `project_doctor`.
- First-run onboarding completion is persisted in the settings JSON as `onboarding_completed`.
- `docs/` contains Markdown docs.
- `docs-site/` contains static HTML documentation.

## Adding a Tauri Command

1. Add the Rust function with `#[tauri::command]`.
2. Register it in `tauri::generate_handler!`.
3. Add the TypeScript wrapper in `src/lib/api.ts` and re-export through `src/shared/api/commands.ts` when needed.
4. Add types in `src/lib/types.ts`.
5. Add tests where practical.
