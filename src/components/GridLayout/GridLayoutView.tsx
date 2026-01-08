import GridLayout from "./GridLayout";
import { getPanelComponent } from "./PanelRegistry";

export default function GridLayoutView() {
  return (
    <div className="h-full w-full">
      <GridLayout>
        {(panel) => {
          const Component = getPanelComponent(panel.component);
          return Component ? (
            <Component />
          ) : (
            <div className="p-4 text-gray-400">
              Component "{panel.component}" not found in registry
            </div>
          );
        }}
      </GridLayout>
    </div>
  );
}

