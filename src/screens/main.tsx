import { Option, Tabs } from "../components/ui/tabs";
import { actions, Tab, useAppState } from "../services/state";
import { NotFound } from "./not-found";
import { Setup } from "./setup";

const tabs: Option<Tab>[] = [
  { label: "Setup", value: Tab.Setup },
  { label: "Write", value: Tab.Write },
  { label: "Engrave", value: Tab.Engrave },
  { label: "Play", value: Tab.Play },
  { label: "Print", value: Tab.Print },
];

const map = new Map([[Tab.Setup, Setup]]);

export const Main = () => {
  const state = useAppState();

  const Content = map.get(state.ui.tab) || NotFound;

  return (
    <div className="h-dvh bg-neutral-900 flex flex-col">
      {/* header */}
      <div className="p-2 border-b-2 border-neutral-950">
        <Tabs
          value={state.ui.tab}
          options={tabs}
          onChange={actions.ui.setTab}
        />
      </div>

      {/* content */}
      <div className="flex-1">
        <Content />
      </div>
    </div>
  );
};
