import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import classNames from "classnames";

type FlowPreparationDetails = {
  layout_id: number;
  active_flow_id: number;
  active_flow_visible_in_layout: boolean;
  fallback_draw_flow_id: number | null;
  prepared_flow_ids: number[];
  prepared_flow_count: number;
  composed_flow_ids: number[];
  composed_flow_count: number;
  single_flow_draw_fallback: boolean;
};

type RenderSurfaceDetails = {
  surface_width_px: number;
  surface_height_px: number;
  viewport_width_px: number;
  viewport_height_px: number;
  page_count: number;
  inter_flow_gap_px: number;
  first_page_x_px: number;
  second_page_x_px: number | null;
  first_page_width_px: number;
  second_page_width_px: number | null;
};

export const PerformanceTimings = () => {
  const [renderTimings, setRenderTimings] = useState<Record<string, number>>(
    {},
  );
  const [flowPreparationDetails, setFlowPreparationDetails] =
    useState<FlowPreparationDetails | null>(null);
  const [renderSurfaceDetails, setRenderSurfaceDetails] =
    useState<RenderSurfaceDetails | null>(null);
  const [layoutCollapsed, setLayoutCollapsed] = useState(true);
  const [renderCollapsed, setRenderCollapsed] = useState(true);
  const [gpuCollapsed, setGpuCollapsed] = useState(true);

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
        setFlowPreparationDetails(
          (event.payload as FlowPreparationDetails | null) ?? null,
        );
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
          {flowPreparationDetails && (
            <div className="mt-2 rounded bg-neutral-950/60 px-2 py-1.5 text-xs text-neutral-400">
              <div>
                Layout {flowPreparationDetails.layout_id} | Active flow{" "}
                {flowPreparationDetails.active_flow_id}
              </div>
              <div>
                Active flow visible in layout:{" "}
                {flowPreparationDetails.active_flow_visible_in_layout
                  ? "yes"
                  : "no"}
              </div>
              <div>
                Prepared flows:{" "}
                {flowPreparationDetails.prepared_flow_ids.join(", ") || "none"}
              </div>
              <div>
                Prepared flow count:{" "}
                {flowPreparationDetails.prepared_flow_count}
              </div>
              <div>
                Composed flows:{" "}
                {flowPreparationDetails.composed_flow_ids.join(", ") || "none"}
              </div>
              <div>
                Composed flow count:{" "}
                {flowPreparationDetails.composed_flow_count}
              </div>
              <div>
                Fallback draw mode:{" "}
                {flowPreparationDetails.single_flow_draw_fallback
                  ? "active-flow-only"
                  : "multi-flow-surface"}
              </div>
              {flowPreparationDetails.single_flow_draw_fallback &&
                flowPreparationDetails.fallback_draw_flow_id !== null && (
                  <div>
                    Fallback draw flow:{" "}
                    {flowPreparationDetails.fallback_draw_flow_id}
                  </div>
                )}
              {renderSurfaceDetails && (
                <>
                  <div>
                    Shared surface:{" "}
                    {renderSurfaceDetails.surface_width_px.toFixed(1)}px x{" "}
                    {renderSurfaceDetails.surface_height_px.toFixed(1)}px
                  </div>
                  <div>Page count: {renderSurfaceDetails.page_count}</div>
                  <div>
                    Page boxes: page 1 @{" "}
                    {renderSurfaceDetails.first_page_x_px.toFixed(1)}px, width{" "}
                    {renderSurfaceDetails.first_page_width_px.toFixed(1)}px
                    {renderSurfaceDetails.second_page_x_px !== null &&
                      renderSurfaceDetails.second_page_width_px !== null &&
                      ` | page 2 @ ${renderSurfaceDetails.second_page_x_px.toFixed(1)}px, width ${renderSurfaceDetails.second_page_width_px.toFixed(1)}px`}
                  </div>
                  {renderSurfaceDetails.page_count > 1 && (
                    <div>
                      Inter-flow gap:{" "}
                      {renderSurfaceDetails.inter_flow_gap_px.toFixed(1)}px
                    </div>
                  )}
                  <div>
                    Viewport:{" "}
                    {renderSurfaceDetails.viewport_width_px.toFixed(1)}px x{" "}
                    {renderSurfaceDetails.viewport_height_px.toFixed(1)}px
                  </div>
                </>
              )}
            </div>
          )}
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
