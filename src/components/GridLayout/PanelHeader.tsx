import { Minimize2, Maximize2, X, MoreVertical } from "lucide-react";
import { Panel } from "./types";

interface PanelHeaderProps {
  panel: Panel;
  onMinimize: () => void;
  onMaximize: () => void;
  onClose: () => void;
  onMenuClick?: () => void;
}

export default function PanelHeader({
  panel,
  onMinimize,
  onMaximize,
  onClose,
  onMenuClick,
}: PanelHeaderProps) {
  return (
    <div className="flex items-center justify-between px-3 py-2 bg-black/30 border-b border-white/10">
      <div className="flex items-center gap-2 flex-1 min-w-0">
        <h3 className="text-sm font-semibold text-gray-300 truncate">
          {panel.title || panel.component}
        </h3>
      </div>
      <div className="flex items-center gap-1">
        <button
          onClick={onMinimize}
          className="p-1 text-gray-400 hover:text-white transition-colors"
          title={panel.minimized ? "Restore" : "Minimize"}
        >
          <Minimize2 className="w-4 h-4" />
        </button>
        <button
          onClick={onMaximize}
          className="p-1 text-gray-400 hover:text-white transition-colors"
          title={panel.maximized ? "Restore" : "Maximize"}
        >
          <Maximize2 className="w-4 h-4" />
        </button>
        {onMenuClick && (
          <button
            onClick={onMenuClick}
            className="p-1 text-gray-400 hover:text-white transition-colors"
            title="Menu"
          >
            <MoreVertical className="w-4 h-4" />
          </button>
        )}
        <button
          onClick={onClose}
          className="p-1 text-gray-400 hover:text-neon-red transition-colors"
          title="Close"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}

