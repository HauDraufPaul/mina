import { ReactNode, ButtonHTMLAttributes } from "react";
import { cn } from "@/utils/cn";

interface TabItem {
  id: string;
  label: ReactNode;
  icon?: ReactNode;
}

interface TabsProps {
  items: TabItem[];
  activeTab: string;
  onTabChange: (tabId: string) => void;
  className?: string;
}

export default function Tabs({ items, activeTab, onTabChange, className }: TabsProps) {
  return (
    <div className={cn("flex gap-2", className)} role="tablist">
      {items.map((item) => {
        const isActive = activeTab === item.id;
        return (
          <button
            key={item.id}
            role="tab"
            aria-selected={isActive}
            aria-controls={`tabpanel-${item.id}`}
            id={`tab-${item.id}`}
            onClick={() => onTabChange(item.id)}
            className={cn(
              "glass-tab",
              isActive && "glass-tab-active",
              !isActive && "glass-tab-inactive"
            )}
          >
            {item.icon && <span className="mr-2">{item.icon}</span>}
            {item.label}
          </button>
        );
      })}
    </div>
  );
}

