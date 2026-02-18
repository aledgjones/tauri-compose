import { FC, useMemo } from "react";
import { Instrument } from "../../../../../services/types";
import { IconButton } from "../../../../../ui/buttons/icon-button";
import { mdiCableData, mdiSizeM, mdiSizeS } from "@mdi/js";
import { TokenizedText } from "../../../../../ui/text";
import classNames from "classnames";
import { Select } from "../../../../../ui/select";
import { useSamplerState } from "../../../../../services/state";
import { actions } from "../../../../../services/actions";

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
  const sampler = useSamplerState();
  const { plugins } = sampler;

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
        <div className="text-sm text-white truncate">
          <TokenizedText>{instrument.long_name}</TokenizedText>
          {instrument.count && <span> {instrument.count}</span>}
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

      <Select
        className="flex-1"
        value={instrument.assigned_plugin_id || ""}
        options={options}
        onChange={handlePluginChange}
        icon={mdiCableData}
        variant="filled"
        placeholder="Unassigned"
      />
    </div>
  );
};
