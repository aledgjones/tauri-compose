import { TabsOption, Tabs } from "../ui/tabs";
import { actions, Tab, useAppState } from "../services/state";
import { NotFound } from "../screens/not-found";
import { Setup } from "../screens/setup";
import { Transport } from "./components/transport";

const tabs: TabsOption<Tab>[] = [
  { label: "Setup", value: Tab.Setup },
  { label: "Write", value: Tab.Write },
  { label: "Engrave", value: Tab.Engrave },
  { label: "Play", value: Tab.Play },
  { label: "Print", value: Tab.Print },
];

const map = new Map([[Tab.Setup, Setup]]);

export const App = () => {
  const state = useAppState();

  const Content = map.get(state.ui.tab) || NotFound;

  return (
    <div className="h-dvh bg-neutral-900 flex flex-col">
      <div
        data-tauri-drag-region
        className="h-[28px] flex items-center justify-center"
      >
        <p className="text-white text-sm">Solo Composer</p>
      </div>
      <div className="p-2 flex items-center justify-between">
        <div className="flex">
          <Tabs
            value={state.ui.tab}
            options={tabs}
            onChange={actions.ui.setTab}
            className="mr-2"
          />
          {/* <Select
            value={state.ui.layout}
            onChange={actions.ui.setActiveLayoutId}
            options={[
              { value: "full-score", label: "Full Score" },
              { value: "456", label: "Violin 1" },
              { value: "789", label: "Violin 2" },
            ]}
          /> */}
        </div>
        <Transport />
      </div>

      <div className="flex-1">
        <Content />
      </div>
    </div>
  );
};
