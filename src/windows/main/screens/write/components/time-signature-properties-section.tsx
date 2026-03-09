import { FC, useMemo } from "react";
import { actions } from "../../../../../services/actions";
import {
  NoteDuration,
  TimeSignature,
  TimeSignatureDrawType,
  UniqueId,
} from "../../../../../services/types";
import { Switch } from "../../../../../ui/switch";
import { CommonPropertiesSection } from "./common-properties-section";
import { PanelSection, PanelSectionColumn } from "./properties-panel";

type Props = {
  flowId: UniqueId;
  timeSignature: TimeSignature;
};

const isCommonTimeCandidate = (timeSignature: TimeSignature) => {
  return (
    timeSignature.beats === 4 &&
    timeSignature.beat_type === NoteDuration.Quarter
  );
};

const isCutTimeCandidate = (timeSignature: TimeSignature) => {
  return (
    timeSignature.beats === 2 && timeSignature.beat_type === NoteDuration.Half
  );
};

const getSwitchValue = (timeSignature: TimeSignature) => {
  if (isCommonTimeCandidate(timeSignature)) {
    return timeSignature.draw_type === TimeSignatureDrawType.CommonTime;
  }

  if (isCutTimeCandidate(timeSignature)) {
    return timeSignature.draw_type === TimeSignatureDrawType.SplitCommonTime;
  }

  return false;
};

const getNextDrawType = (timeSignature: TimeSignature, enabled: boolean) => {
  if (isCommonTimeCandidate(timeSignature)) {
    return enabled
      ? TimeSignatureDrawType.CommonTime
      : TimeSignatureDrawType.Regular;
  }

  if (isCutTimeCandidate(timeSignature)) {
    return enabled
      ? TimeSignatureDrawType.SplitCommonTime
      : TimeSignatureDrawType.Regular;
  }

  return null;
};

export const TimeSignaturePropertiesSection: FC<Props> = ({
  flowId,
  timeSignature,
}) => {
  const symbolFormEnabled = useMemo(() => {
    return (
      isCommonTimeCandidate(timeSignature) || isCutTimeCandidate(timeSignature)
    );
  }, [timeSignature]);

  const symbolFormValue = useMemo(() => {
    return getSwitchValue(timeSignature);
  }, [timeSignature]);

  return (
    <>
      <CommonPropertiesSection
        hidden={timeSignature.hidden}
        onHiddenChange={(hidden) => {
          void actions.score.timeSignature.update(
            flowId,
            timeSignature.id,
            timeSignature.draw_type,
            hidden,
          );
        }}
      />
      <PanelSection title="Time signature">
        <PanelSectionColumn>
          <Switch
            small
            label="Symbol form"
            description="Use C / cutC instead of stacked numerals when the signature allows it."
            value={symbolFormValue}
            disabled={!symbolFormEnabled}
            onChange={(enabled) => {
              const nextDrawType = getNextDrawType(timeSignature, enabled);

              if (nextDrawType === null) {
                return;
              }

              void actions.score.timeSignature.update(
                flowId,
                timeSignature.id,
                nextDrawType,
                timeSignature.hidden,
              );
            }}
          />
        </PanelSectionColumn>
      </PanelSection>
    </>
  );
};
