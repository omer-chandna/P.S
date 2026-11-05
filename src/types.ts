export type OptimizationMode = "improve" | "compact";
export type OptimizationStatus = "rewritten" | "unchanged" | "needs_clarification";
export type DeliveryPlan =
  | "replace"
  | "leave_unchanged"
  | "ask_clarification"
  | "preview"
  | "abort";
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
  improveShortcut: ShortcutBinding;
  compactShortcut: ShortcutBinding;
  defaultMode: OptimizationMode;
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
  mode: OptimizationMode;
  model: string;
  promptVersion: string;
  status: OptimizationStatus;
  originalText: string;
  rewrittenPrompt: string;
  clarificationQuestion: string;
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
  improveAccelerator: string;
  improveDisplay: string;
  improveEnabled: boolean;
  compactAccelerator: string;
  compactDisplay: string;
  compactEnabled: boolean;
  paused: boolean;
  conflict: string | null;
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
    version: 1,
    theme: "dark",
    launchAtStartup: false,
    runInBackground: true,
    notifications: true,
    shortcutsPaused: false,
    improveShortcut: { enabled: true, accelerator: "Ctrl+Alt+P" },
    compactShortcut: { enabled: true, accelerator: "Ctrl+Alt+Shift+P" },
    defaultMode: "improve",
    autoReplace: true,
    previewBeforeReplace: false,
    promptVersion: "1.0.0",
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
