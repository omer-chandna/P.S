import { useEffect, useState } from "react";
import { appInfo, openExternal } from "@/lib/api";
import type { AppInfo } from "@/types";

export function About() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  useEffect(() => {
    void appInfo().then(setInfo);
  }, []);

  return (
    <div className="mx-auto max-w-2xl space-y-5">
      <div>
        <p className="text-3xl font-semibold tracking-tight">P.S.</p>
        <p className="mt-1 text-muted">Local Prompt Optimizer</p>
      </div>
      <p className="text-base leading-7">
        Make every instruction earn its place in the prompt. P.S. improves a selected prompt only when that helps a coding agent finish the task, and it leaves an effective prompt unchanged.
      </p>
      <ul className="space-y-2 text-sm leading-6 text-muted">
        <li>Inference stays on this machine. There is no account and no cloud model fallback.</li>
        <li>Selected prompts are not saved to disk or written to logs.</li>
        <li>A rewrite that drops a path, version, or explicit constraint is held for review.</li>
        <li>P.S. never submits the prompt to Cursor, Claude Code, Codex, or any other agent.</li>
      </ul>
      <dl className="grid grid-cols-2 gap-3 text-sm">
        <div>
          <dt className="text-muted">Version</dt>
          <dd>{info?.version ?? "…"}</dd>
        </div>
        <div>
          <dt className="text-muted">Prompt version</dt>
          <dd>{info?.promptVersion ?? "1.2.0"}</dd>
        </div>
        <div>
          <dt className="text-muted">System</dt>
          <dd>
            {info?.os} {info?.arch}
          </dd>
        </div>
        <div>
          <dt className="text-muted">Supported target</dt>
          <dd>{info?.supportedPlatform ? "Yes" : "Development host"}</dd>
        </div>
      </dl>
      {!info?.supportedPlatform ? (
        <p className="text-sm leading-6 text-muted">
          Selection capture and installers target Apple Silicon macOS 14+ and Windows 11 x64. This session can still exercise the optimizer, model manager, and interface.
        </p>
      ) : null}
      <div className="flex flex-wrap gap-3 text-sm">
        <button type="button" className="text-accent" onClick={() => void openExternal("https://ollama.com/download")}>
          Ollama
        </button>
        <button type="button" className="text-accent" onClick={() => void openExternal("https://docs.ollama.com/")}>
          Ollama API
        </button>
        <button
          type="button"
          className="text-accent"
          onClick={() => void openExternal("https://github.com/ollama/ollama")}
        >
          Ollama source
        </button>
      </div>
      <p className="text-xs text-muted">Released under the MIT license.</p>
    </div>
  );
}
