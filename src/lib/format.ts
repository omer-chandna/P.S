export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) {
    return "—";
  }
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  const digits = value >= 10 || Math.abs(value - Math.round(value)) < 0.05 ? 0 : 1;
  return `${value.toFixed(digits)} ${units[unit]}`;
}

export function approximateTokens(text: string): number {
  return Math.max(1, Math.ceil(text.length / 4));
}

export function formatDuration(durationMs: number): string {
  if (durationMs < 1000) {
    return `${durationMs} ms`;
  }
  return `${(durationMs / 1000).toFixed(1)} s`;
}

export function statusLabel(status: string): string {
  switch (status) {
    case "rewritten":
      return "Rewritten";
    case "unchanged":
      return "Already clear";
    case "needs_clarification":
      return "Needs a decision";
    default:
      return "Result";
  }
}

export function fitLabel(fit: string): string {
  switch (fit) {
    case "comfortable":
      return "Comfortable";
    case "tight":
      return "Tight";
    case "unlikely":
      return "Unlikely";
    default:
      return "Unknown";
  }
}
