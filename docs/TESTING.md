# Testing

Automated tests do not measure whether a particular Ollama model writes good prompts. They check that the application handles structured results, protects technical references, and refuses unsafe replacement.

## Commands

```bash
cargo test -p ps-core
cargo clippy -p ps-core -p ps-desktop --all-targets -- -D warnings
npm run lint
npm run typecheck
npm test
npm run build
```

`ps-core` tests include a local mock server for `/api/tags`, `/api/chat`, `/api/pull`, and `/api/delete`. No test downloads a model.

The desktop crate tests the clipboard sentinel: copy is detected even when the previous clipboard is restored afterwards, and a blank copy is reported as no selection.

## What the cloud environment verified

On the Linux development host:

- `cargo test -p ps-core` passed, including mocked Ollama timeouts, cancellation, and the representative fixtures.
- `cargo clippy` passed for `ps-core` and `ps-desktop` on the host target.
- Frontend lint, typecheck, Vitest, and the production Vite build passed.
- `cargo check -p ps-desktop` passed for the Linux host. Capture calls return the unsupported-platform error there.
- The first GitHub Actions run compiled and bundled both targets: `PS_0.1.0_aarch64.dmg` (6.78 MiB) and `PS_0.1.0_x64-setup.exe` (4.49 MiB). The upload step looked in `src-tauri/target` instead of the workspace `target` directory. That path is corrected. Installing those bundles and using them in other applications is still unverified.

The interface was exercised in a browser preview of the production bundle: onboarding, home, models, settings, and about. That preview cannot register a global shortcut or paste into another application.

## Manual checks still required

Run these on Apple Silicon macOS 14+ and on Windows 11 after installing Ollama. Do not treat a mocked test as a substitute.

| Check | Expected |
| --- | --- |
| Shortcut from the menu bar or tray, with P.S. unfocused | The selected prompt is read |
| Cursor composer, VS Code, a browser field, and a native editor | Copy and paste succeed where the control supports them |
| Multiline and long selections | The full selection is preserved |
| Already clear prompt | No paste, “Already clear” notification |
| Clarification result | The question is shown in P.S., not inserted into the editor |
| Rewrite with preview off | The selection is replaced and undo restores it where the editor allows |
| Switch apps during inference | No paste; review or cancellation |
| Change the selection during inference | No paste |
| Clipboard had an image or other text | Text is restored when P.S. can; mixed clipboard content is best-effort |
| Ollama stopped | The original text remains, with a recoverable error |
| Second shortcut while one run is active | The second run is rejected |
| Missing macOS Accessibility permission | Settings guidance, no deletion of the selection |
| Shortcut already taken by the OS | A conflict error, previous shortcut kept |
| Quit during inference | No paste |

Record the OS version, Ollama version, and model name next to any manual result. Model quality is judged in the Test Model page, one prompt at a time, without an automated score.
