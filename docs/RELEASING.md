# Releasing

P.S. ships as an Apple Silicon DMG and a Windows 11 NSIS installer. Builds are unsigned unless you add your own credentials. Unsigned apps need the usual Gatekeeper or SmartScreen confirmation.

## Local artifacts

From a Mac:

```bash
npm install
npm run installer:mac
```

The DMG is under `target/aarch64-apple-darwin/release/bundle/dmg/`. The workspace target directory is at the repository root, not inside `src-tauri`.

From Windows 11 x64:

```bash
npm install
npm run installer:win
```

The installer is `target/x86_64-pc-windows-msvc/release/bundle/nsis/PS_0.1.0_x64-setup.exe`. It installs for the current user and includes an uninstaller.

Do not build a universal macOS binary. The bundle identifier is `app.postscript.ps`. The minimum macOS version in `tauri.conf.json` is 14.0.

## GitHub Actions

`.github/workflows/ci.yml` runs frontend checks, Rust tests, and the two platform builds on pull requests. `.github/workflows/release.yml` repeats the platform builds for a `v*` tag and opens a draft GitHub release with the unsigned artifacts.

CI does not download Ollama models and does not need signing certificates.

## Signing

Add these as repository secrets when you are ready to notarize. Do not commit them.

macOS:

- `APPLE_CERTIFICATE` — base64 Developer ID Application certificate
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID`
- `APPLE_PASSWORD` — app-specific password
- `APPLE_TEAM_ID`

Windows:

- `WINDOWS_CERTIFICATE`
- `WINDOWS_CERTIFICATE_PASSWORD`

Tauri’s bundler reads the standard `APPLE_*` and Windows certificate environment variables. Until those secrets exist, leave the release workflow unsigned.

After a signed macOS build, notarize and staple the DMG with `notarytool` and `stapler` before publishing the draft release. Confirm the installed app is the Apple Silicon slice (`lipo -info` or `file` on the binary inside the `.app`).

## Version

Bump `version` in `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` together, then tag `vX.Y.Z`.
