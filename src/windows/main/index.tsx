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
import { getLayoutName, getFlowName } from "../../services/utils";
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

  const flows = useMemo(() => {
    return state.score.flows.order.map((flowId, index) => {
      const name = getFlowName(state, index, flowId);

      return {
        value: flowId,
        label: name,
      };
    });
  }, [state]);

  return (
    <div className="h-dvh w-dvw grid grid-rows-[30px_48px_1fr] bg-transparent p-0.75">
      <AppBar title="Solo Compose" />
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

          <Select
            variant="standard"
            className="mr-0.5 inline-block"
            value={state.ui.flow_id ?? state.score.flows.order[0] ?? null}
            onChange={actions.ui.setActiveFlowId}
            options={flows}
            placeholder="No flows yet..."
            disabled={flows.length === 0}
          />

          <Transport className="mr-0.5" />

          <IconButton
            className="mr-0.5 text-white"
            variant="interactive"
            onClick={() => { }}
            path={mdiTuneVertical}
            isDisabled
          />
          <IconButton
            className="text-white"
            variant="interactive"
            onClick={() => { }}
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
    </div>
  );
};
