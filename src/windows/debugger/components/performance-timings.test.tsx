import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { PerformanceTimings } from "./performance-timings";
import { createAppState } from "../../../services/test-utils";

const { mockUseAppState } = vi.hoisted(() => ({
  mockUseAppState: vi.fn(),
}));

vi.mock("../../../services/state", () => ({
  useAppState: mockUseAppState,
}));

// Capture listen callbacks so tests can fire them
type ListenerFn = (event: { payload: unknown }) => void;
const listeners: Record<string, ListenerFn> = {};

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi
    .fn()
    .mockImplementation((eventName: string, callback: ListenerFn) => {
      listeners[eventName] = callback;
      return Promise.resolve(vi.fn());
    }),
}));

// Helper: fire a timing event with a given microsecond value
const fireTimingEvent = (key: string, value: number) => {
  act(() => {
    if (listeners[key]) {
      listeners[key]({ payload: value });
    }
  });
};

const firePayloadEvent = (key: string, payload: unknown) => {
  act(() => {
    if (listeners[key]) {
      listeners[key]({ payload });
    }
  });
};

// Helper: fire all layout-phase timings with the same value
const fireLayoutTimings = (value: number) => {
  [
    "render-timing-vertical",
    "render-timing-layout",
    "render-timing-structure",
    "render-timing-notation",
    "render-timing-elements",
    "render-timing-spacing",
    "render-timing-stems",
  ].forEach((k) => fireTimingEvent(k, value));
};

// Helper: fire all render-phase timings with the same value
const fireRenderTimings = (value: number) => {
  [
    "render-timing-setup",
    "render-timing-barlines",
    "render-timing-stave",
  ].forEach((k) => fireTimingEvent(k, value));
};

// Helper: fire all GPU-phase timings with the same value
const fireGpuTimings = (value: number) => {
  [
    "native-render-timing-setup",
    "native-render-timing-draw",
    "native-render-timing-flush",
  ].forEach((k) => fireTimingEvent(k, value));
};

const buildAppState = (layoutFlowIdsByLayoutId?: Record<number, number[]>) => {
  const state = createAppState();

  state.score.flows.order = [10, 20, 30];
  state.score.flows.by_id = {
    10: {
      id: 10,
      title: "Flow 1",
      player_ids: [],
      length: 16,
      master_id: 110,
      staves: { by_id: {} },
    },
    20: {
      id: 20,
      title: "Flow 2",
      player_ids: [],
      length: 16,
      master_id: 120,
      staves: { by_id: {} },
    },
    30: {
      id: 30,
      title: "Flow 3",
      player_ids: [],
      length: 16,
      master_id: 130,
      staves: { by_id: {} },
    },
  };

  state.score.layouts.order = [4, 9];
  state.score.layouts.by_id[4] = {
    id: 4,
    title: "Layout 4",
    layout_type: 0,
    player_ids: [],
    flow_ids: layoutFlowIdsByLayoutId?.[4] ?? [10, 20, 30],
  };
  state.score.layouts.by_id[9] = {
    id: 9,
    title: "Layout 9",
    layout_type: 0,
    player_ids: [],
    flow_ids: layoutFlowIdsByLayoutId?.[9] ?? [10, 20, 30],
  };

  return state;
};

describe("PerformanceTimings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseAppState.mockReturnValue(buildAppState());
    // Reset listeners map
    for (const key of Object.keys(listeners)) {
      delete listeners[key];
    }
  });

  it("renders all three section toggle buttons", () => {
    render(<PerformanceTimings />);
    expect(screen.getByText(/Layout Phase/)).toBeInTheDocument();
    expect(screen.getByText(/Render Phase/)).toBeInTheDocument();
    expect(screen.getByText(/GPU/)).toBeInTheDocument();
  });

  it("layout section starts collapsed (aria-expanded=false)", () => {
    render(<PerformanceTimings />);
    const layoutBtn = screen.getByRole("button", { name: /Layout Phase/ });
    expect(layoutBtn.getAttribute("aria-expanded")).toBe("false");
  });

  it("toggles layout section open on click", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    const layoutBtn = screen.getByRole("button", { name: /Layout Phase/ });
    await user.click(layoutBtn);
    expect(layoutBtn.getAttribute("aria-expanded")).toBe("true");
  });

  it("toggles layout section closed on second click", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    const layoutBtn = screen.getByRole("button", { name: /Layout Phase/ });
    await user.click(layoutBtn);
    await user.click(layoutBtn);
    expect(layoutBtn.getAttribute("aria-expanded")).toBe("false");
  });

  it("render phase section starts collapsed and can be toggled", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    const renderBtn = screen.getByRole("button", { name: /Render Phase/ });
    expect(renderBtn.getAttribute("aria-expanded")).toBe("false");
    await user.click(renderBtn);
    expect(renderBtn.getAttribute("aria-expanded")).toBe("true");
  });

  it("GPU phase section starts collapsed and can be toggled", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    const gpuBtn = screen.getByRole("button", { name: /GPU/ });
    expect(gpuBtn.getAttribute("aria-expanded")).toBe("false");
    await user.click(gpuBtn);
    expect(gpuBtn.getAttribute("aria-expanded")).toBe("true");
  });

  it("layout section header shows text-red-400 when total > 50ms", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    // 100_000 µs each × 7 = 700_000 µs / 1000 = 700ms > 50ms → red
    fireLayoutTimings(100_000);
    const layoutBtn = screen.getByRole("button", { name: /Layout Phase/ });
    // The header span should have red styling
    const headerSpan = layoutBtn.querySelector("span:last-child");
    expect(headerSpan?.className).toContain("text-red-400");
    // open it to verify row-level red
    await user.click(layoutBtn);
    expect(screen.getByText("Vertical spacing & staves")).toBeInTheDocument();
  });

  it("layout section header shows text-yellow-400 when total is 20-50ms", async () => {
    render(<PerformanceTimings />);
    // 7_000 µs each × 7 = 49_000 µs / 1000 = 49ms → yellow (>20ms and <=50ms)
    fireLayoutTimings(7_000);
    const layoutBtn = screen.getByRole("button", { name: /Layout Phase/ });
    const headerSpan = layoutBtn.querySelector("span:last-child");
    expect(headerSpan?.className).toContain("text-yellow-400");
  });

  it("layout section header shows text-green-400 when total <= 20ms", async () => {
    render(<PerformanceTimings />);
    // 0 µs (default) → green
    const layoutBtn = screen.getByRole("button", { name: /Layout Phase/ });
    const headerSpan = layoutBtn.querySelector("span:last-child");
    expect(headerSpan?.className).toContain("text-green-400");
  });

  it("individual row in layout section shows text-red-400 for high timing", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    fireTimingEvent("render-timing-vertical", 100_000); // 100ms > 50ms
    await user.click(screen.getByRole("button", { name: /Layout Phase/ }));
    expect(screen.getByText("Vertical spacing & staves")).toBeInTheDocument();
    // The value span next to it should have text-red-400
    const row = screen.getByText("Vertical spacing & staves").closest("div");
    const valueSpan = row?.querySelector("span:last-child");
    expect(valueSpan?.className).toContain("text-red-400");
  });

  it("individual row in layout section shows text-yellow-400 for mid timing", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    fireTimingEvent("render-timing-layout", 30_000); // 30ms → yellow
    await user.click(screen.getByRole("button", { name: /Layout Phase/ }));
    const row = screen.getByText("Layout calculations").closest("div");
    const valueSpan = row?.querySelector("span:last-child");
    expect(valueSpan?.className).toContain("text-yellow-400");
  });

  it("render phase header shows text-red-400 when total > 50ms", async () => {
    render(<PerformanceTimings />);
    fireRenderTimings(100_000);
    const renderBtn = screen.getByRole("button", { name: /Render Phase/ });
    const headerSpan = renderBtn.querySelector("span:last-child");
    expect(headerSpan?.className).toContain("text-red-400");
  });

  it("render phase header shows text-yellow-400 for mid timings", async () => {
    render(<PerformanceTimings />);
    fireRenderTimings(25_000); // 25ms × 3 = 75ms/3 but each is 25ms - section total = 75ms/1000 = 75ms per sum
    // Actually 25_000 × 3 = 75_000µs / 1000 = 75ms → RED since > 50ms
    // Need smaller value: 7_000 × 3 = 21_000µs / 1000 = 21ms → yellow
    fireRenderTimings(7_000);
    const renderBtn = screen.getByRole("button", { name: /Render Phase/ });
    const headerSpan = renderBtn.querySelector("span:last-child");
    expect(headerSpan?.className).toContain("text-yellow-400");
  });

  it("render phase rows expand and show correct values", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    fireRenderTimings(100_000);
    await user.click(screen.getByRole("button", { name: /Render Phase/ }));
    expect(screen.getByText("Basic drawing setup")).toBeInTheDocument();
    expect(screen.getByText("Barline rendering")).toBeInTheDocument();
    expect(screen.getByText("Stave contents rendering")).toBeInTheDocument();
    const row = screen.getByText("Barline rendering").closest("div");
    const valueSpan = row?.querySelector("span:last-child");
    expect(valueSpan?.className).toContain("text-red-400");
  });

  it("GPU phase header shows text-red-400 when total > 50ms", async () => {
    render(<PerformanceTimings />);
    fireGpuTimings(100_000);
    const gpuBtn = screen.getByRole("button", { name: /GPU/ });
    const headerSpan = gpuBtn.querySelector("span:last-child");
    expect(headerSpan?.className).toContain("text-red-400");
  });

  it("GPU phase header shows text-yellow-400 for mid timings", async () => {
    render(<PerformanceTimings />);
    fireGpuTimings(10_000); // 10_000 × 3 = 30_000µs / 1000 = 30ms → yellow
    const gpuBtn = screen.getByRole("button", { name: /GPU/ });
    const headerSpan = gpuBtn.querySelector("span:last-child");
    expect(headerSpan?.className).toContain("text-yellow-400");
  });

  it("GPU phase rows expand and show red timing values", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    fireGpuTimings(100_000);
    await user.click(screen.getByRole("button", { name: /GPU/ }));
    expect(screen.getByText("GPU surface & canvas setup")).toBeInTheDocument();
    expect(screen.getByText("GPU drawing execution")).toBeInTheDocument();
    expect(screen.getByText("GPU flush & Metal commands")).toBeInTheDocument();
    const row = screen.getByText("GPU drawing execution").closest("div");
    const valueSpan = row?.querySelector("span:last-child");
    expect(valueSpan?.className).toContain("text-red-400");
  });

  it("displays numeric timing values from events", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    fireTimingEvent("render-timing-vertical", 100_000);
    await user.click(screen.getByRole("button", { name: /Layout Phase/ }));
    expect(screen.getByText("Vertical spacing & staves")).toBeInTheDocument();
    // The section should show non-zero timing
    const row = screen.getByText("Vertical spacing & staves").closest("div");
    const valueSpan = row?.querySelector("span:last-child");
    expect(valueSpan?.textContent).not.toBe("0μs");
  });

  it("layout row shows text-green-400 for low individual timing", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    fireTimingEvent("render-timing-structure", 5_000); // 5ms → green
    await user.click(screen.getByRole("button", { name: /Layout Phase/ }));
    const row = screen.getByText("Score structure parsing").closest("div");
    const valueSpan = row?.querySelector("span:last-child");
    expect(valueSpan?.className).toContain("text-green-400");
  });

  it("render row shows text-yellow-400 for stave contents", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    fireTimingEvent("render-timing-stave", 25_000); // 25ms → yellow
    await user.click(screen.getByRole("button", { name: /Render Phase/ }));
    const row = screen.getByText("Stave contents rendering").closest("div");
    const valueSpan = row?.querySelector("span:last-child");
    expect(valueSpan?.className).toContain("text-yellow-400");
  });

  it("shows flow-aware preparation diagnostics inside the layout timing panel", async () => {
    render(<PerformanceTimings />);

    fireTimingEvent("render-timing-flow-preparation", 12_000);
    firePayloadEvent("render-timing-flow-preparation-details", {
      render_mode: "shared_surface",
      layout_id: 4,
      active_flow_id: 20,
      active_flow_visible_in_layout: true,
      prepared_flow_ids: [10, 20],
      prepared_flow_count: 2,
      prepared_flow_scope: "multiple_flows",
      composed_flow_ids: [10, 20],
      composed_flow_count: 2,
      composed_flow_scope: "multiple_flows",
      recalculated_flow_ids: [10, 20],
      recalculated_flow_count: 2,
      recalculation_scope: "multiple_flows",
    });

    expect(screen.getByText(/Render Scope/)).toBeInTheDocument();
    expect(screen.getByText("Flow cards")).toBeInTheDocument();
    expect(screen.getByText(/Shared update/)).toBeInTheDocument();
    expect(screen.getByText(/Unified shared surface/)).toBeInTheDocument();
    expect(screen.getByText(/Planned 2 · Multiple flows/)).toBeInTheDocument();
    expect(
      screen.getByText(/Recalculated 2 · Multiple flows/),
    ).toBeInTheDocument();
    expect(screen.getByText(/Composed 2 · Multiple flows/)).toBeInTheDocument();
    expect(
      screen.getByText(/Multiple visible flows changed in this pass./),
    ).toBeInTheDocument();
    expect(screen.getAllByText(/^Flow 1$/).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/^Flow 2$/).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/^Flow 3$/).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Flow 3/).length).toBeGreaterThan(0);
    expect(screen.getByText(/Not planned/)).toBeInTheDocument();
    expect(screen.getByText(/^Active$/)).toBeInTheDocument();
    expect(screen.getAllByText(/^Planned$/).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/^Recalculated$/).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/^Composed$/).length).toBeGreaterThan(0);
    expect(
      screen.getByText(/Layout 4 · Active flow 20 · Active flow visible: yes/),
    ).toBeInTheDocument();
  });

  it("shows shared surface and viewport diagnostics when render surface details are emitted", async () => {
    render(<PerformanceTimings />);

    fireTimingEvent("render-timing-flow-preparation", 8_000);
    firePayloadEvent("render-timing-flow-preparation-details", {
      render_mode: "shared_surface",
      layout_id: 4,
      active_flow_id: 20,
      active_flow_visible_in_layout: true,
      prepared_flow_ids: [10, 20],
      prepared_flow_count: 2,
      prepared_flow_scope: "multiple_flows",
      composed_flow_ids: [10, 20],
      composed_flow_count: 2,
      composed_flow_scope: "multiple_flows",
      recalculated_flow_ids: [10, 20],
      recalculated_flow_count: 2,
      recalculation_scope: "multiple_flows",
    });
    firePayloadEvent("render-timing-surface-details", {
      surface_width_px: 1440,
      surface_height_px: 480,
      viewport_width_px: 960,
      viewport_height_px: 420,
      page_count: 2,
      inter_flow_gap_px: 20,
      page_boxes: [
        { page_index: 0, x_px: 0, width_px: 710 },
        { page_index: 1, x_px: 730, width_px: 710 },
      ],
    });

    expect(
      screen.getByText(
        /Canvas 1440.0px x 480.0px · Viewport 960.0px x 420.0px · Gap 20.0px/,
      ),
    ).toBeInTheDocument();
    expect(screen.getByText(/Surface map/)).toBeInTheDocument();
    expect(screen.getByText(/Page 1/)).toBeInTheDocument();
    expect(screen.getByText(/Page 2/)).toBeInTheDocument();
    expect(screen.getByText(/^0.0px$/)).toBeInTheDocument();
    expect(screen.getAllByText(/^710.0px$/)).toHaveLength(2);
  });

  it("shows all composed page boxes when 3+ flows are rendered on one surface", async () => {
    render(<PerformanceTimings />);

    fireTimingEvent("render-timing-flow-preparation", 8_000);
    firePayloadEvent("render-timing-flow-preparation-details", {
      render_mode: "shared_surface",
      layout_id: 4,
      active_flow_id: 20,
      active_flow_visible_in_layout: true,
      prepared_flow_ids: [10, 20, 30],
      prepared_flow_count: 3,
      prepared_flow_scope: "multiple_flows",
      composed_flow_ids: [10, 20, 30],
      composed_flow_count: 3,
      composed_flow_scope: "multiple_flows",
      recalculated_flow_ids: [10, 20, 30],
      recalculated_flow_count: 3,
      recalculation_scope: "multiple_flows",
    });
    firePayloadEvent("render-timing-surface-details", {
      surface_width_px: 2180,
      surface_height_px: 480,
      viewport_width_px: 960,
      viewport_height_px: 420,
      page_count: 3,
      inter_flow_gap_px: 20,
      page_boxes: [
        { page_index: 0, x_px: 0, width_px: 700 },
        { page_index: 1, x_px: 720, width_px: 700 },
        { page_index: 2, x_px: 1440, width_px: 740 },
      ],
    });

    expect(
      screen.getByText(
        /Canvas 2180.0px x 480.0px · Viewport 960.0px x 420.0px · Gap 20.0px/,
      ),
    ).toBeInTheDocument();
    expect(screen.getByText(/Page 3/)).toBeInTheDocument();
    expect(screen.getByText(/^1440.0px$/)).toBeInTheDocument();
    expect(screen.getByText(/^740.0px$/)).toBeInTheDocument();
  });

  it("clears stale flow-aware preparation diagnostics when the backend resets them", async () => {
    mockUseAppState.mockReturnValue(
      buildAppState({ 4: [10], 9: [10, 20, 30] }),
    );

    render(<PerformanceTimings />);

    fireTimingEvent("render-timing-flow-preparation", 12_000);
    firePayloadEvent("render-timing-flow-preparation-details", {
      render_mode: "shared_surface",
      layout_id: 4,
      active_flow_id: 20,
      active_flow_visible_in_layout: false,
      prepared_flow_ids: [10],
      prepared_flow_count: 1,
      prepared_flow_scope: "single_flow",
      composed_flow_ids: [10],
      composed_flow_count: 1,
      composed_flow_scope: "single_flow",
      recalculated_flow_ids: [10],
      recalculated_flow_count: 1,
      recalculation_scope: "single_flow",
    });
    firePayloadEvent("render-timing-surface-details", {
      surface_width_px: 1440,
      surface_height_px: 480,
      viewport_width_px: 960,
      viewport_height_px: 420,
      page_count: 1,
      inter_flow_gap_px: 0,
      page_boxes: [{ page_index: 0, x_px: 0, width_px: 1440 }],
    });

    expect(screen.getByText(/Targeted update/)).toBeInTheDocument();
    expect(screen.getByText(/Planned 1 · Single flow/)).toBeInTheDocument();
    expect(
      screen.getByText(/Recalculated 1 · Single flow/),
    ).toBeInTheDocument();
    expect(screen.getByText(/Composed 1 · Single flow/)).toBeInTheDocument();
    expect(
      screen.getByText(/One visible flow changed in this pass./),
    ).toBeInTheDocument();
    expect(screen.getAllByText(/^Flow 1$/).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/^Flow 2$/).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/^Flow 3$/).length).toBeGreaterThan(0);
    expect(screen.getByText(/^Active$/)).toBeInTheDocument();
    expect(screen.getAllByText(/Not planned/)).toHaveLength(2);
    expect(
      screen.getByText(
        /Canvas 1440.0px x 480.0px · Viewport 960.0px x 420.0px/,
      ),
    ).toBeInTheDocument();

    fireTimingEvent("render-timing-flow-preparation", 0);
    firePayloadEvent("render-timing-flow-preparation-details", null);
    firePayloadEvent("render-timing-surface-details", null);

    expect(screen.getByText(/No active render/)).toBeInTheDocument();
    expect(
      screen.getByText(/Waiting for a render plan or composed surface./),
    ).toBeInTheDocument();
    expect(
      screen.queryByText(/Canvas: 1440.0px x 480.0px/),
    ).not.toBeInTheDocument();
  });

  it("shows broad recalculation scope when multiple visible flows are recomputed", async () => {
    render(<PerformanceTimings />);

    fireTimingEvent("render-timing-flow-preparation", 9_000);
    firePayloadEvent("render-timing-flow-preparation-details", {
      render_mode: "shared_surface",
      layout_id: 9,
      active_flow_id: 30,
      active_flow_visible_in_layout: true,
      prepared_flow_ids: [10, 20, 30],
      prepared_flow_count: 3,
      prepared_flow_scope: "multiple_flows",
      composed_flow_ids: [10, 20, 30],
      composed_flow_count: 3,
      composed_flow_scope: "multiple_flows",
      recalculated_flow_ids: [10, 20, 30],
      recalculated_flow_count: 3,
      recalculation_scope: "multiple_flows",
    });

    expect(screen.getByText(/Shared update/)).toBeInTheDocument();
    expect(
      screen.getByText(/Multiple visible flows changed in this pass./),
    ).toBeInTheDocument();
    expect(screen.getAllByText(/^Flow 3$/).length).toBeGreaterThan(0);
    expect(
      screen.getByText(/Recalculated 3 · Multiple flows/),
    ).toBeInTheDocument();
    expect(screen.getByText(/^Active$/)).toBeInTheDocument();
  });

  it("counts flow preparation in the layout header severity", () => {
    render(<PerformanceTimings />);

    fireTimingEvent("render-timing-flow-preparation", 30_000);

    const layoutBtn = screen.getByRole("button", { name: /Layout Phase/ });
    const headerSpan = layoutBtn.querySelector("span:last-child");
    expect(headerSpan?.className).toContain("text-yellow-400");
  });

  it("GPU row shows text-yellow-400 for flush timing", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    fireTimingEvent("native-render-timing-flush", 25_000); // 25ms → yellow
    await user.click(screen.getByRole("button", { name: /GPU/ }));
    const row = screen.getByText("GPU flush & Metal commands").closest("div");
    const valueSpan = row?.querySelector("span:last-child");
    expect(valueSpan?.className).toContain("text-yellow-400");
  });
});
