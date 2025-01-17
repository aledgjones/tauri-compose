import { mdiFileCogOutline, mdiPlus } from "@mdi/js";
import { Panel } from "./panel";
import { useAppState } from "../../../../../services/state";
import { useEffect, useRef } from "preact/hooks";
import Sortable from "sortablejs";
import { LayoutPanelItem } from "./layout-panel-item";
import { actions } from "../../../../../services/actions";

import "./layout-panel.css";

export const LayoutPlanel = () => {
  const state = useAppState();
  const ref = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!ref.current) {
      return;
    }

    const sortable = Sortable.create(ref.current, {
      group: "layouts",
      animation: 150,
      forceFallback: true,
      ghostClass: "ghost-layout",
      delay: 150,
      onSort: async (e) => {
        if (e.oldIndex === undefined || e.newIndex === undefined) {
          return;
        }

        await actions.score.layout.move(e.oldIndex, e.newIndex);
      },
    });

    return () => {
      sortable.destroy();
    };
  }, [state.score.layouts.order]);

  return (
    <Panel
      className="row-span-2"
      title="Layouts"
      icons={[
        {
          key: "cog",
          path: mdiFileCogOutline,
          onClick: () => {},
        },
        {
          key: "plus",
          path: mdiPlus,
          onClick: () => {},
        },
      ]}
    >
      <div ref={ref} className="flex flex-col gap-2 px-2 flex-1 overflow-auto">
        {state.score.layouts.order.map((layoutId) => {
          const layout = state.score.layouts.by_id[layoutId];

          return <LayoutPanelItem key={layoutId} layout={layout} />;
        })}
      </div>
    </Panel>
  );
};
