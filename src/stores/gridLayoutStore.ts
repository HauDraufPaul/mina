import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { GridLayout, Panel } from "../components/GridLayout/types";
import { invoke } from "@tauri-apps/api/core";

interface GridLayoutState {
  layouts: Record<string, GridLayout>;
  currentLayoutId: string | null;
  panels: Record<string, Panel>;
  
  // Actions
  createLayout: (name: string) => GridLayout;
  setCurrentLayout: (layoutId: string) => void;
  saveLayout: (layout: GridLayout) => Promise<void>;
  deleteLayout: (layoutId: string) => Promise<void>;
  addPanel: (panel: Panel) => void;
  removePanel: (panelId: string) => void;
  updatePanel: (panelId: string, updates: Partial<Panel>) => void;
  getCurrentLayout: () => GridLayout | null;
  getPanel: (panelId: string) => Panel | undefined;
  loadLayouts: () => Promise<void>;
  exportLayout: (layoutId: string) => Promise<string>;
  importLayout: (layoutJson: string) => Promise<GridLayout>;
  createFromTemplate: (templateId: string, name: string) => Promise<GridLayout>;
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
        
        // Save to backend
        (async () => {
          try {
            const layoutJson = JSON.stringify(layout);
            await invoke("create_grid_layout", {
              id: layout.id,
              name: layout.name,
              layoutJson,
              columns: layout.columns,
              rows: layout.rows,
              isTemplate: false,
            });
          } catch (err) {
            console.error("Failed to create layout in backend:", err);
          }
        })();
        
        set((state) => ({
          layouts: { ...state.layouts, [layout.id]: layout },
          currentLayoutId: layout.id,
        }));
        
        return layout;
      },
      
      setCurrentLayout: (layoutId) => {
        set({ currentLayoutId: layoutId });
      },
      
      saveLayout: async (layout) => {
        const updated = {
          ...layout,
          updatedAt: Date.now(),
        };
        
        // Save to backend
        try {
          const layoutJson = JSON.stringify(updated);
          await invoke("update_grid_layout", {
            id: updated.id,
            name: updated.name,
            layoutJson,
            columns: updated.columns,
            rows: updated.rows,
          });
        } catch (err) {
          console.error("Failed to save layout to backend:", err);
          // Still update local state
        }
        
        set((state) => ({
          layouts: { ...state.layouts, [layout.id]: updated },
        }));
      },
      
      deleteLayout: async (layoutId) => {
        // Delete from backend
        try {
          await invoke("delete_grid_layout", { id: layoutId });
        } catch (err) {
          console.error("Failed to delete layout from backend:", err);
        }
        
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
      
      loadLayouts: async () => {
        try {
          const layouts = await invoke<any[]>("list_grid_layouts", { includeTemplates: false });
          const layoutsMap: Record<string, GridLayout> = {};
          
          for (const layoutData of layouts) {
            try {
              const layout: GridLayout = JSON.parse(layoutData.layout_json);
              layoutsMap[layout.id] = layout;
            } catch (err) {
              console.error(`Failed to parse layout ${layoutData.id}:`, err);
            }
          }
          
          set((state) => ({
            layouts: { ...state.layouts, ...layoutsMap },
          }));
        } catch (err) {
          console.error("Failed to load layouts from backend:", err);
        }
      },
      
      exportLayout: async (layoutId) => {
        const layout = get().layouts[layoutId];
        if (!layout) {
          throw new Error(`Layout ${layoutId} not found`);
        }
        return JSON.stringify(layout, null, 2);
      },
      
      importLayout: async (layoutJson) => {
        const layout: GridLayout = JSON.parse(layoutJson);
        const newId = `layout-${Date.now()}`;
        const importedLayout = {
          ...layout,
          id: newId,
          updatedAt: Date.now(),
        };
        
        // Save to backend
        try {
          await invoke("create_grid_layout", {
            id: importedLayout.id,
            name: importedLayout.name,
            layoutJson: JSON.stringify(importedLayout),
            columns: importedLayout.columns,
            rows: importedLayout.rows,
            isTemplate: false,
          });
        } catch (err) {
          console.error("Failed to import layout to backend:", err);
        }
        
        set((state) => ({
          layouts: { ...state.layouts, [importedLayout.id]: importedLayout },
        }));
        
        return importedLayout;
      },
      
      createFromTemplate: async (templateId, name) => {
        try {
          const template = await invoke<any>("get_grid_layout", { id: templateId });
          if (!template) {
            throw new Error(`Template ${templateId} not found`);
          }
          
          const templateLayout: GridLayout = JSON.parse(template.layout_json);
          const newLayout: GridLayout = {
            ...templateLayout,
            id: `layout-${Date.now()}`,
            name,
            createdAt: Date.now(),
            updatedAt: Date.now(),
          };
          
          // Save to backend
          await invoke("create_grid_layout", {
            id: newLayout.id,
            name: newLayout.name,
            layoutJson: JSON.stringify(newLayout),
            columns: newLayout.columns,
            rows: newLayout.rows,
            isTemplate: false,
          });
          
          set((state) => ({
            layouts: { ...state.layouts, [newLayout.id]: newLayout },
            currentLayoutId: newLayout.id,
          }));
          
          return newLayout;
        } catch (err) {
          console.error("Failed to create layout from template:", err);
          throw err;
        }
      },
    }),
    {
      name: "grid-layout-storage",
      storage: createJSONStorage(() => localStorage),
    }
  )
);

