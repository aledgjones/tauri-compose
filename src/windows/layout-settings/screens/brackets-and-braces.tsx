import { FC } from "react";
import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";
import {
  BracketingApproach,
  BracketStyle,
  LayoutType,
} from "../../../services/types";
import { Select, SelectOption } from "../../../ui/select";

const bracketStyleOptions: SelectOption<BracketStyle>[] = [
  { label: "Wings", value: BracketStyle.Wing },
  { label: "Horizontal line", value: BracketStyle.Line },
  { label: "None", value: BracketStyle.None },
];

const bracketingApproachOptions: SelectOption<BracketingApproach>[] = [
  { label: "No brackets", value: BracketingApproach.None },
  { label: "Orchestral", value: BracketingApproach.Orchestral },
  { label: "Small ensemble", value: BracketingApproach.SmallEnsemble },
];

const bracketSingleStavesOptions: SelectOption<"true" | "false">[] = [
  {
    label: "No bracket",
    value: "false",
  },
  {
    label: "Use bracket",
    value: "true",
  },
];

type Props = { layoutType: LayoutType };

export const BracketsAndBraces: FC<Props> = ({ layoutType }) => {
  const state = useAppState();

  return (
    <div className="p-2 overflow-y-auto min-h-full">
      <Select
        variant="filled"
        label="Bracketing Style"
        value={state.score.engraves.by_layout_type[layoutType].bracket_style}
        options={bracketStyleOptions}
        onChange={(bracketStyle) =>
          actions.score.engrave.setBracketStyle(layoutType, bracketStyle)
        }
      />

      <Select
        variant="filled"
        label="Bracketing Approach"
        value={
          state.score.engraves.by_layout_type[layoutType].bracketing_approach
        }
        options={bracketingApproachOptions}
        onChange={(bracketStyle) =>
          actions.score.engrave.setBracketingApproach(layoutType, bracketStyle)
        }
      />

      <Select
        variant="filled"
        label="Bracketed groups with only one instrument"
        value={
          state.score.engraves.by_layout_type[layoutType].bracket_single_staves
            ? "true"
            : "false"
        }
        options={bracketSingleStavesOptions}
        onChange={(enabled) =>
          actions.score.engrave.setBracketSingleStaves(
            layoutType,
            enabled === "true" ? true : false
          )
        }
      />
    </div>
  );
};
