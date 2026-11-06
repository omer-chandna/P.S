# Architecture

P.S. is two crates plus a web interface.

```text
prompts/                 versioned system instructions
crates/ps-core/          Ollama, prompts, validation, delivery decisions
src/                     React interface
src-tauri/               window, tray, shortcuts, clipboard, paste
```

`ps-core` has no Tauri dependency. A future CLI can call the same optimize function. The desktop shell is the only place that touches the clipboard, the focused window, or global shortcuts.

## Optimization path

1. The shell captures the selection and restores the previous clipboard before inference starts.
2. `optimize_prompt` sends one `/api/chat` request. The system message is the shared prompt plus Improve or Compact. The user message wraps the selection.
3. The response must match the JSON schema. Contradictory combinations are rejected. A reply that is only a question leaves the selection unchanged.
4. A rewrite is checked for missing identifiers, paths, URLs, versions, limits, quoted errors, fenced code, and weakened exclusions.
5. `plan_delivery` chooses replace, leave unchanged, or preview.
6. Replace pastes only when the same process is focused and the selection still matches. Anything else opens a review or does nothing to the editor.

Unchanged results never paste. The original selection is not deleted before a successful paste. Paste is one edit in applications that support undo.

## Ollama

The client allows only loopback origins and stops redirects that leave the machine. Installed models come from `GET /api/tags`. Inference uses `POST /api/chat` with `stream: false` and a JSON schema `format`. Downloads use `POST /api/pull` and parse newline-delimited progress. Removal uses `DELETE /api/delete`. Names ending in cloud markers, or records with `remote_host` / `remote_model`, are not runnable.

## Settings

`tauri-plugin-store` saves `settings.json` in the app data directory. The file holds preferences only. It does not hold prompt text. In-memory review state is dropped when the process exits.

## Platforms

`src-tauri/src/platform` hides macOS and Windows behind one host trait. macOS uses Accessibility (`AXIsProcessTrusted`), System Events for the front process, and Command+C / Command+V. Windows uses the foreground window handle and Ctrl+C / Ctrl+V. Other systems return “unsupported” for capture so the manual optimizer can still be developed.

Target identity is the process id captured when the shortcut runs. If that process is no longer focused, replacement is cancelled.

## Interface

The webview is a manual workspace, model manager, settings, onboarding, and review surface. Global shortcuts are handled in Rust so they work while the window is hidden. Closing the window hides it. The tray or menu bar can optimize, pause shortcuts, open Models or Settings, and quit.

## What is intentionally absent

- A second model that classifies or critiques the rewrite
- Automated model scoring or bulk benchmarks
- Accounts, telemetry, and non-local inference
- Linux installers
