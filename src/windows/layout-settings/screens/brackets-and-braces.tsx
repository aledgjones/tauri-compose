import { FC } from "react";
import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";
import {
  BracketingApproach,
  BracketStyle,
  LayoutType,
} from "../../../services/types";
import { Select, SelectOption } from "../../../ui/select";
import { Switch } from "../../../ui/switch";

const bracketStyleOptions: SelectOption<BracketStyle>[] = [
  { label: "None", value: BracketStyle.None },
  { label: "Horizontal line", value: BracketStyle.Line },
  { label: "Wings", value: BracketStyle.Wing },
];

const bracketingApproachOptions: SelectOption<BracketingApproach>[] = [
  { label: "No brackets", value: BracketingApproach.None },
  { label: "Orchestral", value: BracketingApproach.Orchestral },
  { label: "Small ensemble", value: BracketingApproach.SmallEnsemble },
];

type Props = { layoutType: LayoutType };

export const BracketsAndBraces: FC<Props> = ({ layoutType }) => {
  const state = useAppState();

  return (
    <>
      <div className="bg-neutral-900">
        <h3 className="p-2 text-sm font-medium text-white bg-neutral-800">
          Brackets
        </h3>
        <div className="p-4">
          <Select
            variant="filled"
            label="Bracketing Approach"
            value={
              state.score.engraves.by_layout_type[layoutType]
                .bracketing_approach
            }
            options={bracketingApproachOptions}
            onChange={(bracketStyle) =>
              actions.score.engrave.setBracketingApproach(
                layoutType,
                bracketStyle,
              )
            }
            className="w-48"
          />

          <Select
            variant="filled"
            label="Bracketing Style"
            value={
              state.score.engraves.by_layout_type[layoutType].bracket_style
            }
            options={bracketStyleOptions}
            onChange={(bracketStyle) =>
              actions.score.engrave.setBracketStyle(layoutType, bracketStyle)
            }
            className="w-48"
          />
        </div>
      </div>
      <div className="bg-neutral-900">
        <h3 className="p-2 text-sm font-medium text-white bg-neutral-800">
          Bracketed Groups
        </h3>
        <div className="p-4">
          <Switch
            label="Show brackets for single staves"
            description={
              state.score.engraves.by_layout_type[layoutType]
                .bracket_single_staves
                ? "Show brackets for systems with only one staff."
                : "Show brackets for systems with two or more staves."
            }
            value={
              state.score.engraves.by_layout_type[layoutType]
                .bracket_single_staves
            }
            onChange={(value) =>
              actions.score.engrave.setBracketSingleStaves(layoutType, value)
            }
          />
        </div>
      </div>
    </>
  );
};
