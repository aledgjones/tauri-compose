import { FC } from "react";
import { useAppState } from "../../../../../services/state";
import { useSelectedClef } from "../../../hooks/use-selected-clef";
import { useSelectedKeySignature } from "../../../hooks/use-selected-key-signature";
import { useSelectedTempo } from "../../../hooks/use-selected-tempo";
import { useSelectedTimeSignature } from "../../../hooks/use-selected-time-signature";
import { KeySignaturePropertiesSection } from "./key-signature-properties-section";
import { TimeSignaturePropertiesSection } from "./time-signature-properties-section";
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
  const selectedTimeSignature = useSelectedTimeSignature();
  const selectedKeySignature = useSelectedKeySignature();
  const selectedClef = useSelectedClef();

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

  if (selection.type === "TimeSignature" && selectedTimeSignature) {
    return (
      <div className="flex flex-1 min-h-0 bg-neutral-950 overflow-x-auto overflow-y-hidden">
        <div className="flex min-w-full gap-0.5">
          <TimeSignaturePropertiesSection
            flowId={selectedTimeSignature.flow_id}
            timeSignature={selectedTimeSignature.time_signature}
          />
          <div className="bg-neutral-900 flex-1" />
        </div>
      </div>
    );
  }

  if (selection.type === "KeySignature" && selectedKeySignature) {
    return (
      <div className="flex flex-1 min-h-0 bg-neutral-950 overflow-x-auto overflow-y-hidden">
        <div className="flex min-w-full gap-0.5">
          <KeySignaturePropertiesSection />
          <div className="bg-neutral-900 flex-1" />
        </div>
      </div>
    );
  }

  if (selection.type === "Clef" && selectedClef) {
    return (
      <EmptyState
        testId="properties-panel-unsupported"
        title="Clef properties out of scope"
        description="Clef selection and delete shortcuts are supported, but clef property editing is not part of this slice yet."
      />
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
