import { FC, PropsWithChildren } from "react";
import { useAppState } from "../../../../../services/state";
import { useSelectedClef } from "../../../hooks/use-selected-clef";
import { useSelectedKeySignature } from "../../../hooks/use-selected-key-signature";
import { useSelectedTempo } from "../../../hooks/use-selected-tempo";
import { useSelectedTimeSignature } from "../../../hooks/use-selected-time-signature";
import { ClefPropertiesSection } from "./clef-properties-section";
import { KeySignaturePropertiesSection } from "./key-signature-properties-section";
import { TimeSignaturePropertiesSection } from "./time-signature-properties-section";
import { TempoPropertiesSection } from "./tempo-properties-section";

const Panel: FC<PropsWithChildren<{ testId?: string }>> = ({
  children,
  testId,
}) => {
  return (
    <div
      className="flex flex-1 min-h-0 bg-neutral-950 overflow-x-auto overflow-y-hidden"
      data-testid={testId}
    >
      <div className="flex min-w-full gap-0.5">
        {children}
        <div className="bg-neutral-900 flex-1" />
      </div>
    </div>
  );
};

type PanelSectionProps = PropsWithChildren<{
  title: string;
}>;
export const PanelSection: FC<PanelSectionProps> = ({ title, children }) => {
  return (
    <section className="flex min-w-[18rem] flex-col bg-neutral-900">
      <h3 className="text-xs text-white p-3 bg-neutral-800">{title}</h3>
      <div className="p-3 gap-5 flex">{children}</div>
    </section>
  );
};

export const PanelSectionColumn: FC<PropsWithChildren> = ({ children }) => {
  return <div className="flex flex-col gap-3 w-75">{children}</div>;
};

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
      <Panel>
        <TempoPropertiesSection
          flowId={selectedTempo.flow_id}
          tempo={selectedTempo.tempo}
        />
      </Panel>
    );
  }

  if (selection.type === "TimeSignature" && selectedTimeSignature) {
    return (
      <Panel data-testid="time-signature-properties-section">
        <TimeSignaturePropertiesSection
          flowId={selectedTimeSignature.flow_id}
          timeSignature={selectedTimeSignature.time_signature}
        />
      </Panel>
    );
  }

  if (selection.type === "KeySignature" && selectedKeySignature) {
    return (
      <Panel data-testid="key-signature-properties-section">
        <KeySignaturePropertiesSection
          flowId={selectedKeySignature.flow_id}
          keySignature={selectedKeySignature.key_signature}
        />
      </Panel>
    );
  }

  if (selection.type === "Clef" && selectedClef) {
    return (
      <Panel data-testid="clef-properties-section">
        <ClefPropertiesSection
          flowId={selectedClef.flow_id}
          staveId={selectedClef.stave_id}
          clef={selectedClef.clef}
        />
      </Panel>
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
