import type { ReactNode } from "react";
import { cn } from "@/lib/utils";
import type { ConnectionStatus } from "@/types";

export type PageId = "home" | "models" | "settings" | "about";

const NAV: Array<{ id: PageId; label: string }> = [
  { id: "home", label: "Home" },
  { id: "models", label: "Models" },
  { id: "settings", label: "Settings" },
  { id: "about", label: "About" },
];

export function Shell({
  page,
  onNavigate,
  connection,
  modelName,
  children,
}: {
  page: PageId;
  onNavigate: (page: PageId) => void;
  connection: ConnectionStatus | null;
  modelName: string | null;
  children: ReactNode;
}) {
  return (
    <div className="flex h-full min-h-0 bg-bg text-ink">
      <nav className="flex w-40 shrink-0 flex-col border-r border-line bg-surface px-3 py-4">
        <div className="px-2">
          <p className="text-lg font-semibold tracking-tight">P.S.</p>
          <p className="text-[11px] uppercase tracking-[0.16em] text-muted">Local prompts</p>
        </div>
        <div className="mt-6 flex flex-col gap-1">
          {NAV.map((item) => (
            <button
              key={item.id}
              type="button"
              onClick={() => onNavigate(item.id)}
              className={cn(
                "rounded-lg px-2 py-2 text-left text-sm",
                page === item.id ? "bg-surface-2 text-ink" : "text-muted hover:bg-surface-2 hover:text-ink",
              )}
            >
              {item.label}
            </button>
          ))}
        </div>
        <div className="mt-auto px-2 text-xs leading-5 text-muted">
          <p>{connection?.connected ? "Ollama connected" : "Ollama offline"}</p>
          <p className="truncate">{modelName ?? "No model selected"}</p>
        </div>
      </nav>
      <div className="flex min-w-0 flex-1 flex-col">
        <header className="flex items-center justify-between border-b border-line px-6 py-4">
          <div>
            <h1 className="text-lg font-semibold tracking-tight">P.S.</h1>
            <p className="text-sm text-muted">Local Prompt Optimizer</p>
          </div>
          <div className="flex items-center gap-2 text-xs">
            <span
              className={cn(
                "rounded-full border px-2 py-1",
                connection?.connected ? "border-ok/40 text-ok" : "border-line text-muted",
              )}
            >
              {connection?.connected ? connection.endpoint : "Not connected"}
            </span>
            <button type="button" className="rounded-full border border-line px-2 py-1" onClick={() => onNavigate("settings")}>
              Settings
            </button>
          </div>
        </header>
        <main className="min-h-0 flex-1 overflow-auto px-6 py-5">{children}</main>
      </div>
    </div>
  );
}
