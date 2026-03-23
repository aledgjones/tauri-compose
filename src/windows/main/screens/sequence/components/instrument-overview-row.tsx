import { FC } from "react";
import { BottomPanelTab, Instrument } from "../../../../../services/types";
import { IconButton } from "../../../../../ui/buttons/icon-button";
import { mdiSizeM, mdiSizeS } from "@mdi/js";
import { TokenizedText } from "../../../../../ui/text";
import classNames from "classnames";
import {
  useAppState,
  usePluginInventoryState,
} from "../../../../../services/state";
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
  const pluginInventory = usePluginInventoryState();
  const routeState =
    state.resolved_playback_route_by_instrument_id[instrument.id];
  const presentation = getPlaybackReadinessPresentation(
    routeState,
    instrument,
    pluginInventory.entries,
    state.score.playback?.rack,
  );
  const instrumentLabel = instrument.count
    ? `${instrument.long_name} ${instrument.count}`
    : instrument.long_name;

  const handleOpenRack = () => {
    actions.ui.sequencer.setInstrument(instrument.id);
    actions.ui.screens.sequence.bottomPanel.setTab(BottomPanelTab.Instruments);
  };

  return (
    <div
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

      <div className="flex flex-col gap-1.5 text-xs text-neutral-300">
        <div className="text-[11px] uppercase tracking-[0.18em] text-neutral-500">
          Route summary
        </div>
        <div>{presentation.slot_summary}</div>
        <div>{presentation.target_summary}</div>
        {presentation.address_summary ? (
          <div>{presentation.address_summary}</div>
        ) : null}
        {presentation.action_label ? (
          <button
            type="button"
            onClick={handleOpenRack}
            className="mt-1 w-fit border border-sky-700/70 bg-sky-500/10 px-3 py-1.5 text-[11px] uppercase tracking-[0.14em] text-sky-100 hover:bg-sky-500/20"
            aria-label={`Open rack for ${instrumentLabel}`}
          >
            {presentation.action_label}
          </button>
        ) : null}
      </div>
    </div>
  );
};
