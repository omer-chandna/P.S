import { describe, expect, it } from "vitest";
import { approximateTokens, fitLabel, formatBytes, formatDuration, statusLabel } from "@/lib/format";

describe("format helpers", () => {
  it("formats storage sizes without treating zero as a measurement", () => {
    expect(formatBytes(0)).toBe("—");
    expect(formatBytes(1536)).toBe("1.5 KB");
    expect(formatBytes(6 * 1024 * 1024 * 1024)).toBe("6 GB");
  });

  it("labels optimizer states without implying every prompt must change", () => {
    expect(statusLabel("unchanged")).toBe("Already clear");
    expect(statusLabel("rewritten")).toBe("Rewritten");
    expect(fitLabel("unknown")).toBe("Unknown");
  });

  it("estimates tokens and durations", () => {
    expect(approximateTokens("abcd")).toBe(1);
    expect(formatDuration(240)).toBe("240 ms");
    expect(formatDuration(1800)).toBe("1.8 s");
  });
});
