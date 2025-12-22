import { FC, useMemo, useEffect } from "react";
import { generateRainbowColors } from "../../../services/utils";
import { MixerChannel } from "./mixer-channel";
import { actions } from "../../../services/actions";
import { sampler } from "../../../sampler";
import { Instrument } from "../../../services/types";
import { IconButton } from "../../../ui/icon-button";
import { mdiSizeM, mdiSizeS } from "@mdi/js";
import { useSamplerState } from "../../../services/state";

interface MixerToolbarProps {
  instruments: Instrument[];
}

export const MixerToolbar: FC<MixerToolbarProps> = ({ instruments }) => {
  const samplerState = useSamplerState();

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
  }, [samplerState, instruments]);

  return (
    <>
      <div className="flex flex-1 gap-2">
        <IconButton
          path={mdiSizeS}
          isSelected={hasSolo}
          variant="filled"
          onClick={() => {
            instruments.forEach((instrument) => {
              actions.score.instrument.setSolo(instrument.id, !hasSolo);
              if (hasSolo) {
                sampler.unsolo(instrument.id.toString());
              } else {
                sampler.solo(instrument.id.toString());
              }
            });
          }}
        />
        <IconButton
          path={mdiSizeM}
          isSelected={hasMute}
          variant="filled"
          onClick={() => {
            instruments.forEach((instrument) => {
              actions.score.instrument.setMute(instrument.id, !hasMute);
              if (hasMute) {
                sampler.unmute(instrument.id.toString());
              } else {
                sampler.mute(instrument.id.toString());
              }
            });
          }}
        />
      </div>
    </>
  );
};

type Props = { instruments: Instrument[] };

export const Mixer: FC<Props> = ({ instruments }) => {
  const samplerState = useSamplerState();
  const instrumentColors = useMemo(() => {
    return generateRainbowColors(instruments.length);
  }, [instruments.length]);

  // Initialize sampler volumes, mute, and solo states from state
  useEffect(() => {
    instruments.forEach((instrument) => {
      const instrumentId = instrument.id.toString();
      const state = samplerState[instrument.id];
      if (state) {
        sampler.volume(instrumentId, state.volume / 127.0);
        if (state.mute) {
          sampler.mute(instrumentId);
        } else {
          sampler.unmute(instrumentId);
        }
        if (state.solo) {
          sampler.solo(instrumentId);
        } else {
          sampler.unsolo(instrumentId);
        }
      }
    });
  }, [instruments, samplerState]);

  return (
    <div className="flex w-full h-full overflow-auto gap-2">
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
            onVolumeChange={(volume: number) => {
              actions.score.instrument.setVolume(instrument.id, volume);
              sampler.volume(instrument.id.toString(), volume / 127.0);
            }}
            onToggleMute={() => {
              const newMute = !state.mute;
              actions.score.instrument.setMute(instrument.id, newMute);
              if (newMute) {
                sampler.mute(instrument.id.toString());
              } else {
                sampler.unmute(instrument.id.toString());
              }
            }}
            onToggleSolo={() => {
              const newSolo = !state.solo;
              actions.score.instrument.setSolo(instrument.id, newSolo);
              if (newSolo) {
                sampler.solo(instrument.id.toString());
              } else {
                sampler.unsolo(instrument.id.toString());
              }
            }}
          />
        );
      })}
    </div>
  );
};
