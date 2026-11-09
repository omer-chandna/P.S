import { defaultSettings } from "@/types";
import { describe, expect, it } from "vitest";

describe("default settings", () => {
  it("exposes one optimize shortcut and no mode choice", () => {
    const settings = defaultSettings();
    expect(settings.version).toBe(2);
    expect(settings.shortcut).toEqual({ enabled: true, accelerator: "Ctrl+Alt+P" });
    expect(settings.promptVersion).toBe("1.3.0");
    expect(settings).not.toHaveProperty("improveShortcut");
    expect(settings).not.toHaveProperty("compactShortcut");
    expect(settings).not.toHaveProperty("defaultMode");
  });
});
