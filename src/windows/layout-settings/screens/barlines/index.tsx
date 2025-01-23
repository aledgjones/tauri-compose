import { FC } from "react";
import { actions } from "../../../../services/actions";
import { useAppState } from "../../../../services/state";
import { LayoutType } from "../../../../services/types";
import {
  SelectByImage,
  SelectByImageOption,
} from "../../../../components/select-by-image";

import singleSystemicImg from "./assets/single-systemic.webp";
import noSingleSystemicImg from "./assets/no-single-systemic.webp";

const singleSystemicOptions: SelectByImageOption<"true" | "false">[] = [
  {
    label: "Show for one omore staves",
    image: singleSystemicImg,
    value: "true",
  },
  {
    label: "Show for two or more staves",
    image: noSingleSystemicImg,
    value: "false",
  },
];

type Props = { layoutType: LayoutType };

export const Barlines: FC<Props> = ({ layoutType }) => {
  const state = useAppState();

  return (
    <div className="p-2 overflow-y-auto min-h-full">
      <SelectByImage
        label="Systemic Barlines"
        value={
          state.score.engraves.by_layout_type[layoutType]
            .systemic_barline_single_instrument_system
            ? "true"
            : "false"
        }
        options={singleSystemicOptions}
        onChange={(enabled) =>
          actions.score.engrave.setSystemicBarlineSingleInstrumentSystem(
            layoutType,
            enabled === "true" ? true : false
          )
        }
      />
    </div>
  );
};
