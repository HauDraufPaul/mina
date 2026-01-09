import { ComponentType } from "react";
import SystemMonitorHub from "../modules/SystemMonitorHub/SystemMonitorHub";
import NetworkConstellation from "../modules/NetworkConstellation/NetworkConstellation";
import { getWidgetComponent, listAvailableWidgets } from "../widgets/WidgetRegistry";
import type { WidgetProps } from "../widgets/WidgetRegistry";

// Registry of components that can be rendered in panels
export const panelComponentRegistry: Record<string, ComponentType<any>> = {
  "system-monitor": SystemMonitorHub,
  "network": NetworkConstellation,
  // Widgets are handled separately via getPanelComponent
};

export function getPanelComponent(componentId: string): ComponentType<any> | null {
  // Check if it's a widget
  if (listAvailableWidgets().includes(componentId)) {
    return getWidgetComponent(componentId) as ComponentType<any>;
  }
  
  // Check regular panel components
  return panelComponentRegistry[componentId] || null;
}

export function registerPanelComponent(
  componentId: string,
  component: ComponentType<any>
) {
  panelComponentRegistry[componentId] = component;
}

export function listAvailablePanels(): string[] {
  return Object.keys(panelComponentRegistry);
}

