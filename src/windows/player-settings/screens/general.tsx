import { useAppState } from "../../../services/state";

export const Meta = () => {
  const state = useAppState();

  return <div className="p-2 pt-1 overflow-y-auto"></div>;
};
