import { mdiDelete } from "@mdi/js";
import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { useMemo } from "preact/hooks";
import { Instrument } from "../../../../../services/types";
import { IconButton } from "../../../../../ui/icon-button";

interface Props {
  isSelected: boolean;
  instrument: Instrument;
}

export const PlayerPanelInstrumentListItem: FunctionalComponent<Props> = ({
  isSelected,
  instrument,
}) => {
  const bg = isSelected ? "bg-sky-50" : "bg-neutral-700";
  const fg = isSelected ? "text-black" : "text-white";

  const name = useMemo(() => {
    return (
      instrument.long_name + (instrument.count ? ` ${instrument.count}` : "")
    );
  }, []);

  return (
    <div
      data-player-id={instrument.id}
      className={classNames(
        `flex items-center pl-3 pr-1 rounded-lg ${bg} ${fg} min-h-[48px]`
      )}
    >
      <p className="truncate flex-1 text-sm">{name}</p>

      {isSelected && (
        <IconButton
          variant="standard"
          className={fg}
          path={mdiDelete}
          onClick={() => {
            // core.unassign_instrument_from_player(playerKey, instrumentId);
            // core.remove_instrument(instrumentId);
          }}
        />
      )}
    </div>
  );
};
