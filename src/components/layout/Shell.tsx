import type { ReactNode } from "react";
import { Cpu, House, Info, Settings } from "lucide-react";
import { AppIcon } from "@/components/brand/Mark";
import { inTauri } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { ConnectionStatus } from "@/types";

export type PageId = "home" | "models" | "settings" | "about";

const NAV: Array<{ id: PageId; label: string; icon: typeof House }> = [
  { id: "home", label: "Home", icon: House },
  { id: "models", label: "Models", icon: Cpu },
  { id: "settings", label: "Settings", icon: Settings },
  { id: "about", label: "About", icon: Info },
];

export function Shell({
  page,
  onNavigate,
  connection,
  modelName,
  banner,
  children,
}: {
  page: PageId;
  onNavigate: (page: PageId) => void;
  connection: ConnectionStatus | null;
  modelName: string | null;
  banner?: string | null;
  children: ReactNode;
}) {
  const connected = connection?.connected === true;

  return (
    <div className="flex h-full min-h-0 overflow-hidden bg-bg text-ink">
      <nav className="flex w-[212px] shrink-0 flex-col border-r border-[#2a3330] bg-[#141816] px-3 py-4 text-[#f4f6f1]">
        <button
          type="button"
          onClick={() => onNavigate("home")}
          className="flex cursor-pointer items-center gap-2.5 rounded-lg px-1.5 py-1 text-left"
        >
          <AppIcon framed className="h-9 w-9" />
          <span className="text-base font-semibold tracking-tight">P.S.</span>
        </button>
        <div className="mt-7 flex flex-col gap-1">
          {NAV.map((item) => {
            const active = page === item.id;
            const Icon = item.icon;
            return (
              <button
                key={item.id}
                type="button"
                aria-current={active ? "page" : undefined}
                onClick={() => onNavigate(item.id)}
                className={cn(
                  "flex cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-2 text-left text-sm font-medium transition-colors duration-150",
                  active
                    ? "bg-[#24352a] text-[#f4f6f1] shadow-[inset_2px_0_0_#b9e6a3]"
                    : "text-[#a7b2aa] hover:bg-[#1c2420] hover:text-[#f4f6f1]",
                )}
              >
                <Icon className={cn("h-4 w-4", active ? "text-[#b9e6a3]" : "text-[#7f8c84]")} strokeWidth={1.75} />
                {item.label}
              </button>
            );
          })}
        </div>
        <div className="mt-auto">
          {!inTauri() ? (
            <p className="px-2 pb-1 text-[10px] font-medium uppercase tracking-[0.16em] text-[#7f8c84]">Preview</p>
          ) : null}
          <button
            type="button"
            onClick={() => onNavigate("settings")}
            className="w-full cursor-pointer rounded-lg px-2 py-2 text-left transition-colors duration-150 hover:bg-[#1c2420]"
          >
            <span className="flex items-center gap-2 text-xs font-medium">
              <span className={cn("h-1.5 w-1.5 rounded-full", connected ? "bg-[#b9e6a3]" : "bg-[#e4c27a]")} />
              {connected ? "Connected" : "Offline"}
            </span>
            <span className="mt-1 block truncate pl-3.5 text-[11px] text-[#a7b2aa]">
              {modelName ?? "No model"}
            </span>
          </button>
        </div>
      </nav>
      <div className="flex min-h-0 min-w-0 flex-1 flex-col">
        {banner ? (
          <p className="shrink-0 border-b border-danger/30 bg-danger/10 px-5 py-2 text-sm text-danger">{banner}</p>
        ) : null}
        <main className="flex min-h-0 flex-1 flex-col overflow-hidden">{children}</main>
      </div>
    </div>
  );
}
