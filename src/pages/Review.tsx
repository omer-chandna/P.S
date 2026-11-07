import { useState } from "react";
import { ResultView } from "@/components/optimizer/ResultView";
import { Button } from "@/components/ui/button";
import { acceptAttention, asFailure, copyText, dismissAttention } from "@/lib/api";
import type { Attention } from "@/types";

export function Review({
  attention,
  onClose,
}: {
  attention: Attention;
  onClose: () => void;
}) {
  const [text, setText] = useState(attention.outcome.rewrittenPrompt);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const canReplace =
    attention.pendingId !== null && attention.outcome.status === "rewritten" && text === attention.outcome.rewrittenPrompt;

  return (
    <div className="mx-auto flex min-h-0 w-full max-w-4xl flex-1 flex-col gap-4 p-5">
      <div>
        <h1 className="text-xl font-semibold tracking-tight">Review</h1>
        <p className="mt-1 text-sm leading-6 text-muted">{attention.note}</p>
      </div>
      <ResultView
        className="min-h-0 flex-1"
        status={attention.outcome.status}
        original={attention.outcome.originalText}
        rewritten={text}
        warnings={attention.outcome.warnings}
        durationMs={attention.outcome.durationMs}
        promptTokens={attention.outcome.promptTokens}
        completionTokens={attention.outcome.completionTokens}
        suspicious={attention.outcome.suspicious}
        onChangeRewritten={attention.outcome.status === "rewritten" ? setText : undefined}
        onCopy={(value) => void copyText(value)}
      />
      {error ? <p className="text-sm text-danger">{error}</p> : null}
      <div className="flex gap-2">
        {canReplace ? (
          <Button
            disabled={busy}
            onClick={() => {
              if (!attention.pendingId) {
                return;
              }
              setBusy(true);
              void acceptAttention(attention.pendingId)
                .then(onClose)
                .catch((caught: unknown) => setError(asFailure(caught).message))
                .finally(() => setBusy(false));
            }}
          >
            Replace selection
          </Button>
        ) : null}
        <Button variant="secondary" onClick={() => void copyText(text)}>
          Copy
        </Button>
        <Button
          variant="ghost"
          onClick={() => {
            void dismissAttention().finally(onClose);
          }}
        >
          Keep original
        </Button>
      </div>
    </div>
  );
}
