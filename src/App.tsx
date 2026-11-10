import { useEffect, useState } from "react";
import { Shell, type PageId } from "@/components/layout/Shell";
import {
  asFailure,
  finishOnboarding,
  getAttention,
  getSettings,
  ollamaStatus,
  onAttention,
  onNavigate,
  shortcutInfo,
  updateSettings,
} from "@/lib/api";
import { AppIcon } from "@/components/brand/Mark";
import { About } from "@/pages/About";
import { Home } from "@/pages/Home";
import { Models } from "@/pages/Models";
import { Onboarding } from "@/pages/Onboarding";
import { Review } from "@/pages/Review";
import { Settings } from "@/pages/Settings";
import { defaultSettings, type AppSettings, type Attention, type ConnectionStatus, type ShortcutInfo, type ThemeName } from "@/types";

function applyTheme(theme: ThemeName) {
  const systemDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  const dark = theme === "dark" || (theme === "system" && systemDark);
  document.documentElement.classList.toggle("dark", dark);
}

export function App() {
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [ready, setReady] = useState(false);
  const [page, setPage] = useState<PageId>("home");
  const [setup, setSetup] = useState(false);
  const [connection, setConnection] = useState<ConnectionStatus | null>(null);
  const [shortcuts, setShortcuts] = useState<ShortcutInfo | null>(null);
  const [attention, setAttention] = useState<Attention | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    void (async () => {
      try {
        const [loaded, status, shortcutState, pending] = await Promise.all([
          getSettings(),
          ollamaStatus(),
          shortcutInfo(),
          getAttention(),
        ]);
        setSettings(loaded);
        setConnection(status);
        setShortcuts(shortcutState);
        setAttention(pending);
        setSetup(!loaded.onboardingCompleted);
        if (pending) {
          setPage("home");
        }
      } catch (caught) {
        setLoadError(asFailure(caught).message);
      } finally {
        setReady(true);
      }
    })();
  }, []);

  useEffect(() => {
    applyTheme(settings.theme);
  }, [settings.theme]);

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => applyTheme(settings.theme);
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, [settings.theme]);

  useEffect(() => {
    const stops: Array<() => void> = [];
    void onAttention((next) => {
      setAttention(next);
      setSetup(false);
    }).then((stop) => stops.push(stop));
    void onNavigate((next) => {
      if (next === "models" || next === "settings" || next === "home" || next === "about") {
        setPage(next);
        setSetup(false);
        setAttention(null);
      }
    }).then((stop) => stops.push(stop));
    return () => {
      for (const stop of stops) {
        stop();
      }
    };
  }, []);

  if (!ready) {
    return (
      <div className="grid h-full place-items-center bg-[#141816]">
        <AppIcon className="h-14 w-14" />
      </div>
    );
  }

  if (setup) {
    return (
      <Onboarding
        settings={settings}
        onSettings={setSettings}
        onFinished={() => {
          void finishOnboarding(settings).then((saved) => {
            setSettings(saved);
            setSetup(false);
          });
        }}
      />
    );
  }

  return (
    <Shell
      page={page}
      onNavigate={(next) => {
        setAttention(null);
        setPage(next);
      }}
      connection={connection}
      modelName={settings.activeModel}
      banner={loadError}
    >
      {attention ? (
        <Review attention={attention} onClose={() => setAttention(null)} />
      ) : page === "home" ? (
        <Home modelName={settings.activeModel} shortcuts={shortcuts} />
      ) : page === "models" ? (
        <Models
          settings={settings}
          onSettings={(next) => {
            setSettings(next);
            void updateSettings(next).catch((caught: unknown) => setLoadError(asFailure(caught).message));
          }}
        />
      ) : page === "settings" ? (
        <Settings
          settings={settings}
          shortcuts={shortcuts}
          onSettings={(next) => {
            setSettings(next);
            void shortcutInfo().then(setShortcuts);
            void ollamaStatus().then(setConnection);
          }}
          onReplaySetup={() => {
            setSettings((current) => ({ ...current, onboardingStep: 0 }));
            setSetup(true);
          }}
        />
      ) : (
        <About />
      )}
    </Shell>
  );
}
