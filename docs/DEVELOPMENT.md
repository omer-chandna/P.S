# Development log

P.S. is a Tauri desktop app with a shared Rust core. Supported products are Apple Silicon macOS 14+ and Windows 11 x64. Linux is only a development host.

## How to resume

1. Read this file, `docs/ARCHITECTURE.md`, and `docs/TESTING.md`.
2. Run `cargo test -p ps-core` and `npm test`.
3. Continue from the first unchecked item below.
4. Do not download or rank real Ollama models in CI.

## Roadmap

### Phase 1 — Repository foundation

- [x] Rust workspace and `ps-core`
- [x] Versioned prompt resources
- [x] Tauri v2 shell, React, TypeScript, Tailwind, shadcn-style components
- [x] Settings persistence through the Tauri store
- [x] Frontend lint, typecheck, and tests
- [x] GitHub Actions for checks and unsigned installers

### Phase 2 — Ollama engine

- [x] Loopback-only endpoint checks
- [x] Model discovery and cloud-model rejection
- [x] Chat, show, pull progress, and delete
- [x] Timeouts, cancellation, and malformed responses
- [x] Mocked HTTP tests

### Phase 3 — Prompt optimization

- [x] Shared, Improve, and Compact prompts (v1.0.0)
- [x] Separate system and user messages
- [x] JSON schema output
- [x] `rewritten`, `unchanged`, and `needs_clarification`
- [x] Protected-reference checks and suspicious-rewrite routing
- [x] One inference call, no critique pass

### Phase 4 — Model manager

- [x] Presets and conservative hardware suggestions
- [x] Model manager UI, user-started downloads, activation, deletion
- [x] Manual test workspace with status, duration, and token counts

### Phase 5 — Selection and replacement

- [x] Clipboard sentinel rules and delivery planner
- [x] macOS and Windows capture, focus checks, and paste
- [x] No paste for unchanged or clarification results
- [ ] Native verification on macOS and Windows hardware

### Phase 6 — Global shortcuts

- [x] Accelerator parsing, conflict detection, registration, pause
- [x] Single-flight optimization and cancellation
- [ ] Confirm registration against a real OS shortcut conflict

### Phase 7 — Background app

- [x] Menu bar and tray actions
- [x] Hide on close, launch at startup, notifications
- [ ] Confirm the macOS accessory policy and Windows foreground-lock behaviour

### Phase 8 — Interface and onboarding

- [x] Home, Models, Settings, About
- [x] Onboarding, themes, preview-before-replace
- [x] Browser pass of the production bundle

### Phase 9 — Verification

- [x] Core automated tests
- [x] Frontend tests and production bundle
- [ ] Manual GUI matrix in `docs/TESTING.md`

### Phase 10 — Packaging

- [x] Workflow for an Apple Silicon DMG and a Windows NSIS installer
- [x] First CI compile produced `PS_0.1.0_aarch64.dmg` and `PS_0.1.0_x64-setup.exe`
- [ ] Artifact upload path corrected after that run; confirm the next CI upload
- [ ] Signed and notarized release

### Phase 11 — Stabilization

- [x] Core and shell separated; clippy clean on the Linux host
- [ ] Fix anything the first macOS and Windows CI builds report
- [ ] Release candidate after the manual matrix

## Decisions

| Decision | Why |
| --- | --- |
| Core crate separate from Tauri | Tests and a future CLI should not need WebKit |
| Tauri 2.12, Rust 1.90 | Matches the current stable Tauri 2 line. Tauri 3 is still alpha |
| Binary name `postscript` | A binary named `ps` would collide with the Unix process tool |
| Product name `PS` | Avoids a period in installer paths. The interface still says P.S. |
| Loopback endpoints only | Stops the app being pointed at a remote inference service |
| Heuristic preservation, then preview | A fluent rewrite that drops a path or version is not pasted |
| No model benchmark | Quality is compared by hand in Test Model |

## Latest local test status

Recorded while implementing the shell:

- `cargo test -p ps-core` passed (37 unit tests and the mocked HTTP suite).
- `cargo test -p ps-desktop --lib` passed (clipboard sentinel tests).
- `cargo clippy -p ps-core -p ps-desktop --all-targets -- -D warnings` passed on Linux.
- `npm run lint`, `npm run typecheck`, `npm test`, and `npm run build` passed.
- Windows cross-compilation from Linux failed inside the `ring` crate build script before P.S. code was typechecked. That is an environment limit, not a test result.

## Pull requests

- Core engine branch: `cursor/ps-core-engine-500d`
- Desktop shell branch: `cursor/ps-desktop-shell-500d`, based on the core branch

The shell pull request should be reviewed against the core branch so its diff is the app, not the engine again.

## Blockers

- No Apple Silicon or Windows runner is available inside this cloud session, so installers were not produced here.
- Real model quality is intentionally not measured here.

## Next action

Confirm the corrected artifact upload on the next CI run, then run the manual matrix in `docs/TESTING.md` on Apple Silicon macOS and Windows 11.
