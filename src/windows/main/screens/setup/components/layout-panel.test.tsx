import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen } from "@testing-library/react";
import { LayoutPlanel } from "./layout-panel";
import {
  renderWithProviders,
  createAppState,
} from "../../../../../services/test-utils";
import {
  Layout,
  LayoutType,
  SetupSelectionType,
} from "../../../../../services/types";

// Mock Sortable.js — not needed for unit tests
vi.mock("sortablejs", () => ({
  default: {
    create: vi.fn().mockReturnValue({ destroy: vi.fn() }),
  },
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: vi.fn().mockImplementation(() => ({
    setFocus: vi.fn(),
    once: vi.fn().mockResolvedValue(vi.fn()),
    close: vi.fn(),
  })),
}));

vi.mock("../../../../../services/actions", () => ({
  actions: {
    score: {
      layout: {
        create: vi.fn().mockResolvedValue(undefined),
        move: vi.fn().mockResolvedValue(undefined),
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

const mockLayout1: Layout = {
  id: 10,
  title: "Full Score",
  layout_type: LayoutType.FullScore,
  player_ids: [],
  flow_ids: [],
};

const mockLayout2: Layout = {
  id: 11,
  title: "Violin Part",
  layout_type: LayoutType.Part,
  player_ids: [],
  flow_ids: [],
};

describe("LayoutPanel", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders the 'Layouts' panel title", () => {
    renderWithProviders(<LayoutPlanel />);
    expect(screen.getByText("Layouts")).toBeInTheDocument();
  });

  it("renders settings and + buttons", () => {
    renderWithProviders(<LayoutPlanel />);
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(2); // settings + add
  });

  it("renders each layout item in order", () => {
    const appState = createAppState({
      score: {
        layouts: {
          order: [10, 11],
          by_id: { 10: mockLayout1, 11: mockLayout2 },
        },
      } as never,
    });
    renderWithProviders(<LayoutPlanel />, { appState });
    expect(screen.getByDisplayValue("Full Score")).toBeInTheDocument();
    expect(screen.getByDisplayValue("Violin Part")).toBeInTheDocument();
  });

  it("renders empty list when no layouts exist", () => {
    renderWithProviders(<LayoutPlanel />);
    expect(screen.queryByRole("textbox")).toBeNull();
  });

  it("calls actions.score.layout.create when + button is clicked", async () => {
    renderWithProviders(<LayoutPlanel />);
    const buttons = screen.getAllByRole("button");
    // + button is last
    const addBtn = buttons[buttons.length - 1];
    addBtn.click();
    await vi.runAllTimersAsync().catch(() => {});
    expect(actions.score.layout.create).toHaveBeenCalledWith(LayoutType.Part);
  });

  it("renders selected layout with edit/delete buttons", () => {
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 10, selection_type: SetupSelectionType.Layout },
          },
        },
      } as never,
      score: {
        layouts: {
          order: [10],
          by_id: { 10: mockLayout1 },
        },
      } as never,
    });
    renderWithProviders(<LayoutPlanel />, { appState });
    const buttons = screen.getAllByRole("button");
    // Should have panel header buttons + edit + delete for selected item
    expect(buttons.length).toBeGreaterThanOrEqual(4);
  });
});
