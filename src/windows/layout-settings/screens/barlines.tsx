import { FC } from "react";
import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";
import { LayoutType } from "../../../services/types";
import { Switch } from "../../../ui/switch";

type Props = { layoutType: LayoutType };

export const Barlines: FC<Props> = ({ layoutType }) => {
  const state = useAppState();

  const value =
    state.score.engraves.by_layout_type[layoutType]
      .systemic_barline_single_instrument_system;

  return (
    <div className="gap-2 flex flex-col">
      <div className="bg-neutral-900">
        <h3 className="p-2 text-sm font-medium text-white bg-neutral-800">
          Systemic Barlines
        </h3>
        <div className="p-4">
          <Switch
            label="Show systemic barlines for single staves"
            description={
              value
                ? "Show systemic barlines for systems with only one staff."
                : "Show systemic barlines for systems with two or more staves."
            }
            value={value}
            onChange={(value) =>
              actions.score.engrave.setSystemicBarlineSingleInstrumentSystem(
                layoutType,
                value,
              )
            }
          />
        </div>
      </div>
    </div>
  );
};
