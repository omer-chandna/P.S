import { useEffect, useState } from "react";
import { AppIcon, MenuMark } from "@/components/brand/Mark";
import { Button } from "@/components/ui/button";
import { ResultView } from "@/components/optimizer/ResultView";
import { Segmented } from "@/components/ui/segmented";
import { Switch } from "@/components/ui/switch";
import {
  asFailure,
  hardwareRecommendations,
  listModels,
  ollamaStatus,
  onPullProgress,
  openExternal,
  openPermissionSettings,
  optimizeText,
  permissionInfo,
  pullModel,
  shortcutInfo,
  updateSettings,
} from "@/lib/api";
import { isMacDesktop } from "@/lib/desktop";
import { cn } from "@/lib/utils";
import { SAMPLE_CLEAR, SAMPLE_VAGUE } from "@/lib/samples";
import type {
  AppSettings,
  ConnectionStatus,
  ListedModel,
  OptimizationMode,
  OptimizeOutcome,
  PermissionInfo,
  PresetRecommendation,
  PullProgress,
  ShortcutInfo,
} from "@/types";

const STEPS = ["Start", "Connect", "Model", "Shortcuts", "Try", "Ready"] as const;

const MODES: Array<{ value: OptimizationMode; label: string }> = [
  { value: "improve", label: "Improve" },
  { value: "compact", label: "Compact" },
];

export function Onboarding({
  settings,
  onSettings,
  onFinished,
}: {
  settings: AppSettings;
  onSettings: (settings: AppSettings) => void;
  onFinished: () => void;
}) {
  const [step, setStep] = useState(settings.onboardingStep);
  const [connection, setConnection] = useState<ConnectionStatus | null>(null);
  const [models, setModels] = useState<ListedModel[]>([]);
  const [presets, setPresets] = useState<PresetRecommendation[]>([]);
  const [permission, setPermission] = useState<PermissionInfo | null>(null);
  const [shortcuts, setShortcuts] = useState<ShortcutInfo | null>(null);
  const [pull, setPull] = useState<PullProgress | null>(null);
  const [sample, setSample] = useState(SAMPLE_VAGUE);
  const [mode, setMode] = useState<OptimizationMode>("improve");
  const [outcome, setOutcome] = useState<OptimizeOutcome | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void ollamaStatus().then(setConnection);
    void listModels().then(setModels).catch(() => setModels([]));
    void hardwareRecommendations().then((response) => setPresets(response.recommendations.presets));
    void permissionInfo().then(setPermission);
    void shortcutInfo().then(setShortcuts);
    const stopPromise = onPullProgress(setPull);
    return () => {
      void stopPromise.then((stop) => stop());
    };
  }, []);

  function go(next: number) {
    const bounded = Math.max(0, Math.min(STEPS.length - 1, next));
    setStep(bounded);
    const updated = { ...settings, onboardingStep: bounded };
    onSettings(updated);
    void updateSettings(updated).catch(() => undefined);
  }

  const runnable = models.filter((model) => model.runnable);

  return (
    <div className="ps-deck grid h-full min-h-0 grid-cols-1 overflow-hidden text-ink md:grid-cols-[232px_1fr]">
      <aside className="flex gap-3 overflow-auto border-b border-[#b6f5a8]/25 bg-[#0e1411] px-4 py-4 text-[#f4f6f1] shadow-[inset_-8px_0_16px_rgb(0_0_0/0.35)] md:flex-col md:gap-6 md:border-r md:border-b-0 md:px-4 md:py-6">
        <div className="flex shrink-0 items-center gap-2.5">
          <AppIcon framed className="h-10 w-10" />
          {isMacDesktop() ? null : <span className="text-lg font-semibold tracking-tight">P.S.</span>}
        </div>
        <ol className="flex gap-1 md:flex-col">
          {STEPS.map((label, index) => {
            const current = index === step;
            return (
              <li key={label}>
                <button
                  type="button"
                  onClick={() => go(index)}
                  className={cn(
                    "flex cursor-pointer items-center gap-2 rounded-lg px-2 py-1.5 text-left text-sm font-medium transition-colors duration-150",
                    current
                      ? "bg-[#121a16] text-[#f4f6f1] shadow-[inset_3px_3px_8px_#070b09,inset_-2px_-2px_6px_#24302a]"
                      : "text-[#a7b2aa] hover:bg-[#18211c] hover:text-[#f4f6f1]",
                  )}
                >
                  <span
                    className={cn(
                      "h-1.5 w-1.5 shrink-0 rounded-full",
                      index <= step ? "bg-[#b9e6a3]" : "bg-[#3c4741]",
                    )}
                  />
                  {label}
                </button>
              </li>
            );
          })}
        </ol>
      </aside>

      <div className="flex min-h-0 flex-col px-6 py-6 md:px-12 md:py-8">
        <p className="text-[11px] font-medium uppercase tracking-[0.16em] text-muted">
          {step + 1} / {STEPS.length}
        </p>
        <div className="mt-4 min-h-0 flex-1 overflow-auto">
          <div className="flex min-h-full items-center">
          <div className="w-full">
          {step === 0 ? (
            <div className="max-w-lg">
              <AppIcon framed className="mb-6 h-16 w-16" />
              <h1 className="text-4xl font-semibold tracking-tight">A clearer prompt, only when it helps.</h1>
              <p className="mt-4 text-base leading-7 text-muted">
                Select text and press the shortcut. A local model rewrites it, or leaves an effective prompt as you wrote it.
              </p>
            </div>
          ) : null}

          {step === 1 ? (
            <div className="max-w-lg space-y-4">
              <h1 className="text-3xl font-semibold tracking-tight">Ollama</h1>
              <p className="text-sm leading-6 text-muted">The model runs on this computer.</p>
              <p className="ps-kbd">{settings.ollamaEndpoint}</p>
              <p className={cn("text-sm font-medium", connection?.connected ? "text-ok" : "text-warn")}>
                {connection?.message ?? "Checking the local endpoint…"}
              </p>
              <div className="flex flex-wrap gap-2">
                <Button variant="secondary" onClick={() => void ollamaStatus().then(setConnection)}>
                  Retry connection
                </Button>
                <Button variant="ghost" onClick={() => void openExternal("https://ollama.com/download")}>
                  Install Ollama
                </Button>
              </div>
            </div>
          ) : null}

          {step === 2 ? (
            <div className="max-w-lg space-y-3">
              <h1 className="text-3xl font-semibold tracking-tight">Model</h1>
              {runnable.length > 0 ? (
                <div className="space-y-2">
                  {runnable.map((model) => {
                    const selected = settings.activeModel === model.name;
                    return (
                      <button
                        key={model.name}
                        type="button"
                        onClick={() => onSettings({ ...settings, activeModel: model.name })}
                        className={cn(
                          "flex w-full cursor-pointer items-center justify-between rounded-xl border bg-surface px-3 py-2.5 text-left text-sm transition-colors duration-150",
                          selected ? "border-accent ring-1 ring-accent" : "border-line hover:border-accent/60",
                        )}
                      >
                        <span className="font-mono">{model.name}</span>
                        {selected ? (
                          <span className="rounded-md bg-accent px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-accent-ink">
                            Selected
                          </span>
                        ) : null}
                      </button>
                    );
                  })}
                </div>
              ) : (
                <div className="grid gap-2">
                  {presets.map((preset) => (
                    <div key={preset.model} className="ps-raised rounded-xl p-3">
                      <div className="flex items-center justify-between gap-3">
                        <div>
                          <p className="inline-flex rounded-md bg-accent px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-accent-ink">
                        {preset.category}
                      </p>
                          <p className="mt-1 font-mono text-sm">{preset.model}</p>
                          <p className="mt-1 text-xs text-muted">{preset.downloadSizeLabel}</p>
                        </div>
                        <Button
                          size="sm"
                          onClick={() =>
                            void pullModel(preset.model).catch((caught: unknown) => setError(asFailure(caught).message))
                          }
                        >
                          Download
                        </Button>
                      </div>
                      <p className="mt-2 text-sm text-muted">{preset.summary}</p>
                    </div>
                  ))}
                </div>
              )}
              {pull ? (
                <p className="font-mono text-xs text-muted">
                  {pull.model} · {pull.status} {pull.percent ?? 0}% {pull.error ?? ""}
                </p>
              ) : null}
              {error ? <p className="text-sm text-danger">{error}</p> : null}
            </div>
          ) : null}

          {step === 3 ? (
            <div className="max-w-lg space-y-4">
              <h1 className="text-3xl font-semibold tracking-tight">Shortcuts</h1>
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="ps-raised rounded-xl p-4">
                  <p className="text-sm text-muted">Improve</p>
                  <p className="mt-2 font-mono text-lg">{shortcuts?.improveDisplay ?? "Ctrl+Alt+P"}</p>
                </div>
                <div className="ps-raised rounded-xl p-4">
                  <p className="text-sm text-muted">Compact</p>
                  <p className="mt-2 font-mono text-lg">{shortcuts?.compactDisplay ?? "Ctrl+Alt+Shift+P"}</p>
                </div>
              </div>
              <div>
                <p className="text-sm font-medium">{permission?.title}</p>
                <p className="mt-1 text-sm leading-6 text-muted">{permission?.detail}</p>
              </div>
              {permission?.state === "missing" ? (
                <Button
                  variant="secondary"
                  onClick={() =>
                    void openPermissionSettings().catch((caught: unknown) => setError(asFailure(caught).message))
                  }
                >
                  Enable Accessibility
                </Button>
              ) : null}
              {error ? <p className="text-sm text-danger">{error}</p> : null}
            </div>
          ) : null}

          {step === 4 ? (
            <div className="max-w-2xl space-y-3">
              <h1 className="text-3xl font-semibold tracking-tight">Try a prompt</h1>
              <div className="flex flex-wrap gap-2">
                <Button size="sm" variant="secondary" onClick={() => setSample(SAMPLE_VAGUE)}>
                  Vague prompt
                </Button>
                <Button size="sm" variant="secondary" onClick={() => setSample(SAMPLE_CLEAR)}>
                  Already clear
                </Button>
                <Segmented label="Trial mode" value={mode} options={MODES} onChange={setMode} />
              </div>
              <textarea
                aria-label="Onboarding sample"
                value={sample}
                spellCheck={false}
                onChange={(event) => setSample(event.target.value)}
                className="ps-editor min-h-28 resize-y"
              />
              <Button
                size="sm"
                disabled={busy || !settings.activeModel}
                onClick={() => {
                  if (!settings.activeModel) {
                    return;
                  }
                  setBusy(true);
                  setError(null);
                  void optimizeText(sample, mode, settings.activeModel)
                    .then(setOutcome)
                    .catch((caught: unknown) => setError(asFailure(caught).message))
                    .finally(() => setBusy(false));
                }}
              >
                {busy ? "Running…" : "Run locally"}
              </Button>
              {!settings.activeModel ? <p className="text-sm text-muted">Choose a model first, or skip this check.</p> : null}
              {error ? <p className="text-sm text-danger">{error}</p> : null}
              {outcome ? (
                <ResultView
                  status={outcome.status}
                  original={outcome.originalText}
                  rewritten={outcome.rewrittenPrompt}
                  warnings={outcome.warnings}
                  durationMs={outcome.durationMs}
                  promptTokens={outcome.promptTokens}
                  completionTokens={outcome.completionTokens}
                />
              ) : null}
            </div>
          ) : null}

          {step === 5 ? (
            <div className="max-w-lg space-y-5">
              <h1 className="text-3xl font-semibold tracking-tight">Ready</h1>
              <dl className="grid gap-3 sm:grid-cols-2">
                <div className="ps-raised rounded-xl p-4">
                  <dt className="text-sm text-muted">Model</dt>
                  <dd className="mt-1 font-mono text-sm">{settings.activeModel ?? "Not selected"}</dd>
                </div>
                <div className="ps-raised rounded-xl p-4">
                  <dt className="text-sm text-muted">Improve</dt>
                  <dd className="mt-1 font-mono text-sm">{shortcuts?.improveDisplay ?? "Ctrl+Alt+P"}</dd>
                </div>
              </dl>
              <p className="text-sm leading-6 text-muted">Closing the window keeps P.S. running.</p>
              <div className="flex items-center justify-between gap-4 ps-raised rounded-xl px-4 py-3 text-sm">
                <span>Launch at startup</span>
                <Switch
                  checked={settings.launchAtStartup}
                  onCheckedChange={(checked) => onSettings({ ...settings, launchAtStartup: checked })}
                  label="Launch at startup"
                />
              </div>
            </div>
          ) : null}
          </div>
          </div>
        </div>

        <div className="mt-6 flex items-center justify-between">
          <Button variant="ghost" disabled={step === 0} onClick={() => go(step - 1)}>
            Back
          </Button>
          {step < STEPS.length - 1 ? (
            <Button onClick={() => go(step + 1)}>Continue</Button>
          ) : (
            <Button onClick={onFinished}>
              <MenuMark onAccent className="h-3.5 w-3.5" />
              Finish
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
