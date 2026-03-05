import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import { LayoutPanelItem } from "./layout-panel-item";
import {
  renderWithProviders,
  createAppState,
} from "../../../../../services/test-utils";
import {
  Layout,
  LayoutType,
  SetupSelectionType,
} from "../../../../../services/types";

vi.mock("../../../../../services/actions", () => ({
  actions: {
    score: {
      layout: {
        rename: vi.fn().mockResolvedValue(undefined),
        remove: vi.fn().mockResolvedValue(undefined),
        assignPlayer: vi.fn().mockResolvedValue(undefined),
        unassignPlayer: vi.fn().mockResolvedValue(undefined),
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

const mockLayout: Layout = {
  id: 10,
  title: "My Score",
  layout_type: LayoutType.FullScore,
  player_ids: [],
  flow_ids: [],
};

const mockLayoutNoTitle: Layout = {
  ...mockLayout,
  id: 11,
  title: "",
  layout_type: LayoutType.FullScore,
};

describe("LayoutPanelItem", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders the layout title when set", () => {
    renderWithProviders(<LayoutPanelItem layout={mockLayout} />, {
      appState: createAppState({
        score: {
          layouts: { order: [10], by_id: { 10: mockLayout } },
        } as never,
      }),
    });
    expect(screen.getByDisplayValue("My Score")).toBeInTheDocument();
  });

  it("renders 'Full Score' for a full score layout with no title", () => {
    renderWithProviders(<LayoutPanelItem layout={mockLayoutNoTitle} />, {
      appState: createAppState({
        score: {
          layouts: { order: [11], by_id: { 11: mockLayoutNoTitle } },
        } as never,
      }),
    });
    expect(screen.getByDisplayValue("Full Score")).toBeInTheDocument();
  });

  it("renders edit and delete buttons when layout is selected", () => {
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 10, selection_type: SetupSelectionType.Layout },
          },
        },
      } as never,
      score: {
        layouts: { order: [10], by_id: { 10: mockLayout } },
      } as never,
    });
    renderWithProviders(<LayoutPanelItem layout={mockLayout} />, { appState });
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(2);
  });

  it("calls actions.score.layout.remove when delete button clicked", async () => {
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 10, selection_type: SetupSelectionType.Layout },
          },
        },
      } as never,
      score: {
        layouts: { order: [10], by_id: { 10: mockLayout } },
      } as never,
    });
    renderWithProviders(<LayoutPanelItem layout={mockLayout} />, { appState });
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[buttons.length - 1]);
    await vi.runAllTimersAsync().catch(() => {});
    expect(actions.score.layout.remove).toHaveBeenCalledWith(10);
  });

  it("calls actions.ui.screens.setup.select when unselected layout is pointer-downed", () => {
    renderWithProviders(<LayoutPanelItem layout={mockLayout} />, {
      appState: createAppState({
        score: {
          layouts: { order: [10], by_id: { 10: mockLayout } },
        } as never,
      }),
    });
    fireEvent.pointerDown(screen.getByDisplayValue("My Score"));
    expect(actions.ui.screens.setup.select).toHaveBeenCalledWith(
      SetupSelectionType.Layout,
      10,
    );
  });
});
