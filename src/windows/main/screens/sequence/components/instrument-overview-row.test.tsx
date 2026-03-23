import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import { InstrumentOverviewRow } from "./instrument-overview-row";
import { renderWithProviders } from "../../../../../services/test-utils";
import {
  Instrument,
  InstrumentType,
  PlaybackAssignmentMode,
  PlaybackAssignmentStatus,
} from "../../../../../services/types";

vi.mock("../../../../../services/actions", () => ({
  actions: {
    score: {
      instrument: {
        assignPlugin: vi.fn().mockResolvedValue(undefined),
        setSolo: vi.fn().mockResolvedValue(undefined),
        setMute: vi.fn().mockResolvedValue(undefined),
      },
    },
  },
}));

import { actions } from "../../../../../services/actions";

const mockInstrument: Instrument = {
  id: 5,
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

describe("InstrumentOverviewRow", () => {
  beforeEach(() => vi.clearAllMocks());

  const renderRow = (resolvedAssignment = {}) => {
    return renderWithProviders(
      <InstrumentOverviewRow
        instrument={mockInstrument}
        color="#ff0000"
        isDisabled={false}
      />,
      {
        appState: {
          resolved_playback_assignment_by_instrument_id: {
            5: {
              status: PlaybackAssignmentStatus.Unassigned,
              mode: PlaybackAssignmentMode.Unassigned,
              effective_plugin_id: null,
              target: null,
              shared_instance_id: null,
              ...resolvedAssignment,
            },
          },
        },
      },
    );
  };

  it("renders the instrument long name", () => {
    renderRow();
    expect(screen.getByText("Violin")).toBeInTheDocument();
  });

  it("renders an accessible unavailable readiness badge by default", () => {
    renderRow();

    expect(
      screen.getByRole("status", { name: "Playback status: Unavailable" }),
    ).toBeInTheDocument();
  });

  it("renders a ready badge when the backend assignment is ready", () => {
    renderRow({
      status: PlaybackAssignmentStatus.Ready,
      mode: PlaybackAssignmentMode.Direct,
      effective_plugin_id: "vst3:grand-piano",
    });

    expect(
      screen.getByRole("status", { name: "Playback status: Ready" }),
    ).toBeInTheDocument();
  });

  it("renders a needs-attention badge when the backend assignment requires recovery", () => {
    renderRow({
      status: PlaybackAssignmentStatus.NeedsAttention,
      mode: PlaybackAssignmentMode.Direct,
    });

    expect(
      screen.getByRole("status", { name: "Playback status: Needs attention" }),
    ).toBeInTheDocument();
  });

  it("renders solo and mute buttons when not disabled", () => {
    renderRow();
    const buttons = screen.getAllByRole("button");
    // solo + mute buttons
    expect(buttons.length).toBeGreaterThanOrEqual(2);
  });

  it("hides solo/mute controls when disabled (applies opacity-50 class)", () => {
    const { container } = renderWithProviders(
      <InstrumentOverviewRow
        instrument={mockInstrument}
        color="#ff0000"
        isDisabled={true}
      />,
    );
    expect(container.querySelector(".opacity-50")).toBeInTheDocument();
  });

  it("renders a disabled plugin select with 'No plugins available' placeholder when no plugins", () => {
    renderRow();
    expect(screen.getByText("No plugins available")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /no plugins available/i }),
    ).toBeDisabled();
  });

  it("calls setSolo when solo button clicked", () => {
    renderRow();
    const [soloBtn] = screen.getAllByRole("button");
    fireEvent.click(soloBtn);
    expect(actions.score.instrument.setSolo).toHaveBeenCalledWith(5, true);
  });

  it("calls setMute when mute button clicked", () => {
    renderRow();
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[1]);
    expect(actions.score.instrument.setMute).toHaveBeenCalledWith(5, true);
  });

  it("renders instrument count when count is set", () => {
    const withCount: Instrument = { ...mockInstrument, count: "I" };
    renderWithProviders(
      <InstrumentOverviewRow
        instrument={withCount}
        color="#ff0000"
        isDisabled={false}
      />,
    );
    expect(screen.getByText("I")).toBeInTheDocument();
  });

  it("applies opacity when disabled", () => {
    const { container } = renderWithProviders(
      <InstrumentOverviewRow
        instrument={mockInstrument}
        color="#ff0000"
        isDisabled={true}
      />,
    );
    expect(container.firstChild).toHaveClass("opacity-50");
  });
});
