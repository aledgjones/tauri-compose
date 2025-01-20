import { mdiPlus } from "@mdi/js";
import { Panel } from "./panel";
import { useAppState } from "../../../../../services/state";
import { actions } from "../../../../../services/actions";
import { useEffect, useRef } from "react";
import Sortable from "sortablejs";
import { FlowPanelItem } from "./flow-panel-item";
import { SelectionType } from "../../../../../services/types";

import "./flow-panel.css";

const onAddFlowPress = async () => {
  const flowId = await actions.score.flow.create();
  await actions.ui.setup.select(SelectionType.Flow, flowId);
};

export const FlowPanel = () => {
  const state = useAppState();
  const ref = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!ref.current) {
      return;
    }

    const sortable = Sortable.create(ref.current, {
      group: "flows",
      animation: 150,
      forceFallback: true,
      ghostClass: "ghost-flow",
      delay: 150,
      onSort: async (e) => {
        if (e.oldIndex === undefined || e.newIndex === undefined) {
          return;
        }

        await actions.score.flow.move(e.oldIndex, e.newIndex);
      },
    });

    return () => {
      sortable.destroy();
    };
  }, [state.score.flows.order]);

  return (
    <Panel
      title="Flows"
      icons={[
        {
          key: "plus",
          path: mdiPlus,
          onClick: onAddFlowPress,
        },
      ]}
    >
      <div ref={ref} className="flex gap-2 px-2 pb-2 flex-1 overflow-auto">
        {state.score.flows.order.map((flowId, index) => {
          const flow = state.score.flows.by_id[flowId];

          return <FlowPanelItem key={flowId} flow={flow} index={index} />;
        })}
      </div>
    </Panel>
  );
};
