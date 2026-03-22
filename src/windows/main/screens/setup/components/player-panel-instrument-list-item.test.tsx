import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { PlayerPanelInstrumentListItem } from "./player-panel-instrument-list-item";
import {
  Instrument,
  InstrumentType,
  PlaybackAssignmentMode,
} from "../../../../../services/types";

vi.mock("../../../../../services/actions", () => ({
  actions: {
    score: {
      player: {
        unassignInstrument: vi.fn().mockResolvedValue(undefined),
      },
      instrument: {
        remove: vi.fn().mockResolvedValue(undefined),
      },
    },
  },
}));

import { actions } from "../../../../../services/actions";

const mockInstrument: Instrument = {
  id: 20,
  definition_id: "violin",
  instrument_type: InstrumentType.Melodic,
  long_name: "Violin",
  short_name: "Vln.",
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

const countedInstrument: Instrument = {
  ...mockInstrument,
  id: 21,
  long_name: "Violin",
  count: "I",
};

describe("PlayerPanelInstrumentListItem", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders the instrument long name", () => {
    render(
      <PlayerPanelInstrumentListItem
        playerId={1}
        instrument={mockInstrument}
        isSelected={false}
      />,
    );
    expect(screen.getByText("Violin")).toBeInTheDocument();
  });

  it("renders instrument name with count when count is set", () => {
    render(
      <PlayerPanelInstrumentListItem
        playerId={1}
        instrument={countedInstrument}
        isSelected={false}
      />,
    );
    expect(screen.getByText("Violin I")).toBeInTheDocument();
  });

  it("does not render delete button when not selected", () => {
    render(
      <PlayerPanelInstrumentListItem
        playerId={1}
        instrument={mockInstrument}
        isSelected={false}
      />,
    );
    expect(screen.queryByRole("button")).toBeNull();
  });

  it("renders delete button when selected", () => {
    render(
      <PlayerPanelInstrumentListItem
        playerId={1}
        instrument={mockInstrument}
        isSelected={true}
      />,
    );
    expect(screen.getByRole("button")).toBeInTheDocument();
  });

  it("calls unassignInstrument and remove when delete button clicked", async () => {
    render(
      <PlayerPanelInstrumentListItem
        playerId={1}
        instrument={mockInstrument}
        isSelected={true}
      />,
    );
    fireEvent.click(screen.getByRole("button"));
    await vi.runAllTimersAsync().catch(() => {});
    expect(actions.score.player.unassignInstrument).toHaveBeenCalledWith(1, 20);
    expect(actions.score.instrument.remove).toHaveBeenCalledWith(20);
  });

  it("applies selected background class when selected", () => {
    const { container } = render(
      <PlayerPanelInstrumentListItem
        playerId={1}
        instrument={mockInstrument}
        isSelected={true}
      />,
    );
    expect(container.firstChild).toHaveClass("bg-white");
  });

  it("applies non-selected background class when not selected", () => {
    const { container } = render(
      <PlayerPanelInstrumentListItem
        playerId={1}
        instrument={mockInstrument}
        isSelected={false}
      />,
    );
    expect(container.firstChild).toHaveClass("bg-neutral-700");
  });
});
