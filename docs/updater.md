# Auto-update preparation

Local Dev Studio is prepared for GitHub Releases, but the Tauri updater is not enabled in the UI yet.

Do not commit private signing keys.

## Generate signing keys

```bash
pnpm tauri signer generate -w ~/.tauri/local-dev-studio.key
```

Save the private key in a secure secret store. The public key can be added to `src-tauri/tauri.conf.json` when updater support is enabled.

## Release flow

1. Update `version` in `package.json` and `src-tauri/tauri.conf.json`.
2. Create release notes.
3. Tag the release, for example `v0.1.1`.
4. Let `.github/workflows/release.yml` build Windows artifacts.
5. Attach the generated `.msi` and NSIS `.exe` to the GitHub Release.

## Future updater config

When update signing keys and release hosting are ready, add Tauri updater plugin configuration and expose an explicit “Check for updates” action in Settings.

Until then, updater controls must remain absent from the UI to avoid a fake or broken feature.
