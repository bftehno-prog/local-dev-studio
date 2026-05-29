# Contributing

Thanks for helping improve Local Dev Studio.

## Development

```bash
pnpm install
pnpm typecheck
pnpm test
pnpm build
cd src-tauri
cargo check
cargo test
```

## Guidelines

- Keep the Tauri + React + TypeScript + Rust + SQLite stack.
- Do not add arbitrary shell command execution.
- Keep UI changes within the existing visual language.
- Add tests for backend validation and reusable UI where practical.
- Update docs when behavior changes.

## Pull Requests

Include:

- summary of user-visible changes;
- commands run;
- known limitations or follow-up work.
