import { FC, useMemo } from "react";
import { Instrument } from "../../../../../services/types";
import { IconButton } from "../../../../../ui/buttons/icon-button";
import { mdiCableData, mdiSizeM, mdiSizeS } from "@mdi/js";
import { TokenizedText } from "../../../../../ui/text";
import classNames from "classnames";
import { Select } from "../../../../../ui/select";
import { useAppState, useSamplerState } from "../../../../../services/state";
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
  const sampler = useSamplerState();
  const { plugins } = sampler;
  const assignmentState =
    state.resolved_playback_assignment_by_instrument_id[instrument.id];
  const presentation = getPlaybackReadinessPresentation(
    assignmentState,
    instrument,
  );

  const options = useMemo(() => {
    return Object.values(plugins).map((plugin) => ({
      label: plugin.name,
      value: plugin.id,
    }));
  }, [plugins]);

  const handlePluginChange = async (pluginId: string) => {
    await actions.score.instrument.assignPlugin(
      instrument.id,
      pluginId || null,
    );
  };

  return (
    <div
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

      <div data-assignment-select-for={instrument.id}>
        <Select
          className="flex-1"
          value={assignmentState?.effective_plugin_id || ""}
          options={options}
          onChange={handlePluginChange}
          icon={mdiCableData}
          variant="filled"
          placeholder={
            options.length === 0 ? "No plugins available" : "Assign plugin"
          }
          disabled={isDisabled || options.length === 0}
        />
      </div>
    </div>
  );
};
