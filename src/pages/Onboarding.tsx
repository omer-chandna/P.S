import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { ResultView } from "@/components/optimizer/ResultView";
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
  updateSettings,
} from "@/lib/api";
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
} from "@/types";

const STEPS = ["Welcome", "Connect", "Model", "Shortcuts", "Try", "Finish"] as const;

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

  return (
    <div className="mx-auto flex h-full max-w-2xl flex-col py-6">
      <p className="text-xs uppercase tracking-[0.16em] text-muted">
        Step {step + 1} of {STEPS.length}
      </p>
      <h1 className="mt-2 text-2xl font-semibold tracking-tight">{STEPS[step]}</h1>
      <div className="mt-6 flex-1">
        {step === 0 ? (
          <div className="space-y-4 text-sm leading-7">
            <p className="text-lg leading-8">Welcome to P.S.</p>
            <p>
              Improve your prompts before sending them to your AI coding agent. Powered by local AI, without subscriptions or cloud inference.
            </p>
            <p>Select text, press the shortcut, and P.S. rewrites the prompt only when that helps. An already effective prompt may stay unchanged.</p>
          </div>
        ) : null}
        {step === 1 ? (
          <div className="space-y-3 text-sm leading-6">
            <p>
              Ollama runs the model on this computer. P.S. looks for it at {settings.ollamaEndpoint}.
            </p>
            <p className={connection?.connected ? "text-ok" : "text-warn"}>
              {connection?.message ?? "Checking the local endpoint…"}
            </p>
            <div className="flex gap-2">
              <Button
                variant="secondary"
                onClick={() => {
                  void ollamaStatus().then(setConnection);
                }}
              >
                Retry connection
              </Button>
              <Button variant="ghost" onClick={() => void openExternal("https://ollama.com/download")}>
                Install Ollama
              </Button>
            </div>
            <p className="text-muted">Start Ollama, then retry. P.S. will not install it for you.</p>
          </div>
        ) : null}
        {step === 2 ? (
          <div className="space-y-3">
            {models.filter((model) => model.runnable).length > 0 ? (
              <div className="space-y-2">
                {models
                  .filter((model) => model.runnable)
                  .map((model) => (
                    <button
                      key={model.name}
                      type="button"
                      onClick={() => onSettings({ ...settings, activeModel: model.name })}
                      className="block w-full rounded-xl border border-line bg-surface px-3 py-2 text-left text-sm"
                    >
                      {model.name}
                      {settings.activeModel === model.name ? " · selected" : ""}
                    </button>
                  ))}
              </div>
            ) : (
              <div className="grid gap-2">
                {presets.map((preset) => (
                  <div key={preset.model} className="rounded-xl border border-line bg-surface p-3 text-sm">
                    <p className="font-medium">
                      {preset.category} · {preset.model}
                    </p>
                    <p className="text-muted">{preset.summary}</p>
                    <p className="text-xs text-muted">Download {preset.downloadSizeLabel}</p>
                    <Button className="mt-2" size="sm" onClick={() => void pullModel(preset.model).catch((caught: unknown) => setError(asFailure(caught).message))}>
                      Download
                    </Button>
                  </div>
                ))}
              </div>
            )}
            {pull ? (
              <p className="text-sm text-muted">
                {pull.model}: {pull.status} {pull.percent ?? 0}% {pull.error ?? ""}
              </p>
            ) : null}
            {error ? <p className="text-sm text-danger">{error}</p> : null}
          </div>
        ) : null}
        {step === 3 ? (
          <div className="space-y-3 text-sm leading-6">
            <p>Improve: Control+Option+P on macOS, Ctrl+Alt+P on Windows. Compact adds Shift.</p>
            <p className="font-medium">{permission?.title}</p>
            <p className="text-muted">{permission?.detail}</p>
            <Button variant="secondary" onClick={() => void openPermissionSettings().catch((caught: unknown) => setError(asFailure(caught).message))}>
              Open Accessibility settings
            </Button>
            {error ? <p className="text-danger">{error}</p> : null}
          </div>
        ) : null}
        {step === 4 ? (
          <div className="space-y-3">
            <div className="flex gap-2">
              <Button size="sm" variant="secondary" onClick={() => setSample(SAMPLE_VAGUE)}>
                Vague prompt
              </Button>
              <Button size="sm" variant="secondary" onClick={() => setSample(SAMPLE_CLEAR)}>
                Already clear
              </Button>
            </div>
            <textarea
              aria-label="Onboarding sample"
              value={sample}
              onChange={(event) => setSample(event.target.value)}
              className="min-h-28 w-full rounded-xl border border-line bg-surface px-3 py-2"
            />
            <div className="flex gap-2">
              <Button size="sm" variant={mode === "improve" ? "default" : "secondary"} onClick={() => setMode("improve")}>
                Improve
              </Button>
              <Button size="sm" variant={mode === "compact" ? "default" : "secondary"} onClick={() => setMode("compact")}>
                Compact
              </Button>
              <Button
                size="sm"
                disabled={busy || !settings.activeModel}
                onClick={() => {
                  if (!settings.activeModel) {
                    return;
                  }
                  setBusy(true);
                  void optimizeText(sample, mode, settings.activeModel)
                    .then(setOutcome)
                    .catch((caught: unknown) => setError(asFailure(caught).message))
                    .finally(() => setBusy(false));
                }}
              >
                {busy ? "Running…" : "Run locally"}
              </Button>
            </div>
            {!settings.activeModel ? (
              <p className="text-sm text-muted">Choose or download a model first. You can also skip this check.</p>
            ) : null}
            {error ? <p className="text-sm text-danger">{error}</p> : null}
            {outcome ? (
              <ResultView
                status={outcome.status}
                original={outcome.originalText}
                rewritten={outcome.rewrittenPrompt}
                question={outcome.clarificationQuestion}
                warnings={outcome.warnings}
                durationMs={outcome.durationMs}
                promptTokens={outcome.promptTokens}
                completionTokens={outcome.completionTokens}
              />
            ) : null}
          </div>
        ) : null}
        {step === 5 ? (
          <div className="space-y-3 text-sm leading-6">
            <p>Active model: {settings.activeModel ?? "Not selected yet"}</p>
            <p>Improve shortcut: Ctrl+Alt+P, shown as Control+Option+P on macOS.</p>
            <p>P.S. keeps running after this window closes. Turn on launch at startup if you want it ready after a restart.</p>
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={settings.launchAtStartup}
                onChange={(event) => onSettings({ ...settings, launchAtStartup: event.target.checked })}
              />
              Launch at startup
            </label>
          </div>
        ) : null}
      </div>
      <div className="mt-6 flex justify-between">
        <Button variant="ghost" disabled={step === 0} onClick={() => go(step - 1)}>
          Back
        </Button>
        {step < STEPS.length - 1 ? (
          <Button onClick={() => go(step + 1)}>Continue</Button>
        ) : (
          <Button onClick={onFinished}>Finish</Button>
        )}
      </div>
    </div>
  );
}
