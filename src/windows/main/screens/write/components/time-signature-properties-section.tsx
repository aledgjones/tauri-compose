import { FC, useEffect, useMemo, useState } from "react";
import { actions } from "../../../../../services/actions";
import {
  NoteDuration,
  TimeSignature,
  TimeSignatureDrawType,
  UniqueId,
} from "../../../../../services/types";
import { Switch } from "../../../../../ui/switch";

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
  const [draftTimeSignature, setDraftTimeSignature] = useState(timeSignature);

  useEffect(() => {
    setDraftTimeSignature(timeSignature);
  }, [timeSignature]);

  const symbolFormEnabled = useMemo(() => {
    return (
      isCommonTimeCandidate(draftTimeSignature) ||
      isCutTimeCandidate(draftTimeSignature)
    );
  }, [draftTimeSignature]);

  const symbolFormValue = useMemo(() => {
    return getSwitchValue(draftTimeSignature);
  }, [draftTimeSignature]);

  return (
    <div
      className="flex min-w-max gap-0.5"
      data-testid="time-signature-properties-section"
    >
      <section className="flex min-w-[18rem] flex-col bg-neutral-900">
        <h3 className="text-xs text-white p-3 bg-neutral-800">
          Time signature
        </h3>
        <div className="p-3 flex">
          <div className="flex w-75 flex-col gap-3">
            <Switch
              small
              label="Symbol form"
              description="Use C / cutC instead of stacked numerals when the signature allows it."
              value={symbolFormValue}
              disabled={!symbolFormEnabled}
              onChange={(enabled) => {
                const nextDrawType = getNextDrawType(
                  draftTimeSignature,
                  enabled,
                );

                if (nextDrawType === null) {
                  return;
                }

                void actions.score.timeSignature
                  .update(flowId, timeSignature.id, nextDrawType)
                  .then(() => {
                    setDraftTimeSignature((current) => ({
                      ...current,
                      draw_type: nextDrawType,
                    }));
                  })
                  .catch(() => undefined);
              }}
            />
          </div>
        </div>
      </section>
    </div>
  );
};
