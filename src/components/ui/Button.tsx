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
  ...props
}: ButtonProps) {
  return (
    <button
      className={cn(
        "glass-button",
        variant === "primary" && "glass-button-primary",
        variant === "ghost" && "bg-transparent border-transparent",
        className
      )}
      {...props}
    >
      {children}
    </button>
  );
}

