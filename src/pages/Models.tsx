import { useEffect, useState } from "react";
import { ConfirmDialog } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ResultView } from "@/components/optimizer/ResultView";
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

  return (
    <div className="space-y-6">
      <section>
        <h2 className="text-base font-medium">Installed models</h2>
        <p className="mt-1 text-sm text-muted">Cloud models stay visible and cannot be selected.</p>
        {error ? <p className="mt-2 text-sm text-danger">{error}</p> : null}
        <div className="mt-3 divide-y divide-line overflow-hidden rounded-2xl border border-line">
          {models.length === 0 ? (
            <p className="px-4 py-6 text-sm text-muted">No installed models reported yet.</p>
          ) : (
            models.map((model) => (
              <div key={model.name} className="flex items-center justify-between gap-3 bg-surface px-4 py-3">
                <div>
                  <p className="font-medium">
                    {model.name}{" "}
                    {settings.activeModel === model.name ? (
                      <span className="text-xs text-ok">Active</span>
                    ) : null}
                  </p>
                  <p className="text-xs text-muted">
                    {formatBytes(model.sizeBytes)}
                    {model.parameterSize ? ` · ${model.parameterSize}` : ""}
                    {model.cloud ? " · Cloud only" : ""}
                  </p>
                </div>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="secondary"
                    disabled={!model.runnable || settings.activeModel === model.name}
                    onClick={() => {
                      void activateModel(model.name).then(onSettings).catch((caught: unknown) => {
                        setError(asFailure(caught).message);
                      });
                    }}
                  >
                    Activate
                  </Button>
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
            ))
          )}
        </div>
      </section>

      <section>
        <h2 className="text-base font-medium">Recommended presets</h2>
        <p className="mt-1 max-w-3xl text-sm leading-6 text-muted">
          These are starting candidates, not a measured ranking. Download size is not the memory a model uses while running.
        </p>
        <div className="mt-3 grid gap-3 lg:grid-cols-3">
          {hardware?.recommendations.presets.map((preset) => (
            <article
              key={preset.model}
              className={cn(
                "rounded-2xl border bg-surface p-4",
                preset.suggested ? "border-accent" : "border-line",
              )}
            >
              <p className="text-xs uppercase tracking-[0.14em] text-muted">{preset.category}</p>
              <h3 className="mt-1 font-medium">{preset.model}</h3>
              <p className="mt-2 text-sm leading-6 text-muted">{preset.summary}</p>
              <p className="mt-3 text-xs text-muted">
                Download {preset.downloadSizeLabel} · runtime budget {formatBytes(preset.runtimeMemoryBytes)} · {fitLabel(preset.fit)}
              </p>
              <p className="mt-2 text-xs leading-5">{preset.note}</p>
              <Button className="mt-4" size="sm" onClick={() => void install(preset.model)}>
                Download
              </Button>
            </article>
          ))}
        </div>
        {hardware ? (
          <ul className="mt-3 space-y-1 text-xs leading-5 text-muted">
            {hardware.recommendations.notes.map((note) => (
              <li key={note}>{note}</li>
            ))}
            <li>
              {hardware.hardware.totalMemoryBytes > 0
                ? `Memory: ${formatBytes(hardware.hardware.availableMemoryBytes)} available of ${formatBytes(hardware.hardware.totalMemoryBytes)}`
                : "System memory was not measured on this host."}
              {hardware.hardware.appleSilicon ? " · Apple Silicon unified memory" : ""}
              {hardware.hardware.gpuName ? ` · ${hardware.hardware.gpuName}` : ""}
            </li>
          </ul>
        ) : null}
        {pull ? (
          <div className="mt-4 rounded-2xl border border-line bg-surface p-4">
            <div className="flex items-center justify-between text-sm">
              <span>
                {pull.model}: {pull.status}
              </span>
              <span>{pull.percent ?? 0}%</span>
            </div>
            <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-surface-2">
              <div className="h-full bg-accent" style={{ width: `${pull.percent ?? 0}%` }} />
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

      <section className="rounded-2xl border border-line bg-surface p-4">
        <h2 className="text-base font-medium">Test model</h2>
        <p className="mt-1 text-sm text-muted">
          Compare one result at a time. P.S. does not score or rank models.
        </p>
        <div className="mt-3 flex flex-wrap items-center gap-2">
          <label className="text-sm text-muted" htmlFor="test-model">
            Model
          </label>
          <select
            id="test-model"
            value={testModel}
            onChange={(event) => setTestModel(event.target.value)}
            className="h-9 rounded-lg border border-line bg-bg px-2"
          >
            <option value="">Select a model</option>
            {models
              .filter((model) => model.runnable)
              .map((model) => (
                <option key={model.name} value={model.name}>
                  {model.name}
                </option>
              ))}
            {models.every((model) => model.name !== testModel) && testModel ? (
              <option value={testModel}>{testModel}</option>
            ) : null}
          </select>
          <div className="flex rounded-lg border border-line p-0.5">
            {(["improve", "compact"] as const).map((item) => (
              <button
                key={item}
                type="button"
                className={cn("rounded-md px-3 py-1 text-sm capitalize", mode === item && "bg-surface-2")}
                onClick={() => setMode(item)}
              >
                {item}
              </button>
            ))}
          </div>
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
          onChange={(event) => setSample(event.target.value)}
          className="mt-3 min-h-28 w-full rounded-xl border border-line bg-bg px-3 py-2 leading-6 outline-none focus:border-accent"
        />
        {outcome ? (
          <div className="mt-3">
            <p className="mb-2 text-xs text-muted">
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
  );
}
