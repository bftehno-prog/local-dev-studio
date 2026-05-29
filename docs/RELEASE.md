# Release

## Version Bump

Update versions in:

- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml` if needed

## Build

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

## GitHub Release

1. Create a tag such as `v0.1.1`.
2. Push the tag.
3. Let `.github/workflows/release.yml` build artifacts.
4. Attach `.msi` and NSIS `.exe` to the release.

## Signing Keys

Do not commit private signing keys. Store them in GitHub Actions secrets or a secure password manager.

## Updater

Updater preparation is documented in [updater.md](updater.md). It is not enabled in the UI until signing and release metadata are ready.
