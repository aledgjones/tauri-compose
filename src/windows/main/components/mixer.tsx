import { FC, useMemo } from "react";
import { generateRainbowColors } from "../../../services/utils";
import { MixerChannel } from "./mixer-channel";
import { Instrument } from "../../../services/types";
import { IconButton } from "../../../ui/icon-button";
import { mdiSizeM, mdiSizeS } from "@mdi/js";

// TODO hook this up properly
const samplerState: Record<
  string,
  { volume: number; solo: boolean; mute: boolean }
> = {};

interface MixerToolbarProps {
  instruments: Instrument[];
}

export const MixerToolbar: FC<MixerToolbarProps> = ({ instruments }) => {
  const { hasSolo, hasMute } = useMemo(() => {
    let hasSolo = false;
    let hasMute = false;

    for (const instrument of instruments) {
      const state = samplerState[instrument.id];
      if (!state) {
        continue;
      }

      if (state.solo) {
        hasSolo = true;
      }

      if (state.mute) {
        hasMute = true;
      }

      if (hasSolo && hasMute) {
        break;
      }
    }

    return { hasSolo, hasMute };
  }, [instruments]);

  return (
    <>
      <div className="flex flex-1 gap-0.5">
        <IconButton
          path={mdiSizeS}
          isSelected={hasSolo}
          variant="interactive"
          onClick={() => {}}
        />
        <IconButton
          path={mdiSizeM}
          isSelected={hasMute}
          variant="interactive"
          onClick={() => {}}
        />
        <div className="bg-neutral-900 flex-1" />
      </div>
    </>
  );
};

type Props = { instruments: Instrument[] };

export const Mixer: FC<Props> = ({ instruments }) => {
  const instrumentColors = useMemo(() => {
    return generateRainbowColors(instruments.length);
  }, [instruments.length]);

  return (
    <div className="flex w-full h-full overflow-auto gap-0.5">
      {instruments.map((instrument, index) => {
        const state = samplerState[instrument.id] || {
          volume: 80,
          mute: false,
          solo: false,
        };
        return (
          <MixerChannel
            key={instrument.id}
            instrument={instrument}
            color={instrumentColors[index]}
            volume={Math.round((state.volume / 127) * 100)}
            isMuted={state.mute}
            isSoloed={state.solo}
            onVolumeChange={() => {}}
            onToggleMute={() => {}}
            onToggleSolo={() => {}}
          />
        );
      })}
      <div className="bg-neutral-900 flex-1" />
    </div>
  );
};
