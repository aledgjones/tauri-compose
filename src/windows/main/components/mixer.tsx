import { FC, useMemo } from "react";
import { generateRainbowColors } from "../../../services/utils";
import { MixerChannel } from "./mixer-channel";
import { actions } from "../../../services/actions";
import { Instrument } from "../../../services/types";

type Props = { instruments: Instrument[] };

export const Mixer: FC<Props> = ({ instruments }) => {
  const instrumentColors = useMemo(() => {
    return generateRainbowColors(instruments.length);
  }, [instruments.length]);

  return (
    <div className="flex w-full h-full overflow-auto gap-2 p-2 pt-0">
      {instruments.map((instrument, index) => (
        <MixerChannel
          key={instrument.id}
          instrument={instrument}
          amplitude={0} // TODO: Calculate from audio levels
          color={instrumentColors[index]}
          onVolumeChange={(volume: number) =>
            actions.score.instrument.setVolume(instrument.id, volume)
          }
          onToggleMute={() =>
            actions.score.instrument.setMute(instrument.id, !instrument.mute)
          }
          onToggleSolo={() =>
            actions.score.instrument.setSolo(instrument.id, !instrument.solo)
          }
        />
      ))}
    </div>
  );
};
