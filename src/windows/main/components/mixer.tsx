import { FC, useMemo } from "react";
import { generateRainbowColors } from "../../../services/utils";
import { MixerChannel } from "./mixer-channel";
import { Instrument } from "../../../services/types";
import { IconButton } from "../../../ui/icon-button";
import { mdiSizeM, mdiSizeS } from "@mdi/js";
import { actions } from "../../../services/actions";

interface MixerToolbarProps {
  instruments: Instrument[];
}

export const MixerToolbar: FC<MixerToolbarProps> = ({ instruments }) => {
  const { hasSolo, hasMute } = useMemo(() => {
    let hasSolo = false;
    let hasMute = false;

    for (const instrument of instruments) {
      if (instrument.solo) {
        hasSolo = true;
      }

      if (instrument.mute) {
        hasMute = true;
      }

      if (hasSolo && hasMute) {
        break;
      }
    }

    return { hasSolo, hasMute };
  }, [instruments]);

  const handleToggleAllSolo = () => {
    const targetState = !hasSolo;
    instruments.forEach((instrument) => {
      actions.score.instrument.setSolo(instrument.id, targetState);
    });
  };

  const handleToggleAllMute = () => {
    const targetState = !hasMute;
    instruments.forEach((instrument) => {
      actions.score.instrument.setMute(instrument.id, targetState);
    });
  };

  return (
    <>
      <div className="flex flex-1 gap-0.5">
        <IconButton
          path={mdiSizeS}
          isSelected={hasSolo}
          variant="interactive"
          onClick={handleToggleAllSolo}
        />
        <IconButton
          path={mdiSizeM}
          isSelected={hasMute}
          variant="interactive"
          onClick={handleToggleAllMute}
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
        return (
          <MixerChannel
            key={instrument.id}
            instrument={instrument}
            color={instrumentColors[index]}
            volume={Math.round((instrument.volume / 127) * 100)}
            isMuted={instrument.mute}
            isSoloed={instrument.solo}
            onVolumeChange={(midiValue) => {
              actions.score.instrument.setVolume(instrument.id, midiValue);
            }}
            onToggleMute={() => {
              actions.score.instrument.setMute(instrument.id, !instrument.mute);
            }}
            onToggleSolo={() => {
              actions.score.instrument.setSolo(instrument.id, !instrument.solo);
            }}
          />
        );
      })}
      <div className="bg-neutral-900 flex-1" />
    </div>
  );
};
