import { FC } from "react";
import { Instrument } from "../../../../../services/types";
import { IconButton } from "../../../../../ui/buttons/icon-button";
import { mdiSizeM, mdiSizeS } from "@mdi/js";
import { TokenizedText } from "../../../../../ui/text";
import classNames from "classnames";
import { useAppState } from "../../../../../services/state";
import { actions } from "../../../../../services/actions";
import { PlaybackReadinessBadge } from "./playback-readiness";
import { getPlaybackReadinessPresentation } from "./playback-readiness-presentation";

interface InstrumentOverviewRowProps {
  instrument: Instrument;
  color: string;
  isDisabled: boolean;
}

export const InstrumentOverviewRow: FC<InstrumentOverviewRowProps> = ({
  instrument,
  color,
  isDisabled,
}) => {
  const state = useAppState();
  const routeState =
    state.resolved_playback_route_by_instrument_id[instrument.id];
  const presentation = getPlaybackReadinessPresentation(routeState, instrument);

  return (
    <div
      data-route-focus-for={instrument.id}
      tabIndex={-1}
      className={classNames(
        "flex flex-col p-2 gap-2 border-l-4 bg-neutral-900",
        { "opacity-50": isDisabled },
      )}
      style={{ borderLeftColor: isDisabled ? "#444" : color }}
    >
      {/* Instrument name */}
      <div className="flex items-center justify-between flex-1 min-w-0">
        <div className="min-w-0 flex flex-1 items-center gap-2">
          <div className="text-sm text-white truncate">
            <TokenizedText>{instrument.long_name}</TokenizedText>
            {instrument.count && <span> {instrument.count}</span>}
          </div>
          <PlaybackReadinessBadge
            presentation={presentation}
            className="max-w-full shrink"
          />
        </div>

        {!isDisabled && (
          <div className="flex items-end gap-2">
            <IconButton
              path={mdiSizeS}
              variant="filled"
              size={0.9}
              isSelected={instrument.solo}
              onClick={() =>
                actions.score.instrument.setSolo(
                  instrument.id,
                  !instrument.solo,
                )
              }
            />
            <IconButton
              path={mdiSizeM}
              variant="filled"
              size={0.9}
              isSelected={instrument.mute}
              onClick={() =>
                actions.score.instrument.setMute(
                  instrument.id,
                  !instrument.mute,
                )
              }
            />
          </div>
        )}
      </div>

      <div className="text-xs text-neutral-400">
        {presentation.plugin_label
          ? `Route target: ${presentation.plugin_label}`
          : "No route target configured"}
      </div>
    </div>
  );
};
