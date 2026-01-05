import { FC, useMemo } from "react";
import { Instrument } from "../../../../../services/types";
import { useSamplerState } from "../../../../../services/state";
import { IconButton } from "../../../../../ui/icon-button";
import { mdiSizeM, mdiSizeS } from "@mdi/js";
import { actions } from "../../../../../services/actions";
import { sampler } from "../../../../../sampler";
import { TokenizedText } from "../../../../../ui/text";
import classNames from "classnames";

interface InstrumentOverviewRowProps {
  instrument: Instrument;
  color: string;
  isInSelectedFlow: boolean;
}

export const InstrumentOverviewRow: FC<InstrumentOverviewRowProps> = ({
  instrument,
  color,
  isInSelectedFlow,
}) => {
  const samplerState = useSamplerState();

  const { isMuted, isSoloed } = useMemo(() => {
    const state = samplerState[instrument.id];
    return {
      isMuted: state?.mute || false,
      isSoloed: state?.solo || false,
    };
  }, [samplerState, instrument.id]);

  const handleToggleMute = () => {
    const newMuted = !isMuted;
    actions.score.instrument.setMute(instrument.id, newMuted);
    if (newMuted) {
      sampler.mute(instrument.id.toString());
    } else {
      sampler.unmute(instrument.id.toString());
    }
  };

  const handleToggleSolo = () => {
    const newSoloed = !isSoloed;
    actions.score.instrument.setSolo(instrument.id, newSoloed);
    if (newSoloed) {
      sampler.solo(instrument.id.toString());
    } else {
      sampler.unsolo(instrument.id.toString());
    }
  };

  return (
    <div
      className={classNames("flex p-2 gap-2 border-l-4 bg-neutral-900 h-16", {
        "opacity-50": !isInSelectedFlow,
      })}
      style={{ borderLeftColor: isInSelectedFlow ? color : "#666" }}
    >
      {/* Instrument name */}
      <div className="flex-1 min-w-0">
        <div className="text-sm text-white truncate">
          <TokenizedText>{instrument.long_name}</TokenizedText>
          <p className="text-[11px] opacity-50">{instrument.definition_id}</p>
        </div>
      </div>

      <div className="flex items-end gap-2">
        <IconButton
          path={mdiSizeS}
          variant="filled"
          size={0.8}
          isSelected={isSoloed}
          onClick={handleToggleSolo}
          isDisabled={!isInSelectedFlow}
        />
        <IconButton
          path={mdiSizeM}
          variant="filled"
          size={0.8}
          isSelected={isMuted}
          onClick={handleToggleMute}
          isDisabled={!isInSelectedFlow}
        />
      </div>
    </div>
  );
};
