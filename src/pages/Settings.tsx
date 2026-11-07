import { Button } from "@/components/ui/button";
import { Segmented } from "@/components/ui/segmented";
import { Switch } from "@/components/ui/switch";
import { asFailure, openPermissionSettings, permissionInfo, updateSettings } from "@/lib/api";
import type { AppSettings, PermissionInfo, ShortcutInfo, ThemeName } from "@/types";
import { useEffect, useState, type ReactNode } from "react";

const THEMES: Array<{ value: ThemeName; label: string }> = [
  { value: "dark", label: "Dark" },
  { value: "light", label: "Light" },
  { value: "system", label: "System" },
];

const MODES: Array<{ value: AppSettings["defaultMode"]; label: string }> = [
  { value: "improve", label: "Improve" },
  { value: "compact", label: "Compact" },
];

export function Settings({
  settings,
  shortcuts,
  onSettings,
  onReplaySetup,
}: {
  settings: AppSettings;
  shortcuts: ShortcutInfo | null;
  onSettings: (settings: AppSettings) => void;
  onReplaySetup: () => void;
}) {
  const [draft, setDraft] = useState(settings);
  const [message, setMessage] = useState<string | null>(null);
  const [permission, setPermission] = useState<PermissionInfo | null>(null);
  const [advanced, setAdvanced] = useState(false);

  useEffect(() => {
    setDraft(settings);
  }, [settings]);

  useEffect(() => {
    void permissionInfo().then(setPermission);
  }, []);

  function update<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    setDraft((current) => ({ ...current, [key]: value }));
  }

  async function save() {
    setMessage(null);
    try {
      const saved = await updateSettings(draft);
      onSettings(saved);
      setMessage("Settings saved.");
    } catch (caught) {
      setMessage(asFailure(caught).message);
    }
  }

  const saved = message === "Settings saved.";

  return (
    <div className="flex min-h-0 flex-1 flex-col">
    <div className="min-h-0 flex-1 overflow-auto">
    <div className="mx-auto max-w-3xl px-5 py-6">
      <h1 className="text-xl font-semibold tracking-tight">Settings</h1>
      <div className="mt-5 space-y-4">
        <Section title="General">
          <Toggle
            label="Launch at startup"
            checked={draft.launchAtStartup}
            onChange={(checked) => update("launchAtStartup", checked)}
          />
          <Toggle
            label="Keep running in the background"
            checked={draft.runInBackground}
            onChange={(checked) => update("runInBackground", checked)}
          />
          <Toggle
            label="Notifications"
            checked={draft.notifications}
            onChange={(checked) => update("notifications", checked)}
          />
          <div className="flex items-center justify-between gap-4 py-3 text-sm">
            <span>Theme</span>
            <Segmented label="Theme" value={draft.theme} options={THEMES} onChange={(theme) => update("theme", theme)} />
          </div>
        </Section>

        <Section title="Shortcuts">
          <ShortcutField
            label="Improve"
            display={shortcuts?.improveDisplay}
            enabled={draft.improveShortcut.enabled}
            accelerator={draft.improveShortcut.accelerator}
            onEnabled={(enabled) => update("improveShortcut", { ...draft.improveShortcut, enabled })}
            onAccelerator={(accelerator) => update("improveShortcut", { ...draft.improveShortcut, accelerator })}
          />
          <ShortcutField
            label="Compact"
            display={shortcuts?.compactDisplay}
            enabled={draft.compactShortcut.enabled}
            accelerator={draft.compactShortcut.accelerator}
            onEnabled={(enabled) => update("compactShortcut", { ...draft.compactShortcut, enabled })}
            onAccelerator={(accelerator) => update("compactShortcut", { ...draft.compactShortcut, accelerator })}
          />
          <Toggle
            label="Pause shortcuts"
            checked={draft.shortcutsPaused}
            onChange={(checked) => update("shortcutsPaused", checked)}
          />
          {shortcuts?.conflict ? <p className="py-3 text-sm text-danger">{shortcuts.conflict}</p> : null}
        </Section>

        <Section title="Optimization">
          <div className="flex items-center justify-between gap-4 py-3 text-sm">
            <span>Default mode</span>
            <Segmented
              label="Default mode"
              value={draft.defaultMode}
              options={MODES}
              onChange={(mode) => update("defaultMode", mode)}
            />
          </div>
          <Toggle
            label="Replace the selection automatically"
            checked={draft.autoReplace}
            onChange={(checked) => update("autoReplace", checked)}
          />
          <Toggle
            label="Preview before replacing"
            checked={draft.previewBeforeReplace}
            onChange={(checked) => update("previewBeforeReplace", checked)}
          />
          <p className="py-3 text-xs leading-5 text-muted">
            Unchanged prompts stay in the editor. Suspicious rewrites open a review.
          </p>
        </Section>

        <Section title="Models">
          <label className="block py-3 text-sm">
            Active model
            <input
              aria-label="Active model"
              value={draft.activeModel ?? ""}
              onChange={(event) => update("activeModel", event.target.value || null)}
              className="ps-field mt-1.5 font-mono text-xs"
              placeholder="qwen3.5:4b"
            />
          </label>
          <label className="block py-3 text-sm">
            Ollama endpoint
            <input
              aria-label="Ollama endpoint"
              value={draft.ollamaEndpoint}
              onChange={(event) => update("ollamaEndpoint", event.target.value)}
              className="ps-field mt-1.5 font-mono text-xs"
            />
            <span className="mt-1.5 block text-xs text-muted">Loopback only: localhost, 127.0.0.1, or ::1.</span>
          </label>
        </Section>

        <Section title="Permissions">
          <div className="py-3">
            <p className="text-sm font-medium">{permission?.title}</p>
            <p className="mt-1 text-sm leading-6 text-muted">{permission?.detail}</p>
            {permission?.state === "missing" ? (
              <Button
                className="mt-3"
                variant="secondary"
                onClick={() => void openPermissionSettings().catch((caught: unknown) => setMessage(asFailure(caught).message))}
              >
                Enable Accessibility
              </Button>
            ) : null}
          </div>
        </Section>

        <section id="advanced-settings" className="scroll-mt-4 rounded-xl border border-line bg-surface px-4">
          <button
            type="button"
            className="flex w-full cursor-pointer items-center justify-between py-3 text-left text-sm font-semibold"
            onClick={() => {
              setAdvanced((open) => {
                const next = !open;
                if (next) {
                  requestAnimationFrame(() => {
                    document.getElementById("advanced-settings")?.scrollIntoView({ block: "nearest" });
                  });
                }
                return next;
              });
            }}
            aria-expanded={advanced}
          >
            Advanced
            <span className="text-xs font-medium text-muted">{advanced ? "Hide" : "Show"}</span>
          </button>
          {advanced ? (
            <div className="border-t border-line py-3">
              <div className="grid gap-3 sm:grid-cols-2">
                <NumberField
                  label="Temperature"
                  value={draft.temperature}
                  step={0.1}
                  onChange={(value) => update("temperature", value)}
                />
                <NumberField label="Top-p" value={draft.topP} step={0.05} onChange={(value) => update("topP", value)} />
                <NumberField
                  label="Timeout seconds"
                  value={draft.requestTimeoutSecs}
                  step={1}
                  onChange={(value) => update("requestTimeoutSecs", value)}
                />
                <label className="text-sm">
                  Output token limit
                  <input
                    aria-label="Output token limit"
                    value={draft.outputTokenLimit ?? ""}
                    placeholder="Automatic"
                    onChange={(event) =>
                      update("outputTokenLimit", event.target.value ? Number(event.target.value) : null)
                    }
                    className="ps-field mt-1.5"
                  />
                </label>
                <label className="text-sm">
                  Keep-alive
                  <input
                    aria-label="Keep-alive"
                    value={draft.keepAlive}
                    onChange={(event) => update("keepAlive", event.target.value)}
                    className="ps-field mt-1.5"
                  />
                </label>
              </div>
              <div className="mt-1">
                <Toggle
                  label="Enable model reasoning when supported"
                  checked={draft.reasoningEnabled}
                  onChange={(checked) => update("reasoningEnabled", checked)}
                />
              </div>
            </div>
          ) : null}
        </section>
      </div>
    </div>
    </div>

      <div className="flex shrink-0 flex-wrap items-center gap-3 border-t border-line bg-bg px-5 py-3">
        <Button onClick={() => void save()}>Save settings</Button>
        <Button variant="secondary" onClick={onReplaySetup}>
          Replay setup
        </Button>
        {message ? <p className={saved ? "text-sm text-ok" : "text-sm text-danger"}>{message}</p> : null}
      </div>
    </div>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="rounded-xl border border-line bg-surface px-4">
      <h2 className="border-b border-line py-3 text-sm font-semibold">{title}</h2>
      <div className="divide-y divide-line">{children}</div>
    </section>
  );
}

function Toggle({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between gap-4 py-3 text-sm">
      <span>{label}</span>
      <Switch checked={checked} onCheckedChange={onChange} label={label} />
    </div>
  );
}

function ShortcutField({
  label,
  display,
  enabled,
  accelerator,
  onEnabled,
  onAccelerator,
}: {
  label: string;
  display?: string;
  enabled: boolean;
  accelerator: string;
  onEnabled: (enabled: boolean) => void;
  onAccelerator: (accelerator: string) => void;
}) {
  return (
    <div className="grid gap-2 py-3 sm:grid-cols-[7rem_1fr_auto] sm:items-center">
      <span className="text-sm">{label}</span>
      <input
        aria-label={`${label} shortcut`}
        value={accelerator}
        onChange={(event) => onAccelerator(event.target.value)}
        className="ps-field font-mono text-xs"
      />
      <div className="flex items-center gap-2">
        <Switch checked={enabled} onCheckedChange={onEnabled} label={`${label} shortcut enabled`} />
        {display ? <span className="ps-kbd">{display}</span> : null}
      </div>
    </div>
  );
}

function NumberField({
  label,
  value,
  step,
  onChange,
}: {
  label: string;
  value: number;
  step: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="text-sm">
      {label}
      <input
        aria-label={label}
        type="number"
        step={step}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
        className="ps-field mt-1.5"
      />
    </label>
  );
}
