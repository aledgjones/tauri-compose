import { mdiDelete } from "@mdi/js";
import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { useMemo } from "preact/hooks";
import { Instrument } from "../../../../../services/types";
import { IconButton } from "../../../../../ui/icon-button";
import { actions } from "../../../../../services/actions";
import { TokenizedText } from "../../../../../ui/text";

const onRemoveInstrumentPress = async (playerId: number, instrumentId: number) => {
  await actions.score.player.unassignInstrument(playerId, instrumentId);
  await actions.score.instrument.remove(instrumentId);
};

interface Props {
  playerId: number;
  instrument: Instrument;
  isSelected: boolean;
}

export const PlayerPanelInstrumentListItem: FunctionalComponent<Props> = ({
  playerId,
  instrument,
  isSelected,
}) => {
  const bg = isSelected ? "bg-white" : "bg-neutral-700";
  const fg = isSelected ? "text-black" : "text-white";

  const name = useMemo(() => {
    return instrument.long_name + (instrument.count ? ` ${instrument.count}` : "");
  }, [instrument.long_name, instrument.count]);

  return (
    <div
      data-instrument-id={instrument.id}
      className={classNames("flex items-center pl-3 pr-1 min-h-[48px]", bg, fg)}
    >
      <p className="truncate flex-1 text-sm">
        <TokenizedText>{name}</TokenizedText>
      </p>

      {isSelected && (
        <IconButton
          variant="standard"
          className={fg}
          path={mdiDelete}
          onClick={() => onRemoveInstrumentPress(playerId, instrument.id)}
        />
      )}
    </div>
  );
};
