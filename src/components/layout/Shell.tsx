import type { ReactNode } from "react";
import { Cpu, House, Info, Settings } from "lucide-react";
import { AppIcon } from "@/components/brand/Mark";
import { inTauri } from "@/lib/api";
import { isMacDesktop } from "@/lib/desktop";
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
    <div className="flex h-full min-h-0 overflow-hidden text-ink">
      <nav className="flex w-[212px] shrink-0 flex-col border-r border-[#b6f5a8]/25 bg-[#0e1411] px-3 py-4 text-[#f4f6f1] shadow-[inset_-8px_0_16px_rgb(0_0_0/0.35)]">
        <button
          type="button"
          aria-label="P.S."
          onClick={() => onNavigate("home")}
          className="flex cursor-pointer items-center gap-2.5 rounded-lg px-1.5 py-1 text-left"
        >
          <AppIcon framed className="h-9 w-9" />
          {isMacDesktop() ? null : <span className="text-base font-semibold tracking-tight">P.S.</span>}
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
                    ? "bg-[#121a16] text-[#f4f6f1] shadow-[inset_3px_3px_8px_#070b09,inset_-2px_-2px_6px_#24302a,inset_2px_0_0_#b6f5a8]"
                    : "text-[#a7b2aa] hover:bg-[#18211c] hover:text-[#f4f6f1]",
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
            className="w-full cursor-pointer rounded-lg px-2 py-2 text-left shadow-[inset_3px_3px_8px_#070b09,inset_-2px_-2px_6px_#24302a] transition-colors duration-150"
          >
            <span className="flex items-center gap-2 text-xs font-medium">
              <span
                className={cn(
                  "h-1.5 w-1.5 rounded-full",
                  connected ? "bg-[#b6f5a8] shadow-[0_0_8px_#b6f5a8]" : "bg-[#e4c27a] shadow-[0_0_8px_#e4c27a]",
                )}
              />
              {connected ? "Connected" : "Offline"}
            </span>
            <span className="mt-1 block truncate pl-3.5 text-[11px] text-[#a7b2aa]">
              {modelName ?? "No model"}
            </span>
          </button>
        </div>
      </nav>
      <div className="ps-deck flex min-h-0 min-w-0 flex-1 flex-col">
        {banner ? (
          <p className="shrink-0 border-b border-danger/30 bg-danger/10 px-5 py-2 text-sm text-danger">{banner}</p>
        ) : null}
        <main className="flex min-h-0 flex-1 flex-col overflow-hidden">{children}</main>
      </div>
    </div>
  );
}
