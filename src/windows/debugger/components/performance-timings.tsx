import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import classNames from "classnames";
import { useAppState } from "../../../services/state";

type FlowPreparationDetails = {
  render_mode: string;
  layout_id: number;
  active_flow_id: number;
  active_flow_visible_in_layout: boolean;
  prepared_flow_ids: number[];
  prepared_flow_count: number;
  prepared_flow_scope: string;
  composed_flow_ids: number[];
  composed_flow_count: number;
  composed_flow_scope: string;
  recalculated_flow_ids: number[];
  recalculated_flow_count: number;
  recalculation_scope: string;
};

type RenderSurfacePageBox = {
  page_index: number;
  x_px: number;
  width_px: number;
};

type RenderSurfaceDetails = {
  surface_width_px: number;
  surface_height_px: number;
  viewport_width_px: number;
  viewport_height_px: number;
  page_count: number;
  inter_flow_gap_px: number;
  page_boxes: RenderSurfacePageBox[];
};

type FlowVisualState = {
  flow_id: number;
  flow_number: number;
  flow_title: string;
  is_active: boolean;
  is_in_layout: boolean;
  is_prepared: boolean;
  is_composed: boolean;
  is_recalculated: boolean;
};

const formatRenderMode = (mode: string) => {
  if (mode === "shared_surface") {
    return "Unified shared surface";
  }

  return mode.split("_").join(" ");
};

const formatScopeLabel = (scope: string) => {
  if (scope === "single_flow") {
    return "Single flow";
  }

  if (scope === "multiple_flows") {
    return "Multiple flows";
  }

  if (scope === "no_visible_flows") {
    return "No visible flows";
  }

  return scope.split("_").join(" ");
};

const getPagePreviewWidth = (
  pageBox: RenderSurfacePageBox,
  surfaceWidthPx: number,
) => {
  if (surfaceWidthPx <= 0) {
    return 48;
  }

  return Math.max(48, (pageBox.width_px / surfaceWidthPx) * 160);
};

const collectFlowVisualStates = (
  allFlowIds: number[],
  flowTitlesById: Record<number, string>,
  layoutFlowIds: number[],
  details: FlowPreparationDetails,
): FlowVisualState[] => {
  return allFlowIds.map((flow_id) => ({
    flow_id,
    flow_number: allFlowIds.indexOf(flow_id) + 1,
    flow_title: flowTitlesById[flow_id] || `Flow ${flow_id}`,
    is_active: flow_id === details.active_flow_id,
    is_in_layout: layoutFlowIds.includes(flow_id),
    is_prepared: details.prepared_flow_ids.includes(flow_id),
    is_composed: details.composed_flow_ids.includes(flow_id),
    is_recalculated: details.recalculated_flow_ids.includes(flow_id),
  }));
};

const getPassSummary = (
  details: FlowPreparationDetails | null,
  status: "idle" | "ready" | "cleared",
) => {
  if (status === "cleared" || !details) {
    return {
      title: "No active render",
      description: "Waiting for a render plan or composed surface.",
      tone_class: "border-neutral-800 bg-neutral-950/70 text-neutral-300",
    };
  }

  if (details.recalculation_scope === "single_flow") {
    return {
      title: "Targeted update",
      description: "One visible flow changed in this pass.",
      tone_class: "border-emerald-800/70 bg-emerald-950/30 text-emerald-100",
    };
  }

  if (details.recalculation_scope === "multiple_flows") {
    return {
      title: "Shared update",
      description: "Multiple visible flows changed in this pass.",
      tone_class: "border-amber-800/70 bg-amber-950/25 text-amber-100",
    };
  }

  return {
    title: "Empty layout",
    description: "The layout has no visible flows right now.",
    tone_class: "border-sky-800/70 bg-sky-950/25 text-sky-100",
  };
};

export const PerformanceTimings = () => {
  const state = useAppState();
  const [renderTimings, setRenderTimings] = useState<Record<string, number>>(
    {},
  );
  const [flowPreparationDetails, setFlowPreparationDetails] =
    useState<FlowPreparationDetails | null>(null);
  const [flowPreparationStatus, setFlowPreparationStatus] = useState<
    "idle" | "ready" | "cleared"
  >("idle");
  const [renderSurfaceDetails, setRenderSurfaceDetails] =
    useState<RenderSurfaceDetails | null>(null);
  const [layoutCollapsed, setLayoutCollapsed] = useState(true);
  const [renderCollapsed, setRenderCollapsed] = useState(true);
  const [gpuCollapsed, setGpuCollapsed] = useState(true);
  const passSummary = getPassSummary(
    flowPreparationDetails,
    flowPreparationStatus,
  );
  const flowTitlesById = Object.fromEntries(
    Object.entries(state.score.flows.by_id).map(([flowId, flow]) => [
      Number(flowId),
      flow.title,
    ]),
  );
  const layoutFlowIds = flowPreparationDetails
    ? state.score.layouts.by_id[flowPreparationDetails.layout_id]?.flow_ids ||
      []
    : [];
  const flowVisualStates = flowPreparationDetails
    ? collectFlowVisualStates(
        state.score.flows.order.filter(
          (flowId) => state.score.flows.by_id[flowId],
        ),
        flowTitlesById,
        layoutFlowIds,
        flowPreparationDetails,
      )
    : [];

  useEffect(() => {
    const timingKeys = [
      "render-timing-flow-preparation",
      "render-timing-vertical",
      "render-timing-layout",
      "render-timing-structure",
      "render-timing-notation",
      "render-timing-elements",
      "render-timing-spacing",
      "render-timing-stems",
      "render-timing-setup",
      "render-timing-barlines",
      "render-timing-stave",
      "native-render-timing-setup",
      "native-render-timing-draw",
      "native-render-timing-flush",
    ];

    const unlisteners = timingKeys.map((eventName) =>
      listen(eventName, (event) => {
        const duration = event.payload as number;
        setRenderTimings((prev) => ({ ...prev, [eventName]: duration }));
      }),
    );

    const detailsUnlistener = listen(
      "render-timing-flow-preparation-details",
      (event) => {
        const payload =
          (event.payload as FlowPreparationDetails | null) ?? null;
        setFlowPreparationDetails(payload);
        setFlowPreparationStatus(payload ? "ready" : "cleared");
      },
    );

    const surfaceDetailsUnlistener = listen(
      "render-timing-surface-details",
      (event) => {
        setRenderSurfaceDetails(
          (event.payload as RenderSurfaceDetails | null) ?? null,
        );
      },
    );

    return () => {
      unlisteners.forEach((unlisten) => {
        unlisten.then((fn) => fn());
      });
      detailsUnlistener.then((fn) => fn());
      surfaceDetailsUnlistener.then((fn) => fn());
    };
  }, []);

  return (
    <div className="mx-2">
      {flowPreparationStatus !== "idle" && (
        <div className="mb-3">
          <div className="px-2 pb-1 text-[11px] font-semibold uppercase tracking-wide text-neutral-500">
            Render Scope
          </div>
          <div
            className={classNames(
              "rounded border px-3 py-2 text-xs",
              passSummary.tone_class,
            )}
          >
            {flowPreparationDetails ? (
              <>
                <div className="flex flex-wrap items-end justify-between gap-2">
                  <div>
                    <div className="text-sm font-semibold text-neutral-100">
                      {passSummary.title}
                    </div>
                    <div className="mt-0.5 text-[11px] text-neutral-400">
                      {passSummary.description}
                    </div>
                  </div>
                  <div className="text-[11px] text-neutral-500">
                    {formatRenderMode(flowPreparationDetails.render_mode)}
                  </div>
                </div>
                <div className="mt-1 text-[11px] text-neutral-400">
                  Layout {flowPreparationDetails.layout_id} · Active flow{" "}
                  {flowPreparationDetails.active_flow_id} · Active flow visible:{" "}
                  {flowPreparationDetails.active_flow_visible_in_layout
                    ? "yes"
                    : "no"}
                </div>
                <div className="mt-3">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div className="text-[11px] uppercase tracking-wide text-neutral-500">
                      Flow cards
                    </div>
                    <div className="flex flex-wrap gap-2 text-[11px] text-neutral-500">
                      <span className="rounded-full border border-sky-700/70 bg-sky-500/10 px-2 py-0.5">
                        Planned {flowPreparationDetails.prepared_flow_count} ·{" "}
                        {formatScopeLabel(
                          flowPreparationDetails.prepared_flow_scope,
                        )}
                      </span>
                      <span className="rounded-full border border-amber-700/70 bg-amber-500/10 px-2 py-0.5">
                        Recalculated{" "}
                        {flowPreparationDetails.recalculated_flow_count} ·{" "}
                        {formatScopeLabel(
                          flowPreparationDetails.recalculation_scope,
                        )}
                      </span>
                      <span className="rounded-full border border-emerald-700/70 bg-emerald-500/10 px-2 py-0.5">
                        Composed {flowPreparationDetails.composed_flow_count} ·{" "}
                        {formatScopeLabel(
                          flowPreparationDetails.composed_flow_scope,
                        )}
                      </span>
                    </div>
                  </div>
                  <div className="mt-2 grid grid-cols-3 gap-2">
                    {flowVisualStates.length > 0 ? (
                      flowVisualStates.map((flow) => {
                        const hasStages =
                          flow.is_prepared ||
                          flow.is_composed ||
                          flow.is_recalculated;

                        return (
                          <div
                            key={flow.flow_id}
                            className={classNames(
                              "relative min-w-0 rounded-lg border border-neutral-800 bg-neutral-950/70 px-3 py-3 text-left",
                              flow.is_active && "ring-2 ring-sky-400/80",
                            )}
                          >
                            {flow.is_active && (
                              <span className="absolute right-2 top-2 rounded-full border border-sky-300/90 px-1.5 py-0.5 text-[10px] uppercase tracking-wide text-sky-200">
                                Active
                              </span>
                            )}
                            <div className="text-[10px] uppercase tracking-wide text-neutral-500">
                              Flow {flow.flow_number}
                            </div>
                            <div className="mt-1 text-[11px] text-neutral-400">
                              {flow.flow_title}
                            </div>
                            <div className="mt-3 flex flex-wrap gap-1.5">
                              {flow.is_prepared && (
                                <span className="rounded-full border border-sky-700/70 bg-sky-500/10 px-2 py-0.5 text-[11px] text-sky-100">
                                  Planned
                                </span>
                              )}
                              {flow.is_recalculated && (
                                <span className="rounded-full border border-amber-700/70 bg-amber-500/10 px-2 py-0.5 text-[11px] text-amber-100">
                                  Recalculated
                                </span>
                              )}
                              {flow.is_composed && (
                                <span className="rounded-full border border-emerald-700/70 bg-emerald-500/10 px-2 py-0.5 text-[11px] text-emerald-100">
                                  Composed
                                </span>
                              )}
                              {!hasStages && (
                                <span className="rounded-full border border-dashed border-neutral-700 px-2 py-0.5 text-[11px] text-neutral-400">
                                  Not planned
                                </span>
                              )}
                            </div>
                          </div>
                        );
                      })
                    ) : (
                      <div className="rounded border border-dashed border-neutral-800 px-3 py-2 text-[11px] text-neutral-500">
                        No visible flows in this pass.
                      </div>
                    )}
                  </div>
                </div>
                {renderSurfaceDetails && (
                  <div className="mt-3">
                    <div className="text-[11px] text-neutral-400">
                      Canvas {renderSurfaceDetails.surface_width_px.toFixed(1)}
                      px x {renderSurfaceDetails.surface_height_px.toFixed(1)}px
                      · Viewport{" "}
                      {renderSurfaceDetails.viewport_width_px.toFixed(1)}px x{" "}
                      {renderSurfaceDetails.viewport_height_px.toFixed(1)}px
                      {renderSurfaceDetails.page_count > 1 && (
                        <>
                          {" "}
                          · Gap{" "}
                          {renderSurfaceDetails.inter_flow_gap_px.toFixed(1)}px
                        </>
                      )}
                    </div>
                    {renderSurfaceDetails.page_boxes.length > 0 && (
                      <div className="mt-3">
                        <div className="text-[11px] uppercase tracking-wide text-neutral-500">
                          Surface map
                        </div>
                        <div className="mt-2 flex flex-wrap items-end gap-2">
                          {renderSurfaceDetails.page_boxes.map((pageBox) => (
                            <div
                              key={pageBox.page_index}
                              className="rounded-lg border border-neutral-800 bg-neutral-950/60 px-2 py-2"
                            >
                              <div className="text-[10px] uppercase tracking-wide text-neutral-500">
                                Page {pageBox.page_index + 1}
                              </div>
                              <div
                                className="mt-2 h-3 rounded-sm bg-neutral-300/80"
                                style={{
                                  width: `${getPagePreviewWidth(pageBox, renderSurfaceDetails.surface_width_px)}px`,
                                }}
                              />
                              <div className="mt-2 flex items-center justify-between gap-3 text-[10px] text-neutral-500">
                                <span>{pageBox.x_px.toFixed(1)}px</span>
                                <span>{pageBox.width_px.toFixed(1)}px</span>
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </>
            ) : (
              <>
                <div className="text-sm font-semibold text-neutral-100">
                  {passSummary.title}
                </div>
                <div className="mt-1 text-[11px] text-neutral-400">
                  {passSummary.description}
                </div>
              </>
            )}
          </div>
        </div>
      )}

      <button
        className="text-sm font-bold text-neutral-200 bg-neutral-900 p-2 flex justify-between cursor-pointer hover:bg-neutral-800 w-full text-left border-none"
        onClick={() => setLayoutCollapsed(!layoutCollapsed)}
        type="button"
        aria-expanded={!layoutCollapsed}
      >
        <span>🎼 Layout Phase (CPU)</span>
        <span
          className={classNames("font-mono text-sm", {
            "text-red-400":
              ((renderTimings["render-timing-vertical"] || 0) +
                (renderTimings["render-timing-flow-preparation"] || 0) +
                (renderTimings["render-timing-layout"] || 0) +
                (renderTimings["render-timing-structure"] || 0) +
                (renderTimings["render-timing-notation"] || 0) +
                (renderTimings["render-timing-elements"] || 0) +
                (renderTimings["render-timing-spacing"] || 0) +
                (renderTimings["render-timing-stems"] || 0)) /
                1000 >
              50,
            "text-yellow-400":
              ((renderTimings["render-timing-vertical"] || 0) +
                (renderTimings["render-timing-flow-preparation"] || 0) +
                (renderTimings["render-timing-layout"] || 0) +
                (renderTimings["render-timing-structure"] || 0) +
                (renderTimings["render-timing-notation"] || 0) +
                (renderTimings["render-timing-elements"] || 0) +
                (renderTimings["render-timing-spacing"] || 0) +
                (renderTimings["render-timing-stems"] || 0)) /
                1000 >
                20 &&
              ((renderTimings["render-timing-vertical"] || 0) +
                (renderTimings["render-timing-flow-preparation"] || 0) +
                (renderTimings["render-timing-layout"] || 0) +
                (renderTimings["render-timing-structure"] || 0) +
                (renderTimings["render-timing-notation"] || 0) +
                (renderTimings["render-timing-elements"] || 0) +
                (renderTimings["render-timing-spacing"] || 0) +
                (renderTimings["render-timing-stems"] || 0)) /
                1000 <=
                50,
            "text-green-400":
              ((renderTimings["render-timing-vertical"] || 0) +
                (renderTimings["render-timing-flow-preparation"] || 0) +
                (renderTimings["render-timing-layout"] || 0) +
                (renderTimings["render-timing-structure"] || 0) +
                (renderTimings["render-timing-notation"] || 0) +
                (renderTimings["render-timing-elements"] || 0) +
                (renderTimings["render-timing-spacing"] || 0) +
                (renderTimings["render-timing-stems"] || 0)) /
                1000 <=
              20,
          })}
        >
          {(
            (renderTimings["render-timing-vertical"] || 0) +
            (renderTimings["render-timing-flow-preparation"] || 0) +
            (renderTimings["render-timing-layout"] || 0) +
            (renderTimings["render-timing-structure"] || 0) +
            (renderTimings["render-timing-notation"] || 0) +
            (renderTimings["render-timing-elements"] || 0) +
            (renderTimings["render-timing-spacing"] || 0) +
            (renderTimings["render-timing-stems"] || 0)
          ).toLocaleString()}
          μs
        </span>
      </button>
      {!layoutCollapsed && (
        <div className="px-2 py-1.5">
          <div className="flex items-center">
            <span className="text-xs text-neutral-400 flex-1">
              Flow-aware preparation
            </span>
            <span
              className={classNames("text-xs font-mono text-right", {
                "text-red-400":
                  (renderTimings["render-timing-flow-preparation"] || 0) /
                    1000 >
                  50,
                "text-yellow-400":
                  (renderTimings["render-timing-flow-preparation"] || 0) /
                    1000 >
                    20 &&
                  (renderTimings["render-timing-flow-preparation"] || 0) /
                    1000 <=
                    50,
                "text-green-400":
                  (renderTimings["render-timing-flow-preparation"] || 0) /
                    1000 <=
                  20,
              })}
            >
              {(
                renderTimings["render-timing-flow-preparation"] || 0
              ).toLocaleString()}
              μs
            </span>
          </div>
          <div className="flex items-center">
            <span className="text-xs text-neutral-400 flex-1">
              Vertical spacing & staves
            </span>
            <span
              className={classNames("text-xs font-mono text-right", {
                "text-red-400":
                  (renderTimings["render-timing-vertical"] || 0) / 1000 > 50,
                "text-yellow-400":
                  (renderTimings["render-timing-vertical"] || 0) / 1000 > 20 &&
                  (renderTimings["render-timing-vertical"] || 0) / 1000 <= 50,
                "text-green-400":
                  (renderTimings["render-timing-vertical"] || 0) / 1000 <= 20,
              })}
            >
              {(renderTimings["render-timing-vertical"] || 0).toLocaleString()}
              μs
            </span>
          </div>
          <div className="flex items-center">
            <span className="text-xs text-neutral-400 flex-1">
              Layout calculations
            </span>
            <span
              className={classNames("text-xs font-mono text-right", {
                "text-red-400":
                  (renderTimings["render-timing-layout"] || 0) / 1000 > 50,
                "text-yellow-400":
                  (renderTimings["render-timing-layout"] || 0) / 1000 > 20 &&
                  (renderTimings["render-timing-layout"] || 0) / 1000 <= 50,
                "text-green-400":
                  (renderTimings["render-timing-layout"] || 0) / 1000 <= 20,
              })}
            >
              {(renderTimings["render-timing-layout"] || 0).toLocaleString()}
              μs
            </span>
          </div>
          <div className="flex items-center">
            <span className="text-xs text-neutral-400 flex-1">
              Score structure parsing
            </span>
            <span
              className={classNames("text-xs font-mono text-right", {
                "text-red-400":
                  (renderTimings["render-timing-structure"] || 0) / 1000 > 50,
                "text-yellow-400":
                  (renderTimings["render-timing-structure"] || 0) / 1000 > 20 &&
                  (renderTimings["render-timing-structure"] || 0) / 1000 <= 50,
                "text-green-400":
                  (renderTimings["render-timing-structure"] || 0) / 1000 <= 20,
              })}
            >
              {(renderTimings["render-timing-structure"] || 0).toLocaleString()}
              μs
            </span>
          </div>
          <div className="flex items-center">
            <span className="text-xs text-neutral-400 flex-1">
              Written durations analysis
            </span>
            <span
              className={classNames("text-xs font-mono text-right", {
                "text-red-400":
                  (renderTimings["render-timing-notation"] || 0) / 1000 > 50,
                "text-yellow-400":
                  (renderTimings["render-timing-notation"] || 0) / 1000 > 20 &&
                  (renderTimings["render-timing-notation"] || 0) / 1000 <= 50,
                "text-green-400":
                  (renderTimings["render-timing-notation"] || 0) / 1000 <= 20,
              })}
            >
              {(renderTimings["render-timing-notation"] || 0).toLocaleString()}
              μs
            </span>
          </div>
          <div className="flex items-center">
            <span className="text-xs text-neutral-400 flex-1">
              Musical elements analysis
            </span>
            <span
              className={classNames("text-xs font-mono text-right", {
                "text-red-400":
                  (renderTimings["render-timing-elements"] || 0) / 1000 > 50,
                "text-yellow-400":
                  (renderTimings["render-timing-elements"] || 0) / 1000 > 20 &&
                  (renderTimings["render-timing-elements"] || 0) / 1000 <= 50,
                "text-green-400":
                  (renderTimings["render-timing-elements"] || 0) / 1000 <= 20,
              })}
            >
              {(renderTimings["render-timing-elements"] || 0).toLocaleString()}
              μs
            </span>
          </div>
          <div className="flex items-center">
            <span className="text-xs text-neutral-400 flex-1">
              Horizontal spacing
            </span>
            <span
              className={classNames("text-xs font-mono text-right", {
                "text-red-400":
                  (renderTimings["render-timing-spacing"] || 0) / 1000 > 50,
                "text-yellow-400":
                  (renderTimings["render-timing-spacing"] || 0) / 1000 > 20 &&
                  (renderTimings["render-timing-spacing"] || 0) / 1000 <= 50,
                "text-green-400":
                  (renderTimings["render-timing-spacing"] || 0) / 1000 <= 20,
              })}
            >
              {(renderTimings["render-timing-spacing"] || 0).toLocaleString()}
              μs
            </span>
          </div>
          <div className="flex items-center">
            <span className="text-xs text-neutral-400 flex-1">
              Stem length calculations
            </span>
            <span
              className={classNames("text-xs font-mono text-right", {
                "text-red-400":
                  (renderTimings["render-timing-stems"] || 0) / 1000 > 50,
                "text-yellow-400":
                  (renderTimings["render-timing-stems"] || 0) / 1000 > 20 &&
                  (renderTimings["render-timing-stems"] || 0) / 1000 <= 50,
                "text-green-400":
                  (renderTimings["render-timing-stems"] || 0) / 1000 <= 20,
              })}
            >
              {(renderTimings["render-timing-stems"] || 0).toLocaleString()}μs
            </span>
          </div>
        </div>
      )}

      <button
        className="text-sm font-bold text-neutral-200 bg-neutral-900 p-2 flex justify-between cursor-pointer hover:bg-neutral-800 w-full text-left border-none"
        onClick={() => setRenderCollapsed(!renderCollapsed)}
        type="button"
        aria-expanded={!renderCollapsed}
      >
        <span>✏️ Render Phase (CPU)</span>
        <span
          className={classNames("font-mono text-sm", {
            "text-red-400":
              ((renderTimings["render-timing-setup"] || 0) +
                (renderTimings["render-timing-barlines"] || 0) +
                (renderTimings["render-timing-stave"] || 0)) /
                1000 >
              50,
            "text-yellow-400":
              ((renderTimings["render-timing-setup"] || 0) +
                (renderTimings["render-timing-barlines"] || 0) +
                (renderTimings["render-timing-stave"] || 0)) /
                1000 >
                20 &&
              ((renderTimings["render-timing-setup"] || 0) +
                (renderTimings["render-timing-barlines"] || 0) +
                (renderTimings["render-timing-stave"] || 0)) /
                1000 <=
                50,
            "text-green-400":
              ((renderTimings["render-timing-setup"] || 0) +
                (renderTimings["render-timing-barlines"] || 0) +
                (renderTimings["render-timing-stave"] || 0)) /
                1000 <=
              20,
          })}
        >
          {(
            (renderTimings["render-timing-setup"] || 0) +
            (renderTimings["render-timing-barlines"] || 0) +
            (renderTimings["render-timing-stave"] || 0)
          ).toLocaleString()}
          μs
        </span>
      </button>
      {!renderCollapsed && (
        <div className="px-2 py-1.5">
          <div className="flex items-center">
            <span className="text-xs text-neutral-400 flex-1">
              Basic drawing setup
            </span>
            <span
              className={classNames("text-xs font-mono text-right", {
                "text-red-400":
                  (renderTimings["render-timing-setup"] || 0) / 1000 > 50,
                "text-yellow-400":
                  (renderTimings["render-timing-setup"] || 0) / 1000 > 20 &&
                  (renderTimings["render-timing-setup"] || 0) / 1000 <= 50,
                "text-green-400":
                  (renderTimings["render-timing-setup"] || 0) / 1000 <= 20,
              })}
            >
              {(renderTimings["render-timing-setup"] || 0).toLocaleString()}μs
            </span>
          </div>
          <div className="flex items-center">
            <span className="text-xs text-neutral-400 flex-1">
              Barline rendering
            </span>
            <span
              className={classNames("text-xs font-mono text-right", {
                "text-red-400":
                  (renderTimings["render-timing-barlines"] || 0) / 1000 > 50,
                "text-yellow-400":
                  (renderTimings["render-timing-barlines"] || 0) / 1000 > 20 &&
                  (renderTimings["render-timing-barlines"] || 0) / 1000 <= 50,
                "text-green-400":
                  (renderTimings["render-timing-barlines"] || 0) / 1000 <= 20,
              })}
            >
              {(renderTimings["render-timing-barlines"] || 0).toLocaleString()}
              μs
            </span>
          </div>
          <div className="flex items-center">
            <span className="text-xs text-neutral-400 flex-1">
              Stave contents rendering
            </span>
            <span
              className={classNames("text-xs font-mono text-right", {
                "text-red-400":
                  (renderTimings["render-timing-stave"] || 0) / 1000 > 50,
                "text-yellow-400":
                  (renderTimings["render-timing-stave"] || 0) / 1000 > 20 &&
                  (renderTimings["render-timing-stave"] || 0) / 1000 <= 50,
                "text-green-400":
                  (renderTimings["render-timing-stave"] || 0) / 1000 <= 20,
              })}
            >
              {(renderTimings["render-timing-stave"] || 0).toLocaleString()}μs
            </span>
          </div>
        </div>
      )}

      <button
        className="text-sm font-bold text-neutral-200 bg-neutral-900 p-2 flex justify-between cursor-pointer hover:bg-neutral-800 w-full text-left border-none"
        onClick={() => setGpuCollapsed(!gpuCollapsed)}
        type="button"
        aria-expanded={!gpuCollapsed}
      >
        <span>🚀 GPU Phase (Metal/Skia)</span>
        <span
          className={classNames("font-mono text-sm", {
            "text-red-400":
              ((renderTimings["native-render-timing-setup"] || 0) +
                (renderTimings["native-render-timing-draw"] || 0) +
                (renderTimings["native-render-timing-flush"] || 0)) /
                1000 >
              50,
            "text-yellow-400":
              ((renderTimings["native-render-timing-setup"] || 0) +
                (renderTimings["native-render-timing-draw"] || 0) +
                (renderTimings["native-render-timing-flush"] || 0)) /
                1000 >
                20 &&
              ((renderTimings["native-render-timing-setup"] || 0) +
                (renderTimings["native-render-timing-draw"] || 0) +
                (renderTimings["native-render-timing-flush"] || 0)) /
                1000 <=
                50,
            "text-green-400":
              ((renderTimings["native-render-timing-setup"] || 0) +
                (renderTimings["native-render-timing-draw"] || 0) +
                (renderTimings["native-render-timing-flush"] || 0)) /
                1000 <=
              20,
          })}
        >
          {(
            (renderTimings["native-render-timing-setup"] || 0) +
            (renderTimings["native-render-timing-draw"] || 0) +
            (renderTimings["native-render-timing-flush"] || 0)
          ).toLocaleString()}
          μs
        </span>
      </button>
      {!gpuCollapsed && (
        <div className="px-2 py-1.5">
          <div className="flex items-center">
            <span className="text-xs text-neutral-400 flex-1">
              GPU surface & canvas setup
            </span>
            <span
              className={classNames("text-xs font-mono text-right", {
                "text-red-400":
                  (renderTimings["native-render-timing-setup"] || 0) / 1000 >
                  50,
                "text-yellow-400":
                  (renderTimings["native-render-timing-setup"] || 0) / 1000 >
                    20 &&
                  (renderTimings["native-render-timing-setup"] || 0) / 1000 <=
                    50,
                "text-green-400":
                  (renderTimings["native-render-timing-setup"] || 0) / 1000 <=
                  20,
              })}
            >
              {(
                renderTimings["native-render-timing-setup"] || 0
              ).toLocaleString()}
              μs
            </span>
          </div>
          <div className="flex items-center">
            <span className="text-xs text-neutral-400 flex-1">
              GPU drawing execution
            </span>
            <span
              className={classNames("text-xs font-mono text-right", {
                "text-red-400":
                  (renderTimings["native-render-timing-draw"] || 0) / 1000 > 50,
                "text-yellow-400":
                  (renderTimings["native-render-timing-draw"] || 0) / 1000 >
                    20 &&
                  (renderTimings["native-render-timing-draw"] || 0) / 1000 <=
                    50,
                "text-green-400":
                  (renderTimings["native-render-timing-draw"] || 0) / 1000 <=
                  20,
              })}
            >
              {(
                renderTimings["native-render-timing-draw"] || 0
              ).toLocaleString()}
              μs
            </span>
          </div>
          <div className="flex items-center">
            <span className="text-xs text-neutral-400 flex-1">
              GPU flush & Metal commands
            </span>
            <span
              className={classNames("text-xs font-mono text-right", {
                "text-red-400":
                  (renderTimings["native-render-timing-flush"] || 0) / 1000 >
                  50,
                "text-yellow-400":
                  (renderTimings["native-render-timing-flush"] || 0) / 1000 >
                    20 &&
                  (renderTimings["native-render-timing-flush"] || 0) / 1000 <=
                    50,
                "text-green-400":
                  (renderTimings["native-render-timing-flush"] || 0) / 1000 <=
                  20,
              })}
            >
              {(
                renderTimings["native-render-timing-flush"] || 0
              ).toLocaleString()}
              μs
            </span>
          </div>
        </div>
      )}
    </div>
  );
};
