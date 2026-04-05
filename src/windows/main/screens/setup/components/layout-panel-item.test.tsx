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

  it("does not call select when layout is already selected", () => {
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
    fireEvent.pointerDown(screen.getByDisplayValue("My Score"));
    expect(actions.ui.screens.setup.select).not.toHaveBeenCalled();
  });

  it("renders checkbox when a Flow is selected (assignable mode)", () => {
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 5, selection_type: SetupSelectionType.Flow },
          },
        },
      } as never,
      score: {
        layouts: { order: [10], by_id: { 10: mockLayout } },
      } as never,
    });
    renderWithProviders(<LayoutPanelItem layout={mockLayout} />, { appState });
    expect(screen.getByRole("checkbox")).toBeInTheDocument();
  });

  it("renders checkbox when a Player is selected (assignable mode)", () => {
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 7, selection_type: SetupSelectionType.Player },
          },
        },
      } as never,
      score: {
        layouts: { order: [10], by_id: { 10: mockLayout } },
      } as never,
    });
    renderWithProviders(<LayoutPanelItem layout={mockLayout} />, { appState });
    expect(screen.getByRole("checkbox")).toBeInTheDocument();
  });

  it("checkbox is checked when layout.flow_ids includes the selected flow", () => {
    const layout: Layout = { ...mockLayout, flow_ids: [5] };
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 5, selection_type: SetupSelectionType.Flow },
          },
        },
      } as never,
      score: {
        layouts: { order: [10], by_id: { 10: layout } },
      } as never,
    });
    renderWithProviders(<LayoutPanelItem layout={layout} />, { appState });
    expect(screen.getByRole("checkbox")).toBeChecked();
  });

  it("checkbox is checked when layout.player_ids includes the selected player", () => {
    const layout: Layout = { ...mockLayout, player_ids: [7] };
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 7, selection_type: SetupSelectionType.Player },
          },
        },
      } as never,
      score: {
        layouts: { order: [10], by_id: { 10: layout } },
      } as never,
    });
    renderWithProviders(<LayoutPanelItem layout={layout} />, { appState });
    expect(screen.getByRole("checkbox")).toBeChecked();
  });

  it("calls assignFlow when checkbox toggled on with Flow selection", async () => {
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 5, selection_type: SetupSelectionType.Flow },
          },
        },
      } as never,
      score: {
        layouts: { order: [10], by_id: { 10: mockLayout } },
      } as never,
    });
    renderWithProviders(<LayoutPanelItem layout={mockLayout} />, { appState });
    fireEvent.click(screen.getByRole("checkbox"));
    await vi.runAllTimersAsync().catch(() => {});
    expect(actions.score.layout.assignFlow).toHaveBeenCalledWith(10, 5);
  });

  it("calls unassignFlow when checkbox toggled off with Flow selection", async () => {
    const layout: Layout = { ...mockLayout, flow_ids: [5] };
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 5, selection_type: SetupSelectionType.Flow },
          },
        },
      } as never,
      score: {
        layouts: { order: [10], by_id: { 10: layout } },
      } as never,
    });
    renderWithProviders(<LayoutPanelItem layout={layout} />, { appState });
    fireEvent.click(screen.getByRole("checkbox"));
    await vi.runAllTimersAsync().catch(() => {});
    expect(actions.score.layout.unassignFlow).toHaveBeenCalledWith(10, 5);
  });

  it("calls assignPlayer when checkbox toggled on with Player selection", async () => {
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 7, selection_type: SetupSelectionType.Player },
          },
        },
      } as never,
      score: {
        layouts: { order: [10], by_id: { 10: mockLayout } },
      } as never,
    });
    renderWithProviders(<LayoutPanelItem layout={mockLayout} />, { appState });
    fireEvent.click(screen.getByRole("checkbox"));
    await vi.runAllTimersAsync().catch(() => {});
    expect(actions.score.layout.assignPlayer).toHaveBeenCalledWith(10, 7);
  });

  it("calls unassignPlayer when checkbox toggled off with Player selection", async () => {
    const layout: Layout = { ...mockLayout, player_ids: [7] };
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 7, selection_type: SetupSelectionType.Player },
          },
        },
      } as never,
      score: {
        layouts: { order: [10], by_id: { 10: layout } },
      } as never,
    });
    renderWithProviders(<LayoutPanelItem layout={layout} />, { appState });
    fireEvent.click(screen.getByRole("checkbox"));
    await vi.runAllTimersAsync().catch(() => {});
    expect(actions.score.layout.unassignPlayer).toHaveBeenCalledWith(10, 7);
  });

  it("renders correctly for LayoutType.Part (uses file icon)", () => {
    const partLayout: Layout = { ...mockLayout, layout_type: LayoutType.Part };
    const appState = createAppState({
      score: {
        layouts: { order: [10], by_id: { 10: partLayout } },
      } as never,
    });
    renderWithProviders(<LayoutPanelItem layout={partLayout} />, { appState });
    expect(screen.getByDisplayValue("My Score")).toBeInTheDocument();
  });

  it("shows the edited draft while typing", () => {
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

    fireEvent.click(screen.getAllByRole("button")[0]);
    vi.clearAllMocks();

    const input = screen.getByDisplayValue("My Score");
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "Updated Score" } });

    expect(screen.getByDisplayValue("Updated Score")).toBeInTheDocument();
  });

  it("commits the rename on blur", () => {
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

    fireEvent.click(screen.getAllByRole("button")[0]);
    vi.clearAllMocks();

    const input = screen.getByDisplayValue("My Score");
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "Updated Score" } });
    fireEvent.blur(input);

    expect(actions.score.layout.rename).toHaveBeenCalledWith(
      10,
      "Updated Score",
    );
  });

  it("reverts local edits on Escape without renaming", () => {
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

    fireEvent.click(screen.getAllByRole("button")[0]);
    vi.clearAllMocks();

    const input = screen.getByDisplayValue("My Score");
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "Updated Score" } });
    fireEvent.keyDown(input, { key: "Escape" });

    expect(actions.score.layout.rename).not.toHaveBeenCalled();
    expect(screen.getByDisplayValue("My Score")).toBeInTheDocument();
  });
});
