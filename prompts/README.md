# P.S. prompt resources

These files are the production system prompts. They are versioned together.

Current version: **1.2.0** (`<!-- ps-prompt-version: 1.2.0 -->` in each file).

| File | Role |
| --- | --- |
| `shared.md` | Governing rules, preservation, grounding, task awareness, and the JSON output contract |
| `improve.md` | Extra instructions for Improve mode |
| `compact.md` | Extra instructions for Compact mode |

The desktop app loads these files at compile time. The selected user prompt is never concatenated into the system prompt. It is sent as a separate user message.

HTML comments are stripped before inference. The version marker stays in source so logs and the settings screen can show which instructions ran.

When you change behaviour, bump `ps-prompt-version` in every file in the same commit and describe the change in `docs/PROMPT_ENGINE.md`.

These templates are an informed starting point. They are not an experimental claim that this wording outperforms every alternative.
