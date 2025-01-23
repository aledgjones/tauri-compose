import { FC } from "react";
import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";
import { BracketStyle, LayoutType } from "../../../services/types";
import {
  SelectByImage,
  SelectByImageOption,
} from "../../../components/select-by-image";

import bracketStyleNoneImg from "../assets/brackets-none.webp";
import bracketStyleLineImg from "../assets/brackets-line.webp";
import bracketStyleWingImg from "../assets/brackets-wing.webp";

const brackStyleOptions: SelectByImageOption<BracketStyle>[] = [
  { label: "Wings", image: bracketStyleWingImg, value: BracketStyle.Wing },
  {
    label: "Horizontal Line",
    image: bracketStyleLineImg,
    value: BracketStyle.Line,
  },
  { label: "None", image: bracketStyleNoneImg, value: BracketStyle.None },
];

type Props = { layoutType: LayoutType };

export const BracketsAndBraces: FC<Props> = ({ layoutType }) => {
  const state = useAppState();

  return (
    <div className="p-2 overflow-y-auto min-h-full">
      <SelectByImage
        label="Bracketing Style"
        value={state.score.engraves.by_layout_type[layoutType].bracket_style}
        options={brackStyleOptions}
        onChange={(bracketStyle) =>
          actions.score.engrave.setBracketStyle(layoutType, bracketStyle)
        }
      />
    </div>
  );
};
