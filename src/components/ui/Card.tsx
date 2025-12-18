import { ReactNode } from "react";
import { cn } from "@/utils/cn";

interface CardProps {
  children: ReactNode;
  className?: string;
  title?: string;
  subtitle?: string;
}

export default function Card({ children, className, title, subtitle }: CardProps) {
  return (
    <div className={cn("glass-card p-6", className)}>
      {(title || subtitle) && (
        <div className="mb-4 pb-4 border-b border-white/10">
          {title && (
            <h3 className="text-lg font-semibold phosphor-glow-cyan mb-1">
              {title}
            </h3>
          )}
          {subtitle && (
            <p className="text-sm text-gray-400">{subtitle}</p>
          )}
        </div>
      )}
      {children}
    </div>
  );
}

