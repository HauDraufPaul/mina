import { ReactNode, createContext, useContext } from "react";
import { Panel } from "./types";
import PanelHeader from "./PanelHeader";
import { useGridLayoutStore } from "../../stores/gridLayoutStore";

interface PanelContextValue {
  panelId: string;
  panel: Panel;
  isMinimized: boolean;
  isMaximized: boolean;
  onMinimize: () => void;
  onMaximize: () => void;
  onClose: () => void;
  updateConfig: (config: Record<string, any>) => void;
}

const PanelContext = createContext<PanelContextValue | null>(null);

export function usePanelContext() {
  const context = useContext(PanelContext);
  if (!context) {
    throw new Error("usePanelContext must be used within a GridPanel");
  }
  return context;
}

interface GridPanelProps {
  panel: Panel;
  children: ReactNode;
  onClose?: () => void;
}

export default function GridPanel({ panel, children, onClose }: GridPanelProps) {
  const { updatePanel, removePanel } = useGridLayoutStore();

  const handleMinimize = () => {
    updatePanel(panel.id, { minimized: !panel.minimized });
  };

  const handleMaximize = () => {
    updatePanel(panel.id, { maximized: !panel.maximized });
  };

  const handleClose = () => {
    if (onClose) {
      onClose();
    } else {
      removePanel(panel.id);
    }
  };

  const updateConfig = (config: Record<string, any>) => {
    updatePanel(panel.id, {
      config: { ...panel.config, ...config },
    });
  };

  const contextValue: PanelContextValue = {
    panelId: panel.id,
    panel,
    isMinimized: panel.minimized,
    isMaximized: panel.maximized,
    onMinimize: handleMinimize,
    onMaximize: handleMaximize,
    onClose: handleClose,
    updateConfig,
  };

  if (panel.minimized) {
    return (
      <PanelContext.Provider value={contextValue}>
        <div className="glass-card border border-white/10 rounded-lg overflow-hidden">
          <PanelHeader
            panel={panel}
            onMinimize={handleMinimize}
            onMaximize={handleMaximize}
            onClose={handleClose}
          />
        </div>
      </PanelContext.Provider>
    );
  }

  return (
    <PanelContext.Provider value={contextValue}>
      <div className="glass-card border border-white/10 rounded-lg overflow-hidden h-full flex flex-col">
        <PanelHeader
          panel={panel}
          onMinimize={handleMinimize}
          onMaximize={handleMaximize}
          onClose={handleClose}
        />
        <div className="flex-1 overflow-auto p-4">{children}</div>
      </div>
    </PanelContext.Provider>
  );
}

