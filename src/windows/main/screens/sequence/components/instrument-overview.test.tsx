import { describe, it, expect, vi, beforeEach } from "vitest";
import { fireEvent, screen } from "@testing-library/react";
import { InstrumentOverview } from "./instrument-overview";
import {
  renderWithProviders,
  createAppState,
} from "../../../../../services/test-utils";
import {
  Flow,
  Instrument,
  InstrumentType,
  Player,
  PlayerType,
} from "../../../../../services/types";

import { actions } from "../../../../../services/actions";

vi.mock("../../../../../services/actions", () => ({
  actions: {
    ui: {
      setActiveFlowId: vi.fn(),
    },
    score: {
      instrument: {
        setSolo: vi.fn().mockResolvedValue(undefined),
        setMute: vi.fn().mockResolvedValue(undefined),
      },
    },
  },
}));

const mockFlow: Flow = {
  id: 1,
  title: "Test Flow",
  player_ids: [10],
  length: 0,
  master_id: 0,
  staves: { by_id: {} },
};

const mockInstrument: Instrument = {
  id: 20,
  definition_id: "piano",
  instrument_type: InstrumentType.Melodic,
  long_name: "Piano",
  short_name: "Pno.",
  stave_ids: [],
  count: null,
  volume: 100,
  pan: 64,
  mute: false,
  solo: false,
};

const mockPlayer: Player = {
  id: 10,
  player_type: PlayerType.Solo,
  instrument_ids: [20],
};

describe("InstrumentOverview", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders 'no flows' message when there are no flows", () => {
    renderWithProviders(<InstrumentOverview />);
    expect(
      screen.getByText(/you don't have any flows yet/i),
    ).toBeInTheDocument();
  });

  it("renders without crashing when a flow exists", () => {
    const appState = createAppState({
      ui: { flow_id: 1 } as never,
      score: {
        flows: {
          order: [1],
          by_id: { 1: mockFlow },
        },
        players: {
          order: [10],
          by_id: { 10: mockPlayer },
        },
        instruments: {
          by_id: { 20: mockInstrument },
        },
      } as never,
    });
    renderWithProviders(<InstrumentOverview />, { appState });
    // Should render the instrument name in the overview row
    expect(screen.getByText("Piano")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "1. Test Flow" }),
    ).toBeInTheDocument();
  });

  it("falls back to first flow when ui.flow_id is null but flows exist", () => {
    const appState = createAppState({
      ui: { flow_id: null } as never,
      score: {
        flows: {
          order: [1],
          by_id: { 1: { ...mockFlow, player_ids: [] } },
        },
      } as never,
    });
    renderWithProviders(<InstrumentOverview />, { appState });
    // No instruments, flow exists — should render the instrument list area (no 'no flows' message)
    expect(screen.queryByText(/you don't have any flows yet/i)).toBeNull();
  });

  it("falls back to the first available flow when ui.flow_id is stale", () => {
    const appState = createAppState({
      ui: { flow_id: 99 } as never,
      score: {
        flows: {
          order: [1, 2],
          by_id: {
            1: { ...mockFlow, id: 1, title: "Flow A", player_ids: [] },
            2: { ...mockFlow, id: 2, title: "Flow B", player_ids: [] },
          },
        },
      } as never,
    });

    renderWithProviders(<InstrumentOverview />, { appState });

    expect(screen.queryByText(/you don't have any flows yet/i)).toBeNull();
    expect(
      screen.getByRole("button", { name: "1. Flow A" }),
    ).toBeInTheDocument();
  });

  it("falls back to the first valid flow in order when earlier flow ids are missing", () => {
    const appState = createAppState({
      ui: { flow_id: 99 } as never,
      score: {
        flows: {
          order: [1, 2, 3],
          by_id: {
            2: { ...mockFlow, id: 2, title: "Flow B", player_ids: [] },
            3: { ...mockFlow, id: 3, title: "Flow C", player_ids: [] },
          },
        },
      } as never,
    });

    renderWithProviders(<InstrumentOverview />, { appState });

    expect(screen.queryByText(/you don't have any flows yet/i)).toBeNull();
    expect(
      screen.getByRole("button", { name: "2. Flow B" }),
    ).toBeInTheDocument();
  });

  it("renders the flow picker inside the sticky left overview header when multiple flows exist", () => {
    const appState = createAppState({
      ui: { flow_id: 2 } as never,
      score: {
        flows: {
          order: [1, 2],
          by_id: {
            1: { ...mockFlow, id: 1, title: "Flow A" },
            2: { ...mockFlow, id: 2, title: "Flow B" },
          },
        },
        players: {
          order: [10],
          by_id: { 10: mockPlayer },
        },
        instruments: {
          by_id: { 20: mockInstrument },
        },
      } as never,
    });

    renderWithProviders(<InstrumentOverview />, { appState });

    expect(
      screen.getByRole("button", { name: "2. Flow B" }),
    ).toBeInTheDocument();
  });

  it("renders the flow picker when only one flow exists", () => {
    const appState = createAppState({
      ui: { flow_id: 1 } as never,
      score: {
        flows: {
          order: [1],
          by_id: {
            1: { ...mockFlow, id: 1, title: "Flow A" },
          },
        },
        players: {
          order: [10],
          by_id: { 10: mockPlayer },
        },
        instruments: {
          by_id: { 20: mockInstrument },
        },
      } as never,
    });

    renderWithProviders(<InstrumentOverview />, { appState });

    expect(
      screen.getByRole("button", { name: "1. Flow A" }),
    ).toBeInTheDocument();
  });

  it("dispatches setActiveFlowId when a different flow is chosen", () => {
    const appState = createAppState({
      ui: { flow_id: 2 } as never,
      score: {
        flows: {
          order: [1, 2],
          by_id: {
            1: { ...mockFlow, id: 1, title: "Flow A" },
            2: { ...mockFlow, id: 2, title: "Flow B" },
          },
        },
        players: {
          order: [10],
          by_id: { 10: mockPlayer },
        },
        instruments: {
          by_id: { 20: mockInstrument },
        },
      } as never,
    });

    renderWithProviders(<InstrumentOverview />, { appState });

    fireEvent.click(screen.getByRole("button", { name: "2. Flow B" }));
    fireEvent.click(screen.getByRole("option", { name: "1. Flow A" }));

    expect(actions.ui.setActiveFlowId).toHaveBeenCalledWith(1);
  });
});
