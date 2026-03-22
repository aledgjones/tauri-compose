import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import {
  renderWithProviders,
  createAppState,
} from "../../../../../services/test-utils";
import { PlayerPanelItem } from "./player-panel-item";
import {
  Instrument,
  InstrumentType,
  PlaybackAssignmentMode,
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
        remove: vi.fn().mockResolvedValue(undefined),
        assignInstrument: vi.fn().mockResolvedValue(undefined),
        unassignPlayer: vi.fn().mockResolvedValue(undefined),
      },
      instrument: {
        create: vi.fn().mockResolvedValue(20),
        move: vi.fn().mockResolvedValue(undefined),
      },
      flow: {
        assignPlayer: vi.fn().mockResolvedValue(undefined),
        unassignPlayer: vi.fn().mockResolvedValue(undefined),
      },
      layout: {
        assignPlayer: vi.fn().mockResolvedValue(undefined),
        unassignPlayer: vi.fn().mockResolvedValue(undefined),
      },
    },
    ui: {
      screens: {
        setup: {
          select: vi.fn(),
          expand: vi.fn().mockResolvedValue(undefined),
          collapse: vi.fn().mockResolvedValue(undefined),
        },
      },
    },
  },
}));

import { actions } from "../../../../../services/actions";

const mockInstrument: Instrument = {
  id: 20,
  definition_id: "piano",
  instrument_type: InstrumentType.Melodic,
  long_name: "Piano",
  short_name: "Pno.",
  stave_ids: [],
  count: null,
  assignment_intent: {
    mode: PlaybackAssignmentMode.Unassigned,
    plugin_id: null,
    target: null,
    shared_instance_id: null,
  },
  volume: 100,
  pan: 64,
  mute: false,
  solo: false,
};

const soloPlayer: Player = {
  id: 1,
  player_type: PlayerType.Solo,
  instrument_ids: [20],
};

const emptyPlayer: Player = {
  id: 2,
  player_type: PlayerType.Solo,
  instrument_ids: [],
};

const sectionPlayer: Player = {
  id: 3,
  player_type: PlayerType.Section,
  instrument_ids: [20],
};

const makeAppState = (overrides = {}) =>
  createAppState({
    score: {
      players: { order: [1], by_id: { 1: soloPlayer } },
      instruments: { by_id: { 20: mockInstrument } },
    } as never,
    ...overrides,
  });

describe("PlayerPanelItem", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders the instrument name for a solo player with instrument", () => {
    renderWithProviders(<PlayerPanelItem player={soloPlayer} />, {
      appState: makeAppState(),
    });
    expect(screen.getAllByText("Piano").length).toBeGreaterThanOrEqual(1);
  });

  it("renders 'Empty-handed Player' for player with no instruments", () => {
    renderWithProviders(<PlayerPanelItem player={emptyPlayer} />, {
      appState: createAppState({
        score: {
          players: { order: [2], by_id: { 2: emptyPlayer } },
          instruments: { by_id: {} },
        } as never,
      }),
    });
    expect(screen.getByText("Empty-handed Player")).toBeInTheDocument();
  });

  it("calls select when an unselected player is pointer-downed", () => {
    renderWithProviders(<PlayerPanelItem player={soloPlayer} />, {
      appState: makeAppState(),
    });
    fireEvent.pointerDown(screen.getAllByText("Piano")[0]);
    expect(actions.ui.screens.setup.select).toHaveBeenCalledWith(
      SetupSelectionType.Player,
      1,
    );
  });

  it("does not call select when already selected player is pointer-downed", () => {
    const appState = makeAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 1, selection_type: SetupSelectionType.Player },
            expanded: [],
          },
        },
      } as never,
    });
    renderWithProviders(<PlayerPanelItem player={soloPlayer} />, { appState });
    fireEvent.pointerDown(screen.getAllByText("Piano")[0]);
    expect(actions.ui.screens.setup.select).not.toHaveBeenCalled();
  });

  it("renders delete and add buttons when player is selected and is solo type", () => {
    const appState = makeAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 1, selection_type: SetupSelectionType.Player },
            expanded: [],
          },
        },
      } as never,
    });
    renderWithProviders(<PlayerPanelItem player={soloPlayer} />, { appState });
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(2); // delete + add
  });

  it("calls player.remove when delete button is clicked", async () => {
    const appState = makeAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 1, selection_type: SetupSelectionType.Player },
            expanded: [],
          },
        },
      } as never,
    });
    renderWithProviders(<PlayerPanelItem player={soloPlayer} />, { appState });
    const buttons = screen.getAllByRole("button");
    // delete button is index 0 among the selection buttons
    fireEvent.click(buttons[0]);
    await vi.runAllTimersAsync().catch(() => {});
    expect(actions.score.player.remove).toHaveBeenCalledWith(1);
  });

  it("renders section player with group icon (no crash)", () => {
    const appState = createAppState({
      score: {
        players: { order: [3], by_id: { 3: sectionPlayer } },
        instruments: { by_id: { 20: mockInstrument } },
      } as never,
    });
    renderWithProviders(<PlayerPanelItem player={sectionPlayer} />, {
      appState,
    });
    expect(screen.getAllByText("Piano").length).toBeGreaterThanOrEqual(1);
  });

  it("renders expand/collapse chevron when not assignable", () => {
    renderWithProviders(<PlayerPanelItem player={soloPlayer} />, {
      appState: makeAppState(),
    });
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(1);
  });

  it("renders a checkbox when a Flow is selected (making player assignable)", () => {
    const flow = {
      id: 10,
      title: "F",
      player_ids: [],
      length: 128,
      master_id: 0,
      staves: { by_id: {} },
    };
    const appState = makeAppState({
      ui: {
        screens: {
          setup: {
            selection: { id: 10, selection_type: SetupSelectionType.Flow },
            expanded: [],
          },
        },
      } as never,
      score: {
        players: { order: [1], by_id: { 1: soloPlayer } },
        instruments: { by_id: { 20: mockInstrument } },
        flows: { order: [10], by_id: { 10: flow } },
      } as never,
    });
    renderWithProviders(<PlayerPanelItem player={soloPlayer} />, { appState });
    // Checkbox is rendered as a role=checkbox
    expect(screen.getByRole("checkbox")).toBeInTheDocument();
  });

  it("renders player name as 'A & B' for two instruments", () => {
    const instrA: Instrument = {
      ...mockInstrument,
      id: 21,
      long_name: "Violin",
      count: null,
    };
    const instrB: Instrument = {
      ...mockInstrument,
      id: 22,
      long_name: "Viola",
      count: null,
    };
    const multiPlayer: Player = {
      id: 4,
      player_type: PlayerType.Section,
      instrument_ids: [21, 22],
    };
    const appState = createAppState({
      score: {
        players: { order: [4], by_id: { 4: multiPlayer } },
        instruments: { by_id: { 21: instrA, 22: instrB } },
      } as never,
    });
    renderWithProviders(<PlayerPanelItem player={multiPlayer} />, { appState });
    expect(screen.getByText("Violin & Viola")).toBeInTheDocument();
  });

  it("renders player name with count suffix when count is set", () => {
    const instrWithCount: Instrument = { ...mockInstrument, count: "1" };
    const appState = createAppState({
      score: {
        players: { order: [1], by_id: { 1: soloPlayer } },
        instruments: { by_id: { 20: instrWithCount } },
      } as never,
    });
    renderWithProviders(<PlayerPanelItem player={soloPlayer} />, { appState });
    expect(screen.getAllByText("Piano 1").length).toBeGreaterThanOrEqual(1);
  });
});
