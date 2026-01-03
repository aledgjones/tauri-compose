import { TabsOption, Tabs } from "../../ui/tabs";
import { useAppState } from "../../services/state";
import { NotFound } from "../../components/not-found";
import { Setup } from "./screens/setup";
import { Tools } from "./components/tools";
import { Transport } from "./components/transport";
import { IconButton } from "../../ui/icon-button";
import { mdiTuneVertical, mdiMovieRoll } from "@mdi/js";
import { AppBar } from "../../components/app-bar";
import { Select } from "../../ui/select";
import { Tab } from "../../services/types";
import { actions } from "../../services/actions";
import { useMemo } from "react";
import { getLayoutName } from "../../services/utils";
import { Write } from "./screens/write";
import { Sequence } from "./screens/sequence";
import { PlaybackProvider } from "../../services/playback-context";

const tabs: TabsOption<Tab>[] = [
  { label: "Setup", value: Tab.Setup },
  { label: "Write", value: Tab.Write },
  { label: "Engrave", value: Tab.Engrave },
  { label: "Sequence", value: Tab.Sequence },
  { label: "Print", value: Tab.Print },
];

const screens = new Map([
  [Tab.Setup, Setup],
  [Tab.Write, Write],
  [Tab.Sequence, Sequence],
]);

export const Main = () => {
  const state = useAppState();

  const Content = screens.get(state.ui.tab) || NotFound;

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
    <PlaybackProvider appState={state}>
      <div className="h-dvh w-dvw grid grid-rows-[30px_36px_1fr] bg-transparent gap-0.5">
        <AppBar title="Solo Compose" filename={state.score.meta.title} />
        <div className="mx-0.75 flex items-center">
          <Tabs
            className="flex flex-1 justify-start"
            value={state.ui.tab}
            options={tabs}
            onChange={actions.ui.setTab}
          />

          <div className="ml-0.5 h-full bg-neutral-900 flex-1" />

          <div className="flex px-0.5">
            <Select
              variant="standard"
              className="mr-0.5 inline-block"
              value={state.ui.layout_id}
              onChange={actions.ui.setActiveLayoutId}
              options={layouts}
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
          </div>

          <div className="h-full bg-neutral-900 flex-1" />

          <Tools className="flex-1" />
        </div>

        <Content />
      </div>
    </PlaybackProvider>
  );
};
