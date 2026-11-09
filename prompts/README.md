# P.S. prompt resources

This file is the production system prompt. It is versioned in source.

Current version: **1.3.0** (`<!-- ps-prompt-version: 1.3.0 -->`).

| File | Role |
| --- | --- |
| `shared.md` | The only optimization prompt: clarify, condense, or leave the selection unchanged |

The desktop app loads this file at compile time. The selected user prompt is never concatenated into the system prompt. It is sent as a separate user message.

HTML comments are stripped before inference. The version marker stays in source so logs and the settings screen can show which instructions ran.

When you change behaviour, bump `ps-prompt-version` in the same commit and describe the change in `docs/PROMPT_ENGINE.md`.

This template is an informed starting point. It is not an experimental claim that this wording outperforms every alternative.
