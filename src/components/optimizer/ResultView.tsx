import { approximateTokens, formatDuration, statusLabel } from "@/lib/format";
import type { OptimizationStatus } from "@/types";

export interface ResultViewProps {
  status: OptimizationStatus | "idle" | "error";
  original: string;
  rewritten?: string;
  question?: string;
  warnings?: string[];
  message?: string;
  durationMs?: number;
  promptTokens?: number | null;
  completionTokens?: number | null;
  suspicious?: boolean;
  onCopy?: (text: string) => void;
  onChangeRewritten?: (value: string) => void;
}

export function ResultView({
  status,
  original,
  rewritten = "",
  question = "",
  warnings = [],
  message,
  durationMs,
  promptTokens,
  completionTokens,
  suspicious = false,
  onCopy,
  onChangeRewritten,
}: ResultViewProps) {
  if (status === "idle") {
    return (
      <div className="flex min-h-64 items-center justify-center rounded-2xl border border-dashed border-line px-6 text-center text-sm leading-6 text-muted">
        A clear prompt can stay exactly as it is. P.S. only rewrites when that helps the next agent.
      </div>
    );
  }

  if (status === "error") {
    return (
      <section className="rounded-2xl border border-line bg-surface p-4">
        <p className="text-xs font-medium uppercase tracking-[0.14em] text-danger">Could not optimize</p>
        <p className="mt-2 text-sm leading-6">{message}</p>
        <p className="mt-3 text-sm leading-6 text-muted">The original text was left unchanged.</p>
      </section>
    );
  }

  return (
    <section className="flex flex-col gap-3 rounded-2xl border border-line bg-surface p-4">
      <div className="flex items-center justify-between gap-3">
        <p
          className={`text-xs font-medium uppercase tracking-[0.14em] ${
            status === "unchanged" ? "text-ok" : status === "needs_clarification" ? "text-warn" : "text-accent"
          }`}
        >
          {statusLabel(status)}
          {suspicious ? " · review" : ""}
        </p>
        <p className="text-xs text-muted">
          {durationMs !== undefined ? formatDuration(durationMs) : null}
          {promptTokens != null || completionTokens != null
            ? ` · ${promptTokens ?? "—"} in / ${completionTokens ?? "—"} out`
            : null}
        </p>
      </div>

      {status === "needs_clarification" ? (
        <div>
          <h2 className="text-sm font-medium">Clarification</h2>
          <p className="mt-2 text-base leading-7">{question}</p>
          <p className="mt-3 text-sm leading-6 text-muted">
            The selection was not changed. Answer this in your next prompt, or cancel and keep the original.
          </p>
        </div>
      ) : null}

      {status === "unchanged" ? (
        <div>
          <h2 className="text-sm font-medium">Original prompt</h2>
          <pre className="mt-2 whitespace-pre-wrap font-sans text-sm leading-6">{original}</pre>
          <p className="mt-3 text-sm text-muted">Nothing was pasted back into the editor.</p>
        </div>
      ) : null}

      {status === "rewritten" ? (
        <div className="grid min-h-0 flex-1 gap-3 lg:grid-cols-2">
          <div className="min-h-0">
            <h2 className="text-sm font-medium text-muted">Original</h2>
            <pre className="mt-2 max-h-80 overflow-auto whitespace-pre-wrap font-sans text-sm leading-6 text-muted">
              {original}
            </pre>
          </div>
          <div className="min-h-0">
            <div className="flex items-center justify-between">
              <h2 className="text-sm font-medium">Optimized</h2>
              {onCopy ? (
                <button
                  type="button"
                  className="text-xs text-accent"
                  onClick={() => onCopy(rewritten)}
                >
                  Copy
                </button>
              ) : null}
            </div>
            {onChangeRewritten ? (
              <textarea
                aria-label="Optimized prompt"
                value={rewritten}
                onChange={(event) => onChangeRewritten(event.target.value)}
                className="mt-2 min-h-40 w-full resize-y rounded-xl border border-line bg-bg px-3 py-2 text-sm leading-6 outline-none focus:border-accent"
              />
            ) : (
              <pre className="mt-2 max-h-80 overflow-auto whitespace-pre-wrap font-sans text-sm leading-6">
                {rewritten}
              </pre>
            )}
            <p className="mt-2 text-xs text-muted">About {approximateTokens(rewritten)} tokens</p>
          </div>
        </div>
      ) : null}

      {warnings.length > 0 ? (
        <ul className="space-y-1 rounded-xl bg-accent-soft px-3 py-2 text-xs leading-5 text-ink">
          {warnings.map((warning) => (
            <li key={warning}>{warning}</li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}
