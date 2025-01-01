import { TabsOption, Tabs } from "../ui/tabs";
import { actions, Tab, useAppState } from "../services/state";
import { NotFound } from "./screens/not-found";
import { Setup } from "./screens/setup";
import { Tools } from "./components/tools";
import { Transport } from "./components/transport";
import { Select } from "../ui/select";
import { IconButton } from "../ui/icon-button";
import { mdiMovieRoll, mdiTuneVertical } from "@mdi/js";

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
    <div className="h-dvh bg-neutral-950 flex flex-col">
      <div
        data-tauri-drag-region
        className="h-[28px] flex items-center justify-center"
      >
        <p className="text-white text-xs">Solo Composer</p>
      </div>
      <div className="p-2 flex items-center">
        <Tabs
          className="flex flex-1 justify-start"
          value={state.ui.tab}
          options={tabs}
          onChange={actions.ui.setTab}
        />

        <Select
          className="mr-2"
          value={state.ui.layout_id}
          onChange={actions.ui.setActiveLayoutId}
          options={[
            { value: "full-score", label: "Full Score" },
            { value: "456", label: "Violin 1" },
            { value: "789", label: "Violin 2" },
          ]}
        />

        <Transport className="mr-2" />

        <IconButton
          className="mr-1"
          variant="filled"
          onClick={() => {}}
          path={mdiTuneVertical}
          isDisabled
        />
        <IconButton
          variant="filled"
          onClick={() => {}}
          path={mdiMovieRoll}
          isDisabled
        />

        <Tools className="flex-1 justify-end" />
      </div>

      <div className="flex-1">
        <Content />
      </div>
    </div>
  );
};
