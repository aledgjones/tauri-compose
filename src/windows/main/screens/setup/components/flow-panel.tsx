import { mdiPlus } from "@mdi/js";
import { Panel } from "./panel";
import { useAppState } from "../../../../../services/state";
import { actions } from "../../../../../services/actions";
import { useEffect, useRef } from "preact/hooks";
import Sortable from "sortablejs";
import { FlowPanelItem } from "./flow-panel-item";
import { AppState, SelectionType } from "../../../../../services/types";

import "./flow-panel.css";

const isAssigned = (state: AppState, flowId: number) => {
  const selection = state.ui.setup.selection;

  if (selection?.selection_type === SelectionType.Player) {
    return state.score.flows.by_id[flowId].player_ids.includes(selection.id);
  }

  return false;
};

const onAddFlowPress = async () => {
  await actions.score.flow.create();
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

        // await actions.score.player.move(e.oldIndex, e.newIndex);
      },
    });

    return () => {
      sortable.destroy();
    };
  }, []);

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
        {state.score.flows.order.map((flowId) => {
          const flow = state.score.flows.by_id[flowId];
          const selection = state.ui.setup.selection;
          const isAssignable = assignableSet.has(selection?.selection_type);

          return (
            <FlowPanelItem
              key={flowId}
              flow={flow}
              isSelected={selection?.id === flowId}
              isAssignable={isAssignable}
              isAssigned={isAssigned(state, flowId)}
            />
          );
        })}
      </div>
    </Panel>
  );
};

const assignableSet = new Set<SelectionType | undefined>([
  SelectionType.Player,
  SelectionType.Layout,
]);
