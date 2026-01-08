import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { GridLayout, Panel } from "../components/GridLayout/types";

interface GridLayoutState {
  layouts: Record<string, GridLayout>;
  currentLayoutId: string | null;
  panels: Record<string, Panel>;
  
  // Actions
  createLayout: (name: string) => GridLayout;
  setCurrentLayout: (layoutId: string) => void;
  saveLayout: (layout: GridLayout) => void;
  deleteLayout: (layoutId: string) => void;
  addPanel: (panel: Panel) => void;
  removePanel: (panelId: string) => void;
  updatePanel: (panelId: string, updates: Partial<Panel>) => void;
  getCurrentLayout: () => GridLayout | null;
  getPanel: (panelId: string) => Panel | undefined;
}

export const useGridLayoutStore = create<GridLayoutState>()(
  persist(
    (set, get) => ({
      layouts: {},
      currentLayoutId: null,
      panels: {},
      
      createLayout: (name) => {
        const layout: GridLayout = {
          id: `layout-${Date.now()}`,
          name,
          panels: [],
          columns: 2,
          rows: 2,
          createdAt: Date.now(),
          updatedAt: Date.now(),
        };
        
        set((state) => ({
          layouts: { ...state.layouts, [layout.id]: layout },
          currentLayoutId: layout.id,
        }));
        
        return layout;
      },
      
      setCurrentLayout: (layoutId) => {
        set({ currentLayoutId: layoutId });
      },
      
      saveLayout: (layout) => {
        const updated = {
          ...layout,
          updatedAt: Date.now(),
        };
        
        set((state) => ({
          layouts: { ...state.layouts, [layout.id]: updated },
        }));
      },
      
      deleteLayout: (layoutId) => {
        set((state) => {
          const { [layoutId]: _, ...rest } = state.layouts;
          return {
            layouts: rest,
            currentLayoutId: state.currentLayoutId === layoutId ? null : state.currentLayoutId,
          };
        });
      },
      
      addPanel: (panel) => {
        set((state) => ({
          panels: { ...state.panels, [panel.id]: panel },
        }));
        
        // Add to current layout if exists
        const currentLayout = get().getCurrentLayout();
        if (currentLayout) {
          const updated = {
            ...currentLayout,
            panels: [...currentLayout.panels, panel],
            updatedAt: Date.now(),
          };
          get().saveLayout(updated);
        }
      },
      
      removePanel: (panelId) => {
        set((state) => {
          const { [panelId]: _, ...rest } = state.panels;
          return { panels: rest };
        });
        
        // Remove from current layout
        const currentLayout = get().getCurrentLayout();
        if (currentLayout) {
          const updated = {
            ...currentLayout,
            panels: currentLayout.panels.filter((p) => p.id !== panelId),
            updatedAt: Date.now(),
          };
          get().saveLayout(updated);
        }
      },
      
      updatePanel: (panelId, updates) => {
        set((state) => {
          const panel = state.panels[panelId];
          if (!panel) return state;
          
          return {
            panels: {
              ...state.panels,
              [panelId]: { ...panel, ...updates },
            },
          };
        });
        
        // Update in current layout
        const currentLayout = get().getCurrentLayout();
        if (currentLayout) {
          const updated = {
            ...currentLayout,
            panels: currentLayout.panels.map((p) =>
              p.id === panelId ? { ...p, ...updates } : p
            ),
            updatedAt: Date.now(),
          };
          get().saveLayout(updated);
        }
      },
      
      getCurrentLayout: () => {
        const state = get();
        if (!state.currentLayoutId) return null;
        return state.layouts[state.currentLayoutId] || null;
      },
      
      getPanel: (panelId) => {
        return get().panels[panelId];
      },
    }),
    {
      name: "grid-layout-storage",
      storage: createJSONStorage(() => localStorage),
    }
  )
);

