import { FC } from "react";
import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";
import { BracketStyle, LayoutType } from "../../../services/types";
import { Select, SelectOption } from "../../../ui/select";

const brackStyleOptions: SelectOption<BracketStyle>[] = [
  { label: "None", value: BracketStyle.None },
  { label: "Line", value: BracketStyle.Line },
  { label: "Wing", value: BracketStyle.Wing },
];

type Props = { layoutType: LayoutType };

export const BracketsAndBraces: FC<Props> = ({ layoutType }) => {
  const state = useAppState();

  return (
    <div className="p-2 pt-1 overflow-y-auto min-h-full">
      <Select
        variant="filled"
        label="Bracketing Style"
        value={state.score.engraves.by_layout_type[layoutType].bracket_style}
        options={brackStyleOptions}
        onChange={(bracketStyle) =>
          actions.score.engrave.setBracketStyle(layoutType, bracketStyle)
        }
      />
      {/* <Select
        variant="filled"
        label="Bracketing Approach"
        value={state.score.config.auto_count_styles.section}
        options={autoCountOptions}
        onChange={actions.score.config.autoCountStyles.setSection}
      /> */}
    </div>
  );
};
