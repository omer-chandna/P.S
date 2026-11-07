import appIcon from "../../../imgs/ps-app-icon.svg";
import menuIcon from "../../../imgs/ps-menubar-icon.svg";
import { cn } from "@/lib/utils";

export function AppIcon({ className, framed = false }: { className?: string; framed?: boolean }) {
  return (
    <img
      src={appIcon}
      alt=""
      width={1024}
      height={1024}
      draggable={false}
      className={cn(
        "pointer-events-none select-none",
        framed && "[filter:drop-shadow(0_0_0.6px_#B9E6A3)_drop-shadow(0_0_0.6px_#B9E6A3)]",
        className,
      )}
    />
  );
}

export function MenuMark({
  onAccent = false,
  className,
}: {
  onAccent?: boolean;
  className?: string;
}) {
  return (
    <img
      src={menuIcon}
      alt=""
      width={44}
      height={44}
      draggable={false}
      className={cn(
        "pointer-events-none select-none",
        onAccent ? "invert dark:invert-0" : "dark:invert",
        className,
      )}
    />
  );
}
