import { FC } from "react";
import { useAppState } from "../../../../../services/state";
import { useSelectedTempo } from "../../../hooks/use-selected-tempo";
import { TempoPropertiesSection } from "./tempo-properties-section";

const EmptyState: FC<{
  title: string;
  description: string;
  testId: string;
}> = ({ title, description, testId }) => {
  return (
    <div
      className="flex flex-1 items-center justify-center bg-neutral-900 px-4 py-6 text-center"
      data-testid={testId}
    >
      <div className="max-w-md space-y-1">
        <p className="text-sm font-medium text-white">{title}</p>
        <p className="text-xs text-neutral-400">{description}</p>
      </div>
    </div>
  );
};

export const PropertiesPanel: FC = () => {
  const selection = useAppState().ui.sequencer.selection;
  const selectedTempo = useSelectedTempo();

  if (selection === null) {
    return (
      <EmptyState
        testId="properties-panel-empty"
        title="No selection"
        description="Select a score item to edit its properties."
      />
    );
  }

  if (selection.type === "Tempo" && selectedTempo) {
    return (
      <div className="flex flex-1 min-h-0 bg-neutral-950 overflow-x-auto overflow-y-hidden">
        <div className="flex min-w-full gap-0.5">
          <TempoPropertiesSection
            flowId={selectedTempo.flow_id}
            tempo={selectedTempo.tempo}
          />
          <div className="bg-neutral-900 flex-1" />
        </div>
      </div>
    );
  }

  return (
    <EmptyState
      testId="properties-panel-unsupported"
      title="Properties unavailable"
      description="This selection type does not have an editor yet."
    />
  );
};
