import { ReactNode, useEffect } from "react";
import { Panel } from "./types";
import GridPanel from "./GridPanel";
import { useGridLayoutStore } from "../../stores/gridLayoutStore";
import { Group, Panel as ResizablePanel, Separator } from "react-resizable-panels";

interface GridLayoutProps {
  layoutId?: string;
  defaultLayout?: { columns: number; rows: number };
  onLayoutChange?: (panels: Panel[]) => void;
  maxPanels?: number;
  children?: (panel: Panel) => ReactNode;
}

export default function GridLayout({
  layoutId,
  onLayoutChange,
  children,
}: GridLayoutProps) {
  const {
    currentLayoutId,
    panels,
    getCurrentLayout,
    setCurrentLayout,
    createLayout,
  } = useGridLayoutStore();

  useEffect(() => {
    if (layoutId) {
      setCurrentLayout(layoutId);
    } else if (!currentLayoutId) {
      // Create default layout
      const layout = createLayout("Default Layout");
      setCurrentLayout(layout.id);
    }
  }, [layoutId, currentLayoutId, setCurrentLayout, createLayout]);

  const currentLayout = getCurrentLayout();
  const layoutPanels = currentLayout?.panels || [];

  useEffect(() => {
    if (onLayoutChange) {
      onLayoutChange(layoutPanels);
    }
  }, [layoutPanels, onLayoutChange]);

  if (layoutPanels.length === 0) {
    return (
      <div className="flex items-center justify-center h-full min-h-[400px] text-gray-400">
        <div className="text-center">
          <p className="text-lg mb-2">No panels in this layout</p>
          <p className="text-sm">Use the command bar to add panels (coming soon)</p>
        </div>
      </div>
    );
  }

  // Simple 2-column layout for now
  // Can be enhanced with more complex grid logic
  const leftPanels = layoutPanels.filter((_, i) => i % 2 === 0);
  const rightPanels = layoutPanels.filter((_, i) => i % 2 === 1);

  return (
    <div className="h-full w-full">
      <Group orientation="horizontal" className="h-full">
        <ResizablePanel defaultSize={50} minSize={20}>
          <div className="flex-1 flex flex-col gap-2 p-2 h-full">
            {leftPanels.map((panel) => {
              const panelData = panels[panel.id] || panel;
              return (
                <GridPanel key={panel.id} panel={panelData}>
                  {children ? children(panelData) : <div>Panel: {panel.component}</div>}
                </GridPanel>
              );
            })}
          </div>
        </ResizablePanel>
        {rightPanels.length > 0 && (
          <>
            <Separator className="w-2 bg-white/5 hover:bg-neon-cyan/50 transition-colors" />
            <ResizablePanel defaultSize={50} minSize={20}>
              <div className="flex-1 flex flex-col gap-2 p-2 h-full">
                {rightPanels.map((panel) => {
                  const panelData = panels[panel.id] || panel;
                  return (
                    <GridPanel key={panel.id} panel={panelData}>
                      {children ? children(panelData) : <div>Panel: {panel.component}</div>}
                    </GridPanel>
                  );
                })}
              </div>
            </ResizablePanel>
          </>
        )}
      </Group>
    </div>
  );
}

