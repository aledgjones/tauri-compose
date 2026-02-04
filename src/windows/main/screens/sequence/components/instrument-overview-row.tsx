import { FC, useMemo, useState } from "react";
import { Instrument } from "../../../../../services/types";
import { IconButton } from "../../../../../ui/icon-button";
import { mdiCableData, mdiSizeM, mdiSizeS } from "@mdi/js";
import { TokenizedText } from "../../../../../ui/text";
import classNames from "classnames";
import { Select } from "../../../../../ui/select";
import { useSamplerState } from "../../../../../services/state";

// TODO hook this up properly
const samplerState: Record<
  string,
  { volume: number; solo: boolean; mute: boolean }
> = {};

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
  const [assignedPluginId, setAssignedPluginId] = useState<string>("");

  const { plugins } = sampler;

  const { isMuted, isSoloed } = useMemo(() => {
    const state = samplerState[instrument.id];
    return {
      isMuted: state?.mute || false,
      isSoloed: state?.solo || false,
    };
  }, [instrument.id]);

  const options = useMemo(() => {
    return Object.values(plugins).map((plugin) => ({
      label: plugin.name,
      value: plugin.id,
    }));
  }, [plugins]);

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
              isSelected={isSoloed}
              onClick={() => {}}
            />
            <IconButton
              path={mdiSizeM}
              variant="filled"
              size={0.9}
              isSelected={isMuted}
              onClick={() => {}}
            />
          </div>
        )}
      </div>

      <Select
        className="flex-1"
        value={assignedPluginId}
        options={options}
        onChange={setAssignedPluginId}
        icon={mdiCableData}
        variant="filled"
        placeholder="Unassigned"
      />
    </div>
  );
};
