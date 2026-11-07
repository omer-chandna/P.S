import { useEffect, useState } from "react";
import { AppIcon } from "@/components/brand/Mark";
import { ConfirmDialog } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ResultView } from "@/components/optimizer/ResultView";
import { Segmented } from "@/components/ui/segmented";
import {
  activateModel,
  asFailure,
  cancelPull,
  deleteModel,
  hardwareRecommendations,
  listModels,
  onPullProgress,
  optimizeText,
  pullModel,
} from "@/lib/api";
import { fitLabel, formatBytes, formatDuration } from "@/lib/format";
import { cn } from "@/lib/utils";
import { SAMPLE_VAGUE } from "@/lib/samples";
import type {
  AppSettings,
  HardwareResponse,
  ListedModel,
  OptimizationMode,
  OptimizeOutcome,
  PullProgress,
} from "@/types";

const MODES: Array<{ value: OptimizationMode; label: string }> = [
  { value: "improve", label: "Improve" },
  { value: "compact", label: "Compact" },
];

export function Models({
  settings,
  onSettings,
}: {
  settings: AppSettings;
  onSettings: (settings: AppSettings) => void;
}) {
  const [models, setModels] = useState<ListedModel[]>([]);
  const [hardware, setHardware] = useState<HardwareResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<string | null>(null);
  const [pull, setPull] = useState<PullProgress | null>(null);
  const [testModel, setTestModel] = useState(settings.activeModel ?? "");
  const [sample, setSample] = useState(SAMPLE_VAGUE);
  const [mode, setMode] = useState<OptimizationMode>("improve");
  const [busy, setBusy] = useState(false);
  const [outcome, setOutcome] = useState<OptimizeOutcome | null>(null);

  async function refresh() {
    try {
      const [listed, advice] = await Promise.all([listModels(), hardwareRecommendations()]);
      setModels(listed);
      setHardware(advice);
      setError(null);
      if (!testModel) {
        setTestModel(settings.activeModel ?? advice.recommendations.suggestedModel ?? "");
      }
    } catch (caught) {
      setError(asFailure(caught).message);
      const advice = await hardwareRecommendations().catch(() => null);
      if (advice) {
        setHardware(advice);
      }
    }
  }

  useEffect(() => {
    void refresh();
    let unlisten: () => void = () => undefined;
    void onPullProgress((progress) => {
      setPull(progress);
      if (progress.done && progress.status === "success") {
        void refresh();
      }
    }).then((stop) => {
      unlisten = () => {
        void stop();
      };
    });
    return () => unlisten();
    // Refresh when the page opens.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function install(model: string) {
    setError(null);
    setPull({ model, status: "starting", percent: 0, done: false, error: null });
    try {
      await pullModel(model);
    } catch (caught) {
      setError(asFailure(caught).message);
    }
  }

  const runnable = models.filter((model) => model.runnable);
  const presets = hardware?.recommendations.presets ?? [];
  const sharedNote =
    presets.length > 0 && presets.every((preset) => preset.note === presets[0]?.note) ? presets[0]?.note : null;

  return (
    <div className="min-h-0 flex-1 overflow-auto">
    <div className="mx-auto flex max-w-5xl flex-col gap-8 px-5 py-6">
      <h1 className="text-xl font-semibold tracking-tight">Models</h1>

      <section>
        <h2 className="text-sm font-semibold">Installed</h2>
        {error ? <p className="mt-2 text-sm text-danger">{error}</p> : null}
        <div className="mt-3 overflow-hidden rounded-xl border border-line">
          {models.length === 0 ? (
            <div className="flex flex-col items-center gap-2 bg-surface px-4 py-8">
              <AppIcon framed className="h-10 w-10" />
              <p className="text-sm text-muted">No local models yet</p>
            </div>
          ) : (
            models.map((model) => {
              const active = settings.activeModel === model.name;
              return (
                <div
                  key={model.name}
                  className={cn(
                    "flex items-center justify-between gap-3 border-b border-line bg-surface px-4 py-3 last:border-b-0",
                    active && "bg-accent-soft",
                  )}
                >
                  <div className="min-w-0">
                    <p className="truncate font-mono text-sm font-medium">{model.name}</p>
                    <p className="mt-0.5 font-mono text-[11px] text-muted">
                      {formatBytes(model.sizeBytes)}
                      {model.parameterSize ? ` · ${model.parameterSize}` : ""}
                      {model.cloud ? " · Cloud" : ""}
                    </p>
                  </div>
                  <div className="flex shrink-0 items-center gap-2">
                    {active ? (
                      <span className="rounded-md bg-accent px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-accent-ink">
                        Active
                      </span>
                    ) : (
                      <Button
                        size="sm"
                        variant="secondary"
                        disabled={!model.runnable}
                        onClick={() => {
                          void activateModel(model.name)
                            .then(onSettings)
                            .catch((caught: unknown) => {
                              setError(asFailure(caught).message);
                            });
                        }}
                      >
                        Activate
                      </Button>
                    )}
                    <Button
                      size="sm"
                      variant="ghost"
                      disabled={!model.runnable}
                      onClick={() => setPendingDelete(model.name)}
                    >
                      Delete
                    </Button>
                  </div>
                </div>
              );
            })
          )}
        </div>
      </section>

      <section>
        <h2 className="text-sm font-semibold">Presets</h2>
        <div className="mt-3 grid gap-3 sm:grid-cols-3">
          {presets.map((preset) => (
            <article
              key={preset.model}
              className={cn(
                "flex flex-col rounded-xl border bg-surface p-4 transition-colors duration-150",
                preset.suggested ? "border-accent ring-1 ring-accent" : "border-line hover:border-accent/60",
              )}
            >
              <div className="flex items-center justify-between gap-2">
                <span
                  className={cn(
                    "rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide",
                    categoryChip(preset.category),
                  )}
                >
                  {preset.category}
                </span>
                {preset.suggested ? <span className="text-[11px] font-medium text-accent">Suggested</span> : null}
              </div>
              <h3 className="mt-3 font-mono text-sm font-medium">{preset.model}</h3>
              <p className="mt-2 flex-1 text-sm leading-6 text-muted">{preset.summary}</p>
              <p className="mt-3 font-mono text-[11px] leading-5 text-muted">
                {preset.downloadSizeLabel} download
                <br />
                {formatBytes(preset.runtimeMemoryBytes)} runtime · {fitLabel(preset.fit)}
              </p>
              {sharedNote ? null : <p className="mt-2 text-xs leading-5 text-muted">{preset.note}</p>}
              <Button className="mt-4 w-full" size="sm" onClick={() => void install(preset.model)}>
                Download
              </Button>
            </article>
          ))}
        </div>
        {hardware ? (
          <div className="mt-3 space-y-1 text-xs leading-5 text-muted">
            {sharedNote ? <p>{sharedNote}</p> : null}
            {hardware.recommendations.notes
              .filter((note) => note !== sharedNote)
              .map((note) => (
                <p key={note}>{note}</p>
              ))}
            <p>
              {hardware.hardware.totalMemoryBytes > 0
                ? `${formatBytes(hardware.hardware.availableMemoryBytes)} free of ${formatBytes(hardware.hardware.totalMemoryBytes)}`
                : "Memory was not measured on this host."}
              {hardware.hardware.appleSilicon ? " · Apple Silicon" : ""}
              {hardware.hardware.gpuName ? ` · ${hardware.hardware.gpuName}` : ""}
            </p>
          </div>
        ) : null}
        {pull ? (
          <div className="mt-4">
            <div className="flex items-center justify-between font-mono text-xs">
              <span>
                {pull.model} · {pull.status}
              </span>
              <span>{pull.percent ?? 0}%</span>
            </div>
            <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-surface-2">
              <div className="h-full bg-accent transition-[width] duration-150" style={{ width: `${pull.percent ?? 0}%` }} />
            </div>
            {pull.error ? <p className="mt-2 text-sm text-danger">{pull.error}</p> : null}
            {!pull.done ? (
              <Button className="mt-3" size="sm" variant="secondary" onClick={() => void cancelPull()}>
                Cancel download
              </Button>
            ) : null}
          </div>
        ) : null}
      </section>

      <section className="rounded-xl border border-line bg-surface p-4">
        <h2 className="text-sm font-semibold">Try a model</h2>
        <div className="mt-3 flex flex-wrap items-center gap-2">
          <label className="sr-only" htmlFor="test-model">
            Model
          </label>
          <select
            id="test-model"
            value={testModel}
            onChange={(event) => setTestModel(event.target.value)}
            className="ps-field w-auto min-w-40 font-mono text-xs"
          >
            <option value="">Select a model</option>
            {runnable.map((model) => (
              <option key={model.name} value={model.name}>
                {model.name}
              </option>
            ))}
            {models.every((model) => model.name !== testModel) && testModel ? (
              <option value={testModel}>{testModel}</option>
            ) : null}
          </select>
          <Segmented label="Trial mode" value={mode} options={MODES} onChange={setMode} />
          <Button
            size="sm"
            disabled={busy || !testModel || !sample.trim()}
            onClick={() => {
              setBusy(true);
              setError(null);
              void optimizeText(sample, mode, testModel)
                .then(setOutcome)
                .catch((caught: unknown) => setError(asFailure(caught).message))
                .finally(() => setBusy(false));
            }}
          >
            {busy ? "Running…" : "Run"}
          </Button>
        </div>
        <textarea
          aria-label="Sample prompt"
          value={sample}
          spellCheck={false}
          onChange={(event) => setSample(event.target.value)}
          className="ps-editor mt-3 min-h-28 resize-y"
        />
        {outcome ? (
          <div className="mt-3">
            <p className="mb-2 font-mono text-[11px] text-muted">
              {outcome.status} · {formatDuration(outcome.durationMs)}
              {outcome.promptTokens != null ? ` · ${outcome.promptTokens} in` : ""}
              {outcome.completionTokens != null ? ` · ${outcome.completionTokens} out` : ""}
            </p>
            <ResultView
              status={outcome.status}
              original={outcome.originalText}
              rewritten={outcome.rewrittenPrompt}
              warnings={outcome.warnings}
              durationMs={outcome.durationMs}
              promptTokens={outcome.promptTokens}
              completionTokens={outcome.completionTokens}
              suspicious={outcome.suspicious}
            />
          </div>
        ) : null}
      </section>

      <ConfirmDialog
        open={pendingDelete !== null}
        title="Delete model?"
        body={pendingDelete ? `${pendingDelete} will be removed from local Ollama storage.` : ""}
        confirmLabel="Delete"
        onOpenChange={(open) => {
          if (!open) {
            setPendingDelete(null);
          }
        }}
        onConfirm={() => {
          if (!pendingDelete) {
            return;
          }
          void deleteModel(pendingDelete)
            .then((listed) => {
              setModels(listed);
              if (settings.activeModel === pendingDelete) {
                onSettings({ ...settings, activeModel: null });
              }
            })
            .catch((caught: unknown) => setError(asFailure(caught).message));
        }}
      />
    </div>
    </div>
  );
}

function categoryChip(category: string): string {
  if (category === "fast") {
    return "bg-[#d6f58a] text-[#1c2a12]";
  }
  if (category === "quality") {
    return "bg-[#6fbf9a] text-[#102117]";
  }
  return "bg-accent text-accent-ink";
}
