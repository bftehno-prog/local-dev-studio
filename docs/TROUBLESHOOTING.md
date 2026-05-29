# Troubleshooting

| Problem                   | Fix                                                                               |
| ------------------------- | --------------------------------------------------------------------------------- |
| App does not start        | Run `pnpm tauri dev` from a terminal and inspect Rust output.                     |
| Node.js not found         | Set Node path in Settings or install Node.js.                                     |
| Rust not found            | Install Rust stable and reopen PowerShell.                                        |
| Port occupied             | Use Ports to identify managed ports; external ports are not killed automatically. |
| Preview blank             | Refresh preview, open in browser, and check server logs.                          |
| PHP not found             | Set PHP path in Settings.                                                         |
| Install fails             | Run package manager install manually in the project folder and inspect output.    |
| Tauri build access denied | Close `local-dev-studio.exe` before rebuilding.                                   |
| SQLite error              | Check app data folder permissions in Diagnostics.                                 |
| Windows Defender warning  | Sign release builds before distribution.                                          |
