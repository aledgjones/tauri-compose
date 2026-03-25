import { TabsOption, Tabs } from "../../ui/tabs";
import { useAppState } from "../../services/state";
import { NotFound } from "../../components/not-found";
import { Setup } from "./screens/setup";
import { Tools } from "./components/tools";
import { Transport } from "./components/transport";
import { IconButton } from "../../ui/buttons/icon-button";
import { mdiTuneVertical, mdiMovieRoll } from "@mdi/js";
import { AppBar } from "../../components/app-bar";
import { Select } from "../../ui/select";
import { Tab } from "../../services/types";
import { actions } from "../../services/actions";
import { useMemo } from "react";
import { getLayoutName } from "../../services/utils";
import { Write } from "./screens/write";
import { Sequence } from "./screens/sequence";

const tabs: TabsOption<Tab>[] = [
  { label: "Setup", value: Tab.Setup },
  { label: "Write", value: Tab.Write },
  { label: "Engrave", value: Tab.Engrave },
  { label: "Sequence", value: Tab.Sequence },
  { label: "Print", value: Tab.Print },
];

export const Main = () => {
  const state = useAppState();

  const layouts = useMemo(() => {
    return state.score.layouts.order.map((layoutId) => {
      const name = getLayoutName(state, layoutId);

      return {
        value: layoutId,
        label: name,
      };
    });
  }, [state]);

  return (
    <div className="relative h-dvh w-dvw grid grid-rows-[30px_48px_1fr] bg-transparent p-0.75">
      <AppBar title="Solo Compose" showFilename />
      <div className="flex items-center p-2 bg-neutral-900">
        <div className="flex flex-1">
          <Tabs
            className="flex justify-start w-96.75"
            value={state.ui.tab}
            options={tabs}
            onChange={actions.ui.setTab}
          />
          <div className="bg-neutral-900 flex-1 ml-0.5" />
        </div>

        <div className="flex flex-1 justify-center">
          <div className="bg-neutral-900 flex-1 mr-0.5" />

          <Select
            variant="standard"
            className="mr-0.5 inline-block"
            value={state.ui.layout_id ?? state.score.layouts.order[0] ?? null}
            onChange={actions.ui.setActiveLayoutId}
            options={layouts}
            placeholder="No layouts yet..."
            disabled={layouts.length === 0}
          />

          <Transport className="mr-0.5" />

          <IconButton
            className="mr-0.5 text-white"
            variant="interactive"
            onClick={() => {}}
            path={mdiTuneVertical}
            isDisabled
          />
          <IconButton
            className="text-white"
            variant="interactive"
            onClick={() => {}}
            path={mdiMovieRoll}
            isDisabled
          />

          <div className="bg-neutral-900 flex-1 ml-0.5" />
        </div>

        <div className="flex flex-1">
          <div className="bg-neutral-900 flex-1" />
          <Tools />
        </div>
      </div>

      {state.ui.tab === Tab.Setup && <Setup />}
      {state.ui.tab === Tab.Write && <Write />}
      {state.ui.tab === Tab.Sequence && <Sequence />}
      {[Tab.Engrave, Tab.Print].includes(state.ui.tab) && <NotFound />}

      {state.is_project_loading && (
        <div className="absolute inset-0 z-100 flex items-center justify-center rounded-[14px] bg-neutral-950/55 backdrop-blur-[2px]">
          <div className="flex items-center gap-3 rounded-full border border-amber-400/25 bg-neutral-900/95 px-4 py-2 text-sm font-medium text-neutral-100 shadow-lg shadow-black/30">
            <span className="h-2.5 w-2.5 animate-pulse rounded-full bg-amber-300" />
            Loading project...
          </div>
        </div>
      )}
    </div>
  );
};
