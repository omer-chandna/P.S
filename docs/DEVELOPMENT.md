# Development log

P.S. is built as a shared Rust core plus a Tauri desktop shell. Linux is a development host for the core and the interface. Supported products are Apple Silicon macOS 14+ and Windows 11 x64.

## How to resume

1. Read this file and `docs/ARCHITECTURE.md`.
2. Run `cargo test -p ps-core` and `npm test`.
3. Continue from the first unchecked roadmap item.
4. Do not download or rank real Ollama models in cloud CI. Mock the API.

## Roadmap

### Phase 1 — Repository foundation

- [x] Rust workspace and `ps-core` crate
- [x] Versioned prompt resources
- [ ] Tauri v2 shell, React, TypeScript, Tailwind, shadcn-style components
- [ ] Settings persistence
- [ ] Frontend lint, typecheck, and tests
- [ ] GitHub Actions

### Phase 2 — Ollama engine

- [x] Loopback-only endpoint checks
- [x] Model discovery and cloud-model rejection
- [x] Chat, show, pull progress, and delete clients
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

- [x] Presets and conservative hardware suggestions in the core
- [ ] Model manager UI, downloads, activation, deletion, and manual test workspace

### Phase 5 — Selection and replacement

- [x] Clipboard sentinel rules and delivery planner
- [ ] macOS and Windows capture, focus checks, and paste
- [ ] No paste for unchanged or clarification results

### Phase 6 — Global shortcuts

- [x] Accelerator parsing and conflict detection
- [ ] Registration, customization, pause, and single-flight optimization

### Phase 7 — Background app

- [ ] macOS menu bar and Windows tray
- [ ] Hide on close, launch at startup, notifications

### Phase 8 — Interface and onboarding

- [ ] Home, Models, Settings, About
- [ ] Onboarding
- [ ] Themes and preview-before-replace

### Phase 9 — Verification

- [x] Core automated tests on Linux
- [ ] Frontend tests
- [ ] Native GUI checks documented for a local machine

### Phase 10 — Packaging

- [ ] macOS ARM64 DMG workflow
- [ ] Windows 11 installer workflow
- [ ] Signing and notarization notes

### Phase 11 — Stabilization

- [ ] Review after the shell is in place
- [ ] Release candidate notes

## Decisions

| Decision | Why |
| --- | --- |
| Core crate separate from Tauri | The optimizer must be testable without WebKit, and a future CLI can reuse it |
| Tauri v2, not Tauri 3 | Tauri 3 is still alpha. v2 matches the product requirement |
| Rust 1.90 | Matches the Tauri 2.12 minimum |
| No Linux product target | Capture and installers are macOS and Windows. Linux can still run unit tests |
| No bundled models and no cloud inference | Downloads are explicit. Endpoints must be loopback |
| Heuristic preservation, then preview | Fluent but lossy rewrites are not pasted automatically |
| Binary name `postscript` | A binary named `ps` would collide with the Unix process tool |

## Test status

`cargo test -p ps-core` covers templates, parsing, preservation, delivery, shortcuts, hardware suggestions, and a mocked Ollama server. Record the latest run in the commit that changes tests. Frontend and native shells are not covered yet.

## Pull requests

None yet. Branches use the `cursor/<topic>-500d` prefix required by this workspace.

## Blockers

- Native paste, accessibility prompts, and installers cannot be executed in this Linux cloud environment. Implement them and document the manual pass.
- Real model quality is intentionally not measured here.

## Next action

Build the Tauri shell, model manager, shortcut registration, and platform capture/replacement on top of `ps-core`.
