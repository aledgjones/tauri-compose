import { FC, useMemo } from "react";
import { generateRainbowColors } from "../../../services/utils";
import { MixerChannel } from "./mixer-channel";
import { Instrument, ResolvedPlaybackRoute } from "../../../services/types";
import { IconButton } from "../../../ui/buttons/icon-button";
import { mdiSizeM, mdiSizeS } from "@mdi/js";
import { actions } from "../../../services/actions";

interface MixerToolbarProps {
  instruments: Instrument[];
  resolvedPlaybackRouteByInstrumentId: Record<number, ResolvedPlaybackRoute>;
}

const supportsMixControl = (
  resolvedPlaybackRouteByInstrumentId: Record<number, ResolvedPlaybackRoute>,
  instrumentId: number,
  control: "volume" | "pan" | "mute" | "solo",
) => {
  return Boolean(
    resolvedPlaybackRouteByInstrumentId[instrumentId]?.supported_mix_controls?.[
      control
    ],
  );
};

export const MixerToolbar: FC<MixerToolbarProps> = ({
  instruments,
  resolvedPlaybackRouteByInstrumentId,
}) => {
  const { hasSolo, hasMute } = useMemo(() => {
    let hasSolo = false;
    let hasMute = false;

    for (const instrument of instruments) {
      if (
        supportsMixControl(
          resolvedPlaybackRouteByInstrumentId,
          instrument.id,
          "solo",
        ) &&
        instrument.solo
      ) {
        hasSolo = true;
      }

      if (
        supportsMixControl(
          resolvedPlaybackRouteByInstrumentId,
          instrument.id,
          "mute",
        ) &&
        instrument.mute
      ) {
        hasMute = true;
      }

      if (hasSolo && hasMute) {
        break;
      }
    }

    return { hasSolo, hasMute };
  }, [instruments, resolvedPlaybackRouteByInstrumentId]);

  const handleToggleAllSolo = () => {
    const targetState = !hasSolo;
    instruments.forEach((instrument) => {
      if (
        supportsMixControl(
          resolvedPlaybackRouteByInstrumentId,
          instrument.id,
          "solo",
        )
      ) {
        actions.score.instrument.setSolo(instrument.id, targetState);
      }
    });
  };

  const handleToggleAllMute = () => {
    const targetState = !hasMute;
    instruments.forEach((instrument) => {
      if (
        supportsMixControl(
          resolvedPlaybackRouteByInstrumentId,
          instrument.id,
          "mute",
        )
      ) {
        actions.score.instrument.setMute(instrument.id, targetState);
      }
    });
  };

  return (
    <>
      <div className="flex flex-1 gap-0.5">
        {instruments.some((instrument) =>
          supportsMixControl(
            resolvedPlaybackRouteByInstrumentId,
            instrument.id,
            "solo",
          ),
        ) && (
          <IconButton
            path={mdiSizeS}
            isSelected={hasSolo}
            variant="interactive"
            onClick={handleToggleAllSolo}
            aria-label="Solo all supported instruments"
            title="Solo all supported instruments"
          />
        )}
        {instruments.some((instrument) =>
          supportsMixControl(
            resolvedPlaybackRouteByInstrumentId,
            instrument.id,
            "mute",
          ),
        ) && (
          <IconButton
            path={mdiSizeM}
            isSelected={hasMute}
            variant="interactive"
            onClick={handleToggleAllMute}
            aria-label="Mute all supported instruments"
            title="Mute all supported instruments"
          />
        )}
        <div className="bg-neutral-900 flex-1" />
      </div>
    </>
  );
};

type Props = {
  instruments: Instrument[];
  resolvedPlaybackRouteByInstrumentId: Record<number, ResolvedPlaybackRoute>;
};

export const Mixer: FC<Props> = ({
  instruments,
  resolvedPlaybackRouteByInstrumentId,
}) => {
  const instrumentColors = useMemo(() => {
    return generateRainbowColors(instruments.length);
  }, [instruments.length]);

  return (
    <div className="flex w-full h-full overflow-auto gap-0.5">
      {instruments.map((instrument, index) => {
        const route = resolvedPlaybackRouteByInstrumentId[instrument.id];
        return (
          <MixerChannel
            key={instrument.id}
            instrument={instrument}
            color={instrumentColors[index]}
            showPanControl={Boolean(route?.supported_mix_controls?.pan)}
            showVolumeControl={Boolean(route?.supported_mix_controls?.volume)}
            showMuteControl={Boolean(route?.supported_mix_controls?.mute)}
            showSoloControl={Boolean(route?.supported_mix_controls?.solo)}
            pan={instrument.pan}
            volume={Math.round((instrument.volume / 127) * 100)}
            isMuted={instrument.mute}
            isSoloed={instrument.solo}
            onPanChange={(midiValue) => {
              actions.score.instrument.setPan(instrument.id, midiValue);
            }}
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
