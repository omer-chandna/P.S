import { AppIcon } from "@/components/brand/Mark";
import { approximateTokens, formatDuration, statusLabel } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { OptimizationStatus } from "@/types";

export interface ResultViewProps {
  status: OptimizationStatus | "idle" | "error";
  original: string;
  rewritten?: string;
  warnings?: string[];
  message?: string;
  durationMs?: number;
  promptTokens?: number | null;
  completionTokens?: number | null;
  suspicious?: boolean;
  onCopy?: (text: string) => void;
  onChangeRewritten?: (value: string) => void;
  className?: string;
}

export function ResultView({
  status,
  original,
  rewritten = "",
  warnings = [],
  message,
  durationMs,
  promptTokens,
  completionTokens,
  suspicious = false,
  onCopy,
  onChangeRewritten,
  className,
}: ResultViewProps) {
  if (status === "idle") {
    return (
      <div
        className={cn(
          "ps-phosphor flex h-full min-h-64 flex-col items-center justify-center gap-3 px-6 text-center",
          className,
        )}
      >
        <AppIcon framed className="h-12 w-12" />
        <p className="text-sm text-muted">No result yet</p>
      </div>
    );
  }

  const meta = [
    durationMs !== undefined ? formatDuration(durationMs) : null,
    promptTokens != null || completionTokens != null
      ? `${promptTokens ?? "—"} in / ${completionTokens ?? "—"} out`
      : null,
  ]
    .filter(Boolean)
    .join(" · ");

  return (
    <section
      className={cn(
        "flex h-full min-h-64 flex-col overflow-hidden",
        status === "error" ? "ps-alarm" : "ps-phosphor",
        className,
      )}
    >
      <header className="flex shrink-0 items-center justify-between gap-3 px-4 py-3">
        <div className="flex items-center gap-2">
          <p
            className={cn(
              "rounded-md px-2 py-0.5 text-[11px] font-semibold",
              status === "error" ? "bg-danger text-bg" : "bg-accent text-accent-ink",
            )}
          >
            {status === "error" ? "Could not optimize" : statusLabel(status)}
          </p>
          {suspicious ? <span className="text-xs font-medium text-warn">Review</span> : null}
        </div>
        {meta ? <p className="font-mono text-[11px] text-muted">{meta}</p> : null}
      </header>
      <div className="@container min-h-0 flex-1 overflow-auto px-4 pb-4">
        {status === "error" ? (
          <div>
            <p className="text-sm leading-6">{message}</p>
            <pre className="mt-3 whitespace-pre-wrap break-words font-mono text-[13px] leading-6 text-muted">
              {original}
            </pre>
            <p className="mt-3 text-sm text-muted">The original was left unchanged.</p>
          </div>
        ) : null}

        {status === "unchanged" ? (
          <div>
            <h2 className="text-xs font-medium text-muted">Original</h2>
            <pre className="mt-2 whitespace-pre-wrap break-words font-mono text-[13px] leading-6">{original}</pre>
            <p className="mt-3 text-sm text-muted">Nothing was pasted.</p>
          </div>
        ) : null}

        {status === "rewritten" ? (
          <div className="grid gap-4 @min-[36rem]:grid-cols-2">
            <div className="min-w-0">
              <h2 className="text-xs font-medium text-muted">Original</h2>
              <pre className="mt-2 whitespace-pre-wrap break-words font-mono text-[13px] leading-6 text-muted">
                {original}
              </pre>
            </div>
            <div className="min-w-0">
              <div className="flex items-center justify-between gap-2">
                <h2 className="text-xs font-medium">Optimized</h2>
                {onCopy ? (
                  <button
                    type="button"
                    className="cursor-pointer text-xs font-medium text-accent underline decoration-transparent underline-offset-4 transition-colors duration-150 hover:decoration-current"
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
                  className="ps-editor mt-2 min-h-40 resize-y"
                />
              ) : (
                <pre className="mt-2 whitespace-pre-wrap break-words font-mono text-[13px] leading-6">{rewritten}</pre>
              )}
              <p className="mt-2 font-mono text-[11px] text-muted">~{approximateTokens(rewritten)} tok</p>
            </div>
          </div>
        ) : null}

        {warnings.length > 0 ? (
          <ul className="mt-4 space-y-1 border-t border-accent/30 pt-3 text-xs leading-5">
            {warnings.map((warning) => (
              <li key={warning}>{warning}</li>
            ))}
          </ul>
        ) : null}
      </div>
    </section>
  );
}
