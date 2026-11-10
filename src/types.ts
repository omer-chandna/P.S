export type OptimizationStatus = "rewritten" | "unchanged";
export type DeliveryPlan = "replace" | "leave_unchanged" | "preview" | "abort";
export type ThemeName = "dark" | "light" | "system";
export type MemoryFit = "comfortable" | "tight" | "unlikely" | "unknown";
export type PresetCategory = "fast" | "balanced" | "quality";

export interface ShortcutBinding {
  enabled: boolean;
  accelerator: string;
}

export interface AppSettings {
  version: number;
  theme: ThemeName;
  launchAtStartup: boolean;
  runInBackground: boolean;
  notifications: boolean;
  shortcutsPaused: boolean;
  shortcut: ShortcutBinding;
  autoReplace: boolean;
  previewBeforeReplace: boolean;
  promptVersion: string;
  ollamaEndpoint: string;
  activeModel: string | null;
  temperature: number;
  topP: number;
  outputTokenLimit: number | null;
  requestTimeoutSecs: number;
  reasoningEnabled: boolean;
  keepAlive: string;
  onboardingCompleted: boolean;
  onboardingStep: number;
}

export interface OptimizeOutcome {
  operationId: string;
  model: string;
  promptVersion: string;
  status: OptimizationStatus;
  originalText: string;
  rewrittenPrompt: string;
  delivery: DeliveryPlan;
  warnings: string[];
  durationMs: number;
  promptTokens: number | null;
  completionTokens: number | null;
  suspicious: boolean;
}

export interface Attention {
  outcome: OptimizeOutcome;
  pendingId: string | null;
  note: string;
}

export interface ListedModel {
  name: string;
  sizeBytes: number;
  digest: string | null;
  parameterSize: string | null;
  quantization: string | null;
  family: string | null;
  remoteHost: string | null;
  remoteModel: string | null;
  cloud: boolean;
  runnable: boolean;
}

export interface ConnectionStatus {
  connected: boolean;
  endpoint: string;
  message: string;
  runnableModels: number;
  cloudModels: number;
}

export interface PresetRecommendation {
  category: PresetCategory;
  model: string;
  downloadSizeLabel: string;
  runtimeMemoryBytes: number;
  summary: string;
  fit: MemoryFit;
  suggested: boolean;
  note: string;
}

export interface HardwareProfile {
  os: string;
  arch: string;
  totalMemoryBytes: number;
  availableMemoryBytes: number;
  gpuName: string | null;
  gpuMemoryBytes: number | null;
  appleSilicon: boolean;
  detectionNotes: string[];
}

export interface RecommendationSet {
  presets: PresetRecommendation[];
  suggestedModel: string | null;
  hardwareLimited: boolean;
  notes: string[];
}

export interface HardwareResponse {
  hardware: HardwareProfile;
  recommendations: RecommendationSet;
}

export interface ShortcutInfo {
  accelerator: string;
  display: string;
  enabled: boolean;
  paused: boolean;
  macos: boolean;
}

export interface PermissionInfo {
  state: "granted" | "missing" | "unsupported";
  title: string;
  detail: string;
}

export interface AppInfo {
  name: string;
  version: string;
  promptVersion: string;
  os: string;
  arch: string;
  supportedPlatform: boolean;
}

export interface PullProgress {
  model: string;
  status: string;
  percent: number | null;
  done: boolean;
  error: string | null;
}

export interface PsFailure {
  category: string;
  message: string;
}

export function defaultSettings(): AppSettings {
  return {
    version: 2,
    theme: "dark",
    launchAtStartup: false,
    runInBackground: true,
    notifications: true,
    shortcutsPaused: false,
    shortcut: { enabled: true, accelerator: "Ctrl+Alt+P" },
    autoReplace: true,
    previewBeforeReplace: false,
    promptVersion: "1.3.0",
    ollamaEndpoint: "http://127.0.0.1:11434",
    activeModel: null,
    temperature: 0.2,
    topP: 0.9,
    outputTokenLimit: null,
    requestTimeoutSecs: 90,
    reasoningEnabled: false,
    keepAlive: "10m",
    onboardingCompleted: false,
    onboardingStep: 0,
  };
}
