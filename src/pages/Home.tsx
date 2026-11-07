import { useState } from "react";
import { MenuMark } from "@/components/brand/Mark";
import { ResultView } from "@/components/optimizer/ResultView";
import { Button } from "@/components/ui/button";
import { Segmented } from "@/components/ui/segmented";
import { asFailure, cancelOptimization, copyText, optimizeText } from "@/lib/api";
import { approximateTokens } from "@/lib/format";
import type { OptimizationMode, OptimizeOutcome, ShortcutInfo } from "@/types";

const MODES: Array<{ value: OptimizationMode; label: string }> = [
  { value: "improve", label: "Improve" },
  { value: "compact", label: "Compact" },
];

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

  const shortcut = mode === "compact" ? shortcuts?.compactDisplay : shortcuts?.improveDisplay;

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3 p-4">
      <h1 className="sr-only">Optimize a prompt</h1>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <Segmented label="Mode" value={mode} options={MODES} onChange={setMode} />
        {shortcut ? <span className="ps-kbd">{shortcut}</span> : null}
      </div>
      <div className="grid min-h-0 flex-1 grid-cols-1 gap-3 sm:grid-cols-2">
        <section className="ps-screen flex min-h-0 flex-col transition-[box-shadow] duration-150">
          <textarea
            aria-label="Original prompt"
            value={text}
            spellCheck={false}
            onChange={(event) => setText(event.target.value)}
            placeholder="Paste a prompt"
            className="min-h-40 flex-1 resize-none bg-transparent px-4 py-3 font-mono text-[13px] leading-6 text-ink caret-accent outline-none placeholder:text-muted"
          />
          <div className="flex items-center justify-between gap-3 border-t border-line px-3 py-2.5">
            <p className="font-mono text-[11px] text-muted">
              {text.length} chars · ~{text.trim() ? approximateTokens(text) : 0} tok
              {notice ? <span className="ml-2 text-ok">{notice}</span> : null}
            </p>
            <div className="flex gap-2">
              {busy ? (
                <Button variant="secondary" onClick={() => void cancelOptimization()}>
                  Cancel
                </Button>
              ) : null}
              <Button onClick={() => void run()} disabled={busy || text.trim().length === 0}>
                <MenuMark onAccent className="h-3.5 w-3.5" />
                {busy ? "Optimizing…" : "Optimize"}
              </Button>
            </div>
          </div>
        </section>
        <ResultView
          status={error ? "error" : outcome ? outcome.status : "idle"}
          original={outcome?.originalText ?? text}
          rewritten={edited}
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
    </div>
  );
}
