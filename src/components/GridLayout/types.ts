export interface Panel {
  id: string;
  type: PanelType;
  component: string; // Module/component identifier
  position: { row: number; col: number };
  size: { width: number; height: number };
  config: Record<string, any>; // Panel-specific config
  minimized: boolean;
  maximized: boolean;
  title?: string;
}

export type PanelType = "module" | "widget" | "chart" | "table" | "terminal" | "custom";

export interface GridLayout {
  id: string;
  name: string;
  panels: Panel[];
  columns: number;
  rows: number;
  createdAt: number;
  updatedAt: number;
}

export interface PanelConfig {
  [key: string]: any;
}

