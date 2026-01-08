import { ButtonHTMLAttributes, ReactNode } from "react";
import { cn } from "@/utils/cn";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "ghost";
  children: ReactNode;
}

export default function Button({
  variant = "primary",
  className,
  children,
  disabled,
  ...props
}: ButtonProps) {
  return (
    <button
      className={cn(
        "glass-button",
        variant === "primary" && "glass-button-primary",
        variant === "ghost" && "bg-transparent border-transparent",
        disabled && "glass-button-disabled",
        className
      )}
      disabled={disabled}
      {...props}
    >
      {children}
    </button>
  );
}

