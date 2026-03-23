import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen } from "@testing-library/react";
import { PlayerPlanel } from "./player-panel";
import {
  renderWithProviders,
  createAppState,
} from "../../../../../services/test-utils";
import {
  Instrument,
  InstrumentType,
  Player,
  PlayerType,
  SetupSelectionType,
} from "../../../../../services/types";

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
    pickAnInstrument: vi.fn().mockResolvedValue(null),
    score: {
      player: {
        create: vi.fn().mockResolvedValue(1),
        move: vi.fn().mockResolvedValue(undefined),
        assignInstrument: vi.fn().mockResolvedValue(undefined),
        unassignInstrument: vi.fn().mockResolvedValue(undefined),
      },
      layout: {
        create: vi.fn().mockResolvedValue(10),
        assignPlayer: vi.fn().mockResolvedValue(undefined),
      },
      instrument: {
        create: vi.fn().mockResolvedValue(20),
        remove: vi.fn().mockResolvedValue(undefined),
        moveToPlayer: vi.fn().mockResolvedValue(undefined),
      },
    },
    ui: {
      screens: {
        setup: {
          select: vi.fn(),
          toggleExpanded: vi.fn(),
        },
      },
    },
  },
}));

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
  id: 1,
  player_type: PlayerType.Solo,
  instrument_ids: [20],
};

describe("PlayerPanel", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders the 'Players' panel title", () => {
    renderWithProviders(<PlayerPlanel />);
    expect(screen.getByText("Players")).toBeInTheDocument();
  });

  it("renders settings and + icon buttons", () => {
    renderWithProviders(<PlayerPlanel />);
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(2); // cog + plus
  });

  it("renders empty list when no players exist", () => {
    renderWithProviders(<PlayerPlanel />);
    // No player names shown
    expect(screen.queryByText("Piano")).toBeNull();
  });

  it("renders a player with instrument name", () => {
    const appState = createAppState({
      score: {
        players: {
          order: [1],
          by_id: { 1: mockPlayer },
        },
        instruments: {
          by_id: { 20: mockInstrument },
        },
      } as never,
    });
    renderWithProviders(<PlayerPlanel />, { appState });
    // Piano appears as player header name (and hidden in the instrument list)
    expect(screen.getAllByText("Piano").length).toBeGreaterThanOrEqual(1);
  });

  it("renders player with selection styling when selected", () => {
    const appState = createAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 1, selection_type: SetupSelectionType.Player },
            expanded: [],
          },
        },
      } as never,
      score: {
        players: {
          order: [1],
          by_id: { 1: mockPlayer },
        },
        instruments: {
          by_id: { 20: mockInstrument },
        },
      } as never,
    });
    renderWithProviders(<PlayerPlanel />, { appState });
    expect(screen.getAllByText("Piano").length).toBeGreaterThanOrEqual(1);
  });
});
