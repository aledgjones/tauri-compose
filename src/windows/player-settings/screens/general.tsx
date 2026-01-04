import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";
import { AutoCountStyle } from "../../../services/types";
import { Select, SelectOption } from "../../../ui/select";

const autoCountOptions: SelectOption<AutoCountStyle>[] = [
  { label: "Arabic (1, 2, 3...)", value: AutoCountStyle.Arabic },
  { label: "Roman (I, II, III...)", value: AutoCountStyle.Roman },
];

export const Meta = () => {
  const state = useAppState();

  return (
    <div className="bg-neutral-900">
      <h3 className="p-2 text-sm font-medium text-white bg-neutral-800">
        Systemic Barlines
      </h3>
      <div className="p-4 gap-2 flex flex-col">
        <Select
          variant="filled"
          label="Numbering style for single players"
          value={state.score.config.auto_count_styles.solo}
          options={autoCountOptions}
          onChange={actions.score.config.autoCountStyles.setSolo}
          className="w-1/2"
        />
        <Select
          variant="filled"
          label="Numbering style for section players"
          value={state.score.config.auto_count_styles.section}
          options={autoCountOptions}
          onChange={actions.score.config.autoCountStyles.setSection}
          className="w-1/2"
        />
      </div>
    </div>
  );
};
