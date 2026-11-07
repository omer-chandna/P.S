import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import type { ButtonHTMLAttributes } from "react";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex cursor-pointer items-center justify-center gap-2 rounded-lg border text-sm font-medium transition-[transform,box-shadow,background-color,border-color,color,opacity] duration-150 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent active:translate-y-px disabled:pointer-events-none disabled:opacity-40",
  {
    variants: {
      variant: {
        default:
          "border-transparent bg-accent text-accent-ink shadow-[var(--ps-key)] hover:bg-accent-hover active:shadow-[var(--ps-key-press)]",
        secondary:
          "border-transparent bg-surface text-ink shadow-[var(--ps-raise)] hover:text-ink active:shadow-[var(--ps-inset)]",
        ghost: "border-transparent bg-transparent text-muted hover:bg-surface-2 hover:text-ink",
        danger:
          "border-transparent bg-danger text-bg shadow-[var(--ps-raise)] hover:brightness-110 active:shadow-[var(--ps-key-press)]",
      },
      size: {
        default: "h-9 px-3.5",
        sm: "h-8 px-2.5 text-xs",
        lg: "h-10 px-4",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
);

export interface ButtonProps
  extends ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

export function Button({ className, variant, size, asChild = false, ...props }: ButtonProps) {
  const Comp = asChild ? Slot : "button";
  return <Comp className={cn(buttonVariants({ variant, size }), className)} {...props} />;
}
