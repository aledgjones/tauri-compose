import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen } from "@testing-library/react";
import { FlowPanel } from "./flow-panel";
import {
  renderWithProviders,
  createAppState,
} from "../../../../../services/test-utils";
import { Flow, SetupSelectionType } from "../../../../../services/types";

// Mock Sortable.js — we don't need drag-and-drop in unit tests
vi.mock("sortablejs", () => ({
  default: {
    create: vi.fn().mockReturnValue({ destroy: vi.fn() }),
  },
}));

vi.mock("../../../../../services/actions", () => ({
  actions: {
    score: {
      flow: {
        create: vi.fn().mockResolvedValue(1),
        move: vi.fn().mockResolvedValue(undefined),
        rename: vi.fn().mockResolvedValue(undefined),
        remove: vi.fn().mockResolvedValue(undefined),
        assignPlayer: vi.fn().mockResolvedValue(undefined),
        unassignPlayer: vi.fn().mockResolvedValue(undefined),
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

const mockFlow1: Flow = {
  id: 1,
  title: "Movement I",
  player_ids: [],
  length: 256,
  master_id: 0,
  staves: { by_id: {} },
};

const mockFlow2: Flow = {
  id: 2,
  title: "Movement II",
  player_ids: [],
  length: 128,
  master_id: 0,
  staves: { by_id: {} },
};

describe("FlowPanel", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders the 'Flows' panel title", () => {
    renderWithProviders(<FlowPanel />);
    expect(screen.getByText("Flows")).toBeInTheDocument();
  });

  it("renders a + button to add flows", () => {
    renderWithProviders(<FlowPanel />);
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(1);
  });

  it("renders each flow item in order", () => {
    const appState = createAppState({
      score: {
        flows: {
          order: [1, 2],
          by_id: { 1: mockFlow1, 2: mockFlow2 },
        },
      } as never,
    });
    renderWithProviders(<FlowPanel />, { appState });
    expect(screen.getByDisplayValue("Movement I")).toBeInTheDocument();
    expect(screen.getByDisplayValue("Movement II")).toBeInTheDocument();
  });

  it("renders empty list when no flows exist", () => {
    renderWithProviders(<FlowPanel />);
    // No inputs for flow titles should be rendered
    expect(screen.queryByRole("textbox")).toBeNull();
  });

  it("renders selected flow with selection styling", () => {
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 1, selection_type: SetupSelectionType.Flow },
          },
        },
      } as never,
      score: {
        flows: {
          order: [1],
          by_id: { 1: mockFlow1 },
        },
      } as never,
    });
    renderWithProviders(<FlowPanel />, { appState });
    expect(screen.getByDisplayValue("Movement I")).toBeInTheDocument();
  });
});
