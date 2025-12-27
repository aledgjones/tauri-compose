import { FC } from "react";
import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";
import { LayoutType } from "../../../services/types";
import { Input } from "../../../ui/input";

type Props = { layoutType: LayoutType };

export const NoteSpacing: FC<Props> = ({ layoutType }) => {
  const state = useAppState();

  return (
    <div className="bg-neutral-900 mb-2">
      <h3 className="p-2 text-sm font-medium text-white bg-neutral-800">
        Note Spacing
      </h3>
      <div className="p-4">
        <Input
          type="number"
          label="Base Note Space"
          units="spaces"
          value={
            state.score.engraves.by_layout_type[layoutType].base_note_space
          }
          onChange={(value: number) =>
            actions.score.engrave.setBaseNoteSpace(layoutType, value)
          }
          min={0}
          step={0.1}
          decimalPlaces={1}
          className="w-48"
        />

        <Input
          type="number"
          label="Minimum Note Space"
          units="spaces"
          value={
            state.score.engraves.by_layout_type[layoutType].minimum_note_space
          }
          onChange={(value: number) =>
            actions.score.engrave.setMinimumNoteSpace(layoutType, value)
          }
          min={0}
          step={0.01}
          decimalPlaces={2}
          className="w-48"
        />

        <Input
          type="number"
          label="Minimum Tie Space"
          units="spaces"
          value={
            state.score.engraves.by_layout_type[layoutType].minimum_tie_space
          }
          onChange={(value: number) =>
            actions.score.engrave.setMinimumTieSpace(layoutType, value)
          }
          min={0}
          step={0.01}
          decimalPlaces={2}
          className="w-48"
        />

        <Input
          type="number"
          label="Note Space Ratio"
          value={
            state.score.engraves.by_layout_type[layoutType].note_space_ratio
          }
          onChange={(value: number) =>
            actions.score.engrave.setNoteSpaceRatio(layoutType, value)
          }
          min={1}
          step={0.01}
          decimalPlaces={2}
          className="w-48"
        />
      </div>
    </div>
  );
};
