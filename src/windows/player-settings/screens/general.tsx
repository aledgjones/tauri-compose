import { actions, AutoCountStyle, useAppState } from "../../../services/state";
import { Select, SelectOption } from "../../../ui/select";

const autoCountOptions: SelectOption<AutoCountStyle>[] = [
  { label: "Arabic (1, 2, 3...)", value: AutoCountStyle.Arabic },
  { label: "Roman (I, II, III...)", value: AutoCountStyle.Roman },
];

export const Meta = () => {
  const state = useAppState();

  return (
    <div className="p-2 pt-1 overflow-y-auto min-h-full">
      <Select
        variant="filled"
        label="Numbering style for single players"
        value={state.score.config.auto_count_styles.solo}
        options={autoCountOptions}
        onChange={actions.score.config.autoCountStyles.setSolo}
      />
      <Select
        variant="filled"
        label="Numbering style for section players"
        value={state.score.config.auto_count_styles.section}
        options={autoCountOptions}
        onChange={actions.score.config.autoCountStyles.setSection}
      />
    </div>
  );
};
