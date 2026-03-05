import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import { FlowPanelItem } from "./flow-panel-item";
import {
  renderWithProviders,
  createAppState,
} from "../../../../../services/test-utils";
import { Flow, SetupSelectionType } from "../../../../../services/types";

vi.mock("../../../../../services/actions", () => ({
  actions: {
    score: {
      flow: {
        rename: vi.fn().mockResolvedValue(undefined),
        remove: vi.fn().mockResolvedValue(undefined),
        assignPlayer: vi.fn().mockResolvedValue(undefined),
        unassignPlayer: vi.fn().mockResolvedValue(undefined),
        assignLayout: vi.fn().mockResolvedValue(undefined),
        unassignLayout: vi.fn().mockResolvedValue(undefined),
      },
      layout: {
        assignFlow: vi.fn().mockResolvedValue(undefined),
        unassignFlow: vi.fn().mockResolvedValue(undefined),
      },
    },
    ui: {
      screens: {
        setup: {
          select: vi.fn(),
        },
      },
    },
  },
}));

import { actions } from "../../../../../services/actions";

const mockFlow: Flow = {
  id: 1,
  title: "Symphony No. 1",
  player_ids: [],
  length: 256,
  master_id: 0,
  staves: { by_id: {} },
};

const mockFlowNoTitle: Flow = { ...mockFlow, id: 2, title: "" };

describe("FlowPanelItem", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders the flow title", () => {
    renderWithProviders(<FlowPanelItem flow={mockFlow} index={0} />, {
      appState: createAppState({
        score: {
          flows: { order: [1], by_id: { 1: mockFlow } },
        } as never,
      }),
    });
    expect(screen.getByDisplayValue("Symphony No. 1")).toBeInTheDocument();
  });

  it("renders 'Untitled Flow' when title is empty", () => {
    renderWithProviders(<FlowPanelItem flow={mockFlowNoTitle} index={0} />, {
      appState: createAppState({
        score: {
          flows: { order: [2], by_id: { 2: mockFlowNoTitle } },
        } as never,
      }),
    });
    expect(screen.getByDisplayValue("Untitled Flow")).toBeInTheDocument();
  });

  it("renders 1-based index number", () => {
    renderWithProviders(<FlowPanelItem flow={mockFlow} index={2} />, {
      appState: createAppState({
        score: {
          flows: { order: [1], by_id: { 1: mockFlow } },
        } as never,
      }),
    });
    expect(screen.getByText("3")).toBeInTheDocument();
  });

  it("renders edit and delete buttons when flow is selected", () => {
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 1, selection_type: SetupSelectionType.Flow },
          },
        },
      } as never,
      score: {
        flows: { order: [1], by_id: { 1: mockFlow } },
      } as never,
    });
    renderWithProviders(<FlowPanelItem flow={mockFlow} index={0} />, {
      appState,
    });
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(2); // edit + delete
  });

  it("calls actions.score.flow.remove when delete button clicked", async () => {
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 1, selection_type: SetupSelectionType.Flow },
          },
        },
      } as never,
      score: {
        flows: { order: [1], by_id: { 1: mockFlow } },
      } as never,
    });
    renderWithProviders(<FlowPanelItem flow={mockFlow} index={0} />, {
      appState,
    });
    const buttons = screen.getAllByRole("button");
    // delete button is last button
    fireEvent.click(buttons[buttons.length - 1]);
    await vi.runAllTimersAsync().catch(() => {});
    expect(actions.score.flow.remove).toHaveBeenCalledWith(1);
  });

  it("calls actions.ui.screens.setup.select when unselected flow is pointer-downed", () => {
    renderWithProviders(<FlowPanelItem flow={mockFlow} index={0} />, {
      appState: createAppState({
        score: {
          flows: { order: [1], by_id: { 1: mockFlow } },
        } as never,
      }),
    });
    fireEvent.pointerDown(screen.getByDisplayValue("Symphony No. 1"));
    expect(actions.ui.screens.setup.select).toHaveBeenCalledWith(
      SetupSelectionType.Flow,
      1,
    );
  });
});
