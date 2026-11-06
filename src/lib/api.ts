import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  defaultSettings,
  type AppInfo,
  type AppSettings,
  type Attention,
  type ConnectionStatus,
  type HardwareResponse,
  type ListedModel,
  type OptimizationMode,
  type OptimizeOutcome,
  type PermissionInfo,
  type PsFailure,
  type PullProgress,
  type ShortcutInfo,
} from "@/types";

let browserSettings = defaultSettings();

export function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function asFailure(error: unknown): PsFailure {
  if (typeof error === "string") {
    try {
      const parsed = JSON.parse(error) as Partial<PsFailure>;
      if (parsed.message) {
        return { category: parsed.category ?? "unknown", message: parsed.message };
      }
    } catch {
      return { category: "unknown", message: error };
    }
    return { category: "unknown", message: error };
  }
  if (error && typeof error === "object" && "message" in error) {
    const record = error as Partial<PsFailure>;
    return {
      category: record.category ?? "unknown",
      message: record.message ?? "Something went wrong.",
    };
  }
  return { category: "unknown", message: "Something went wrong." };
}

export async function getSettings(): Promise<AppSettings> {
  if (!inTauri()) {
    return structuredClone(browserSettings);
  }
  return invoke<AppSettings>("get_settings");
}

export async function updateSettings(settings: AppSettings): Promise<AppSettings> {
  if (!inTauri()) {
    browserSettings = structuredClone(settings);
    return getSettings();
  }
  return invoke<AppSettings>("update_settings", { settings });
}

export async function ollamaStatus(): Promise<ConnectionStatus> {
  if (!inTauri()) {
    return {
      connected: false,
      endpoint: browserSettings.ollamaEndpoint,
      message: "This browser view is outside the desktop shell, so Ollama is not contacted.",
      runnableModels: 0,
      cloudModels: 0,
    };
  }
  return invoke<ConnectionStatus>("ollama_status");
}

export async function listModels(): Promise<ListedModel[]> {
  if (!inTauri()) {
    return [];
  }
  return invoke<ListedModel[]>("list_models");
}

export async function activateModel(model: string): Promise<AppSettings> {
  if (!inTauri()) {
    browserSettings.activeModel = model;
    return getSettings();
  }
  return invoke<AppSettings>("activate_model", { model });
}

export async function deleteModel(model: string): Promise<ListedModel[]> {
  return invoke<ListedModel[]>("delete_model", { model });
}

export async function pullModel(model: string): Promise<void> {
  if (!inTauri()) {
    throw {
      category: "unsupported_platform",
      message: "Model downloads run inside the desktop app.",
    } satisfies PsFailure;
  }
  await invoke("pull_model", { model });
}

export async function cancelPull(): Promise<boolean> {
  if (!inTauri()) {
    return false;
  }
  return invoke<boolean>("cancel_pull");
}

export async function hardwareRecommendations(): Promise<HardwareResponse> {
  if (!inTauri()) {
    return {
      hardware: {
        os: "browser",
        arch: "unknown",
        totalMemoryBytes: 0,
        availableMemoryBytes: 0,
        gpuName: null,
        gpuMemoryBytes: null,
        appleSilicon: false,
        detectionNotes: [
          "Memory was not measured in this browser preview.",
          "Suggestions are the default categories, not a hardware match.",
        ],
      },
      recommendations: {
        hardwareLimited: true,
        suggestedModel: "qwen3.5:4b",
        notes: ["Balanced is the default category when hardware cannot be measured."],
        presets: [
          {
            category: "fast",
            model: "qwen3.5:2b",
            downloadSizeLabel: "about 2 GB",
            runtimeMemoryBytes: 3 * 1024 * 1024 * 1024,
            summary: "Lower memory use and quicker rewrites.",
            fit: "unknown",
            suggested: false,
            note: "Memory could not be measured.",
          },
          {
            category: "balanced",
            model: "qwen3.5:4b",
            downloadSizeLabel: "about 3 GB",
            runtimeMemoryBytes: 6 * 1024 * 1024 * 1024,
            summary: "Default suggestion.",
            fit: "unknown",
            suggested: true,
            note: "Memory could not be measured.",
          },
          {
            category: "quality",
            model: "qwen3.5:9b",
            downloadSizeLabel: "about 6 GB",
            runtimeMemoryBytes: 10 * 1024 * 1024 * 1024,
            summary: "Higher capacity for constraint-heavy prompts.",
            fit: "unknown",
            suggested: false,
            note: "Memory could not be measured.",
          },
        ],
      },
    };
  }
  return invoke<HardwareResponse>("hardware_recommendations");
}

export async function optimizeText(
  text: string,
  mode: OptimizationMode,
  model?: string,
): Promise<OptimizeOutcome> {
  if (!inTauri()) {
    throw {
      category: "unsupported_platform",
      message: "Optimization runs inside the desktop app against local Ollama.",
    } satisfies PsFailure;
  }
  return invoke<OptimizeOutcome>("optimize_text", { request: { text, mode, model: model ?? null } });
}

export async function cancelOptimization(): Promise<boolean> {
  if (!inTauri()) {
    return false;
  }
  return invoke<boolean>("cancel_optimization");
}

export async function getAttention(): Promise<Attention | null> {
  if (!inTauri()) {
    return null;
  }
  return invoke<Attention | null>("attention");
}

export async function dismissAttention(): Promise<void> {
  if (!inTauri()) {
    return;
  }
  await invoke("dismiss_attention");
}

export async function acceptAttention(id: string): Promise<void> {
  await invoke("accept_attention", { id });
}

export async function shortcutInfo(): Promise<ShortcutInfo> {
  if (!inTauri()) {
    const macos = navigator.platform.toLowerCase().includes("mac");
    return {
      improveAccelerator: browserSettings.improveShortcut.accelerator,
      improveDisplay: macos ? "Control+Option+P" : browserSettings.improveShortcut.accelerator,
      improveEnabled: browserSettings.improveShortcut.enabled,
      compactAccelerator: browserSettings.compactShortcut.accelerator,
      compactDisplay: macos
        ? "Control+Option+Shift+P"
        : browserSettings.compactShortcut.accelerator,
      compactEnabled: browserSettings.compactShortcut.enabled,
      paused: browserSettings.shortcutsPaused,
      conflict: null,
      macos,
    };
  }
  return invoke<ShortcutInfo>("shortcut_info");
}

export async function permissionInfo(): Promise<PermissionInfo> {
  if (!inTauri()) {
    return {
      state: "unsupported",
      title: "Desktop permissions are unavailable here",
      detail: "Open the installed P.S. app to register shortcuts and check Accessibility.",
    };
  }
  return invoke<PermissionInfo>("permission_info");
}

export async function openPermissionSettings(): Promise<void> {
  await invoke("open_permission_settings");
}

export async function openExternal(url: string): Promise<void> {
  if (!inTauri()) {
    window.open(url, "_blank", "noopener,noreferrer");
    return;
  }
  await invoke("open_external", { url });
}

export async function appInfo(): Promise<AppInfo> {
  if (!inTauri()) {
    return {
      name: "P.S.",
      version: "0.1.0",
      promptVersion: "1.2.0",
      os: "browser",
      arch: "preview",
      supportedPlatform: false,
    };
  }
  return invoke<AppInfo>("app_info");
}

export async function finishOnboarding(settings: AppSettings): Promise<AppSettings> {
  const saved = await updateSettings({ ...settings, onboardingCompleted: true, onboardingStep: 5 });
  if (inTauri()) {
    return invoke<AppSettings>("finish_onboarding");
  }
  return saved;
}

export async function copyText(text: string): Promise<void> {
  if (inTauri()) {
    try {
      await invoke("copy_text", { text });
      return;
    } catch {
      // Fall through to the webview clipboard.
    }
  }
  await navigator.clipboard.writeText(text);
}

export async function onAttention(handler: (attention: Attention) => void): Promise<UnlistenFn> {
  if (!inTauri()) {
    return () => undefined;
  }
  return listen<Attention>("ps://attention", (event) => handler(event.payload));
}

export async function onNavigate(handler: (page: string) => void): Promise<UnlistenFn> {
  if (!inTauri()) {
    return () => undefined;
  }
  return listen<string>("ps://navigate", (event) => handler(event.payload));
}

export async function onPullProgress(handler: (progress: PullProgress) => void): Promise<UnlistenFn> {
  if (!inTauri()) {
    return () => undefined;
  }
  return listen<PullProgress>("ps://pull-progress", (event) => handler(event.payload));
}
