import { ComponentType } from "react";
import SystemMonitorHub from "../modules/SystemMonitorHub/SystemMonitorHub";
import NetworkConstellation from "../modules/NetworkConstellation/NetworkConstellation";

// Registry of components that can be rendered in panels
export const panelComponentRegistry: Record<string, ComponentType<any>> = {
  "system-monitor": SystemMonitorHub,
  "network": NetworkConstellation,
  // Add more components as needed
};

export function getPanelComponent(componentId: string): ComponentType<any> | null {
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

