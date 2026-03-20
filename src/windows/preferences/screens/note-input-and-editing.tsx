import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";
import { Switch } from "../../../ui/switch";

export const NoteInputAndEditing = () => {
  const state = useAppState();

  return (
    <div className="bg-neutral-900">
      <h3 className="p-2 text-sm font-medium text-white bg-neutral-800">
        Note Preview
      </h3>
      <div className="p-4 gap-2 flex flex-col">
        <Switch
          label="Preview notes while writing or editing"
          value={state.preferences.audition}
          onChange={actions.ui.preferences.setAudition}
        />
      </div>
    </div>
  );
};
