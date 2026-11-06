import { useState } from "react";
import { ResultView } from "@/components/optimizer/ResultView";
import { Button } from "@/components/ui/button";
import { asFailure, cancelOptimization, copyText, optimizeText } from "@/lib/api";
import { approximateTokens } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { OptimizationMode, OptimizeOutcome, ShortcutInfo } from "@/types";

export function Home({
  modelName,
  defaultMode,
  shortcuts,
}: {
  modelName: string | null;
  defaultMode: OptimizationMode;
  shortcuts: ShortcutInfo | null;
}) {
  const [text, setText] = useState("");
  const [mode, setMode] = useState<OptimizationMode>(defaultMode);
  const [busy, setBusy] = useState(false);
  const [outcome, setOutcome] = useState<OptimizeOutcome | null>(null);
  const [edited, setEdited] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  async function run() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const next = await optimizeText(text, mode, modelName ?? undefined);
      setOutcome(next);
      setEdited(next.rewrittenPrompt);
    } catch (caught) {
      setOutcome(null);
      setError(asFailure(caught).message);
    } finally {
      setBusy(false);
    }
  }

  const shortcut =
    mode === "compact" ? shortcuts?.compactDisplay : shortcuts?.improveDisplay;

  return (
    <div className="grid h-full min-h-0 gap-4 xl:grid-cols-2">
      <section className="flex min-h-0 flex-col">
        <div className="mb-3 flex items-center justify-between gap-3">
          <div>
            <h2 className="text-base font-medium">Manual optimizer</h2>
            <p className="text-sm text-muted">
              {modelName ? `Using ${modelName}` : "Choose a model before running inference."}
            </p>
          </div>
          <div className="flex rounded-lg border border-line p-0.5">
            {(["improve", "compact"] as const).map((item) => (
              <button
                key={item}
                type="button"
                onClick={() => setMode(item)}
                className={cn(
                  "rounded-md px-3 py-1 text-sm capitalize",
                  mode === item ? "bg-surface-2 text-ink" : "text-muted",
                )}
              >
                {item}
              </button>
            ))}
          </div>
        </div>
        <textarea
          aria-label="Original prompt"
          value={text}
          onChange={(event) => setText(event.target.value)}
          placeholder="Paste a prompt to improve, or leave a clear one unchanged."
          className="min-h-72 flex-1 resize-none rounded-2xl border border-line bg-surface px-4 py-3 leading-6 outline-none focus:border-accent"
        />
        <div className="mt-3 flex items-center justify-between gap-3">
          <p className="text-xs text-muted">
            {text.length} characters · about {text.trim() ? approximateTokens(text) : 0} tokens
            {shortcut ? ` · ${shortcut}` : ""}
          </p>
          <div className="flex gap-2">
            {busy ? (
              <Button variant="secondary" onClick={() => void cancelOptimization()}>
                Cancel
              </Button>
            ) : null}
            <Button onClick={() => void run()} disabled={busy || text.trim().length === 0}>
              {busy ? "Optimizing…" : "Optimize"}
            </Button>
          </div>
        </div>
        {notice ? <p className="mt-2 text-sm text-ok">{notice}</p> : null}
      </section>
      <ResultView
        status={error ? "error" : outcome ? outcome.status : "idle"}
        original={outcome?.originalText ?? text}
        rewritten={edited}
        question={outcome?.clarificationQuestion}
        warnings={outcome?.warnings}
        message={error ?? undefined}
        durationMs={outcome?.durationMs}
        promptTokens={outcome?.promptTokens}
        completionTokens={outcome?.completionTokens}
        suspicious={outcome?.suspicious}
        onChangeRewritten={outcome?.status === "rewritten" ? setEdited : undefined}
        onCopy={(value) => {
          void copyText(value).then(
            () => setNotice("Copied."),
            (caught: unknown) => setError(asFailure(caught).message),
          );
        }}
      />
    </div>
  );
}
