import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { asFailure, openPermissionSettings, permissionInfo, updateSettings } from "@/lib/api";
import type { AppSettings, PermissionInfo, ShortcutInfo } from "@/types";
import { useEffect, useState, type ReactNode } from "react";

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

  return (
    <div className="mx-auto max-w-3xl space-y-6">
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
        <label className="mt-3 block text-sm">
          Theme
          <select
            aria-label="Theme"
            value={draft.theme}
            onChange={(event) => update("theme", event.target.value as AppSettings["theme"])}
            className="mt-1 h-9 w-full rounded-lg border border-line bg-bg px-2"
          >
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="system">System</option>
          </select>
        </label>
      </Section>

      <Section title="Shortcuts">
        <ShortcutField
          label="Improve"
          display={shortcuts?.improveDisplay}
          enabled={draft.improveShortcut.enabled}
          accelerator={draft.improveShortcut.accelerator}
          onEnabled={(enabled) =>
            update("improveShortcut", { ...draft.improveShortcut, enabled })
          }
          onAccelerator={(accelerator) =>
            update("improveShortcut", { ...draft.improveShortcut, accelerator })
          }
        />
        <ShortcutField
          label="Compact"
          display={shortcuts?.compactDisplay}
          enabled={draft.compactShortcut.enabled}
          accelerator={draft.compactShortcut.accelerator}
          onEnabled={(enabled) =>
            update("compactShortcut", { ...draft.compactShortcut, enabled })
          }
          onAccelerator={(accelerator) =>
            update("compactShortcut", { ...draft.compactShortcut, accelerator })
          }
        />
        <Toggle
          label="Pause shortcuts"
          checked={draft.shortcutsPaused}
          onChange={(checked) => update("shortcutsPaused", checked)}
        />
        {shortcuts?.conflict ? <p className="text-sm text-danger">{shortcuts.conflict}</p> : null}
        <p className="text-xs leading-5 text-muted">
          macOS default is Control+Option+P. Windows default is Ctrl+Alt+P. Compact adds Shift.
        </p>
      </Section>

      <Section title="Optimization">
        <label className="block text-sm">
          Default mode
          <select
            aria-label="Default mode"
            value={draft.defaultMode}
            onChange={(event) =>
              update("defaultMode", event.target.value as AppSettings["defaultMode"])
            }
            className="mt-1 h-9 w-full rounded-lg border border-line bg-bg px-2"
          >
            <option value="improve">Improve</option>
            <option value="compact">Compact</option>
          </select>
        </label>
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
        <p className="text-xs leading-5 text-muted">
          Unchanged prompts are never pasted. Suspicious rewrites open a review. Prompt version {draft.promptVersion}.
        </p>
      </Section>

      <Section title="Models">
        <label className="block text-sm">
          Active model
          <input
            aria-label="Active model"
            value={draft.activeModel ?? ""}
            onChange={(event) => update("activeModel", event.target.value || null)}
            className="mt-1 h-9 w-full rounded-lg border border-line bg-bg px-2"
            placeholder="qwen3.5:4b"
          />
        </label>
        <label className="mt-3 block text-sm">
          Local Ollama endpoint
          <input
            aria-label="Ollama endpoint"
            value={draft.ollamaEndpoint}
            onChange={(event) => update("ollamaEndpoint", event.target.value)}
            className="mt-1 h-9 w-full rounded-lg border border-line bg-bg px-2 font-mono text-xs"
          />
        </label>
        <p className="text-xs leading-5 text-muted">Only localhost, 127.0.0.1, and ::1 are accepted.</p>
      </Section>

      <Section title="Permissions">
        <p className="text-sm font-medium">{permission?.title}</p>
        <p className="text-sm leading-6 text-muted">{permission?.detail}</p>
        {permission?.state === "missing" ? (
          <Button variant="secondary" onClick={() => void openPermissionSettings().catch((caught: unknown) => setMessage(asFailure(caught).message))}>
            Enable Accessibility
          </Button>
        ) : null}
      </Section>

      <section className="rounded-2xl border border-line bg-surface p-4">
        <button type="button" className="text-sm font-medium" onClick={() => setAdvanced((open) => !open)}>
          {advanced ? "Hide advanced" : "Advanced"}
        </button>
        {advanced ? (
          <div className="mt-3 grid gap-3 sm:grid-cols-2">
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
                className="mt-1 h-9 w-full rounded-lg border border-line bg-bg px-2"
              />
            </label>
            <label className="text-sm">
              Keep-alive
              <input
                aria-label="Keep-alive"
                value={draft.keepAlive}
                onChange={(event) => update("keepAlive", event.target.value)}
                className="mt-1 h-9 w-full rounded-lg border border-line bg-bg px-2"
              />
            </label>
            <Toggle
              label="Enable model reasoning when supported"
              checked={draft.reasoningEnabled}
              onChange={(checked) => update("reasoningEnabled", checked)}
            />
          </div>
        ) : null}
      </section>

      <div className="flex items-center gap-3">
        <Button onClick={() => void save()}>Save settings</Button>
        <Button variant="secondary" onClick={onReplaySetup}>
          Replay setup
        </Button>
        {message ? <p className="text-sm text-muted">{message}</p> : null}
      </div>
    </div>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="rounded-2xl border border-line bg-surface p-4">
      <h2 className="text-sm font-medium">{title}</h2>
      <div className="mt-3 space-y-3">{children}</div>
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
    <div className="flex items-center justify-between gap-4 text-sm">
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
    <div className="grid gap-2 sm:grid-cols-[8rem_1fr_auto] sm:items-center">
      <span className="text-sm">{label}</span>
      <input
        aria-label={`${label} shortcut`}
        value={accelerator}
        onChange={(event) => onAccelerator(event.target.value)}
        className="h-9 rounded-lg border border-line bg-bg px-2 font-mono text-xs"
      />
      <div className="flex items-center gap-2">
        <Switch checked={enabled} onCheckedChange={onEnabled} label={`${label} shortcut enabled`} />
        <span className="text-xs text-muted">{display}</span>
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
        className="mt-1 h-9 w-full rounded-lg border border-line bg-bg px-2"
      />
    </label>
  );
}
