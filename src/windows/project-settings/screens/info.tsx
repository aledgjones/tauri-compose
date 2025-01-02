import { useAppState } from "../../../services/state";
import { useHumanizedTime } from "../../../services/utils";

export const Info = () => {
  const state = useAppState();
  const created = useHumanizedTime(state.score.meta.created);

  return (
    <div className="p-2 overflow-y-auto">
      <div className="mb-2">
        <p className="text-neutral-400 text-xs">Created</p>
        <p className="text-white text-sm">{created}</p>
      </div>
      <div className="mb-2">
        <p className="text-neutral-400 text-xs">Created by</p>
        <p className="text-white text-sm">aledjones.viola@gmail.com</p>
      </div>
    </div>
  );
};
