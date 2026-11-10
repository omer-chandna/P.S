import { useEffect, useState } from "react";
import { AppIcon } from "@/components/brand/Mark";
import { appInfo, openExternal } from "@/lib/api";
import type { AppInfo } from "@/types";

const LINKS = [
  { label: "Ollama", href: "https://ollama.com/download" },
  { label: "Ollama API", href: "https://docs.ollama.com/" },
  { label: "Ollama source", href: "https://github.com/ollama/ollama" },
];

export function About() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  useEffect(() => {
    void appInfo().then(setInfo);
  }, []);

  return (
    <div className="mx-auto min-h-0 w-full max-w-xl flex-1 overflow-auto px-6 py-8">
      <AppIcon className="h-16 w-16" />
      <h1 className="mt-5 text-3xl font-semibold tracking-tight">P.S.</h1>
      <p className="mt-2 text-muted">Local prompt optimizer</p>
      <p className="mt-6 max-w-md text-base leading-7">
        Rewrites a selected prompt on this machine when that helps a coding agent, and leaves an effective prompt unchanged.
      </p>
      <dl className="mt-8 grid grid-cols-2 gap-x-6 gap-y-4 text-sm">
        <Fact label="Version" value={info?.version ?? "…"} />
        <Fact label="Prompt" value={info?.promptVersion ?? "1.3.0"} />
        <Fact label="System" value={info ? `${info.os} ${info.arch}` : "…"} />
        <Fact label="Target" value={info ? (info.supportedPlatform ? "Supported" : "Development") : "…"} />
      </dl>
      <p className="mt-6 text-sm leading-6 text-muted">
        Prompts stay on this computer. P.S. does not send them to an agent.
        {info && !info.supportedPlatform
          ? " Selection capture targets Apple Silicon macOS and Windows 11."
          : ""}
      </p>
      <div className="mt-6 flex flex-wrap gap-x-5 gap-y-2">
        {LINKS.map((link) => (
          <button
            key={link.href}
            type="button"
            className="cursor-pointer text-sm font-medium text-accent underline decoration-transparent underline-offset-4 transition-colors duration-150 hover:decoration-current"
            onClick={() => void openExternal(link.href)}
          >
            {link.label}
          </button>
        ))}
      </div>
      <p className="mt-8 text-xs text-muted">MIT license</p>
    </div>
  );
}

function Fact({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-xs text-muted">{label}</dt>
      <dd className="mt-0.5 font-medium">{value}</dd>
    </div>
  );
}
