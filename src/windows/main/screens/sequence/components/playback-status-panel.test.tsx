import { describe, it, expect, vi, beforeEach } from "vitest";
import { fireEvent, screen } from "@testing-library/react";
import { renderWithProviders } from "../../../../../services/test-utils";
import {
  Instrument,
  InstrumentType,
  PlaybackAssignmentMode,
  PlaybackAssignmentStatus,
  ResolvedPlaybackAssignment,
} from "../../../../../services/types";
import { PlaybackStatusPanel } from "./playback-status-panel";

const instrument: Instrument = {
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

const readyAssignment: ResolvedPlaybackAssignment = {
  status: PlaybackAssignmentStatus.Ready,
  mode: PlaybackAssignmentMode.Direct,
  effective_plugin_id: "vst3:grand-piano",
  target: null,
  shared_instance_id: null,
};

const unavailableAssignment: ResolvedPlaybackAssignment = {
  status: PlaybackAssignmentStatus.Unassigned,
  mode: PlaybackAssignmentMode.Unassigned,
  effective_plugin_id: null,
  target: null,
  shared_instance_id: null,
};

describe("PlaybackStatusPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders selected instrument playback context when ready", () => {
    renderWithProviders(
      <PlaybackStatusPanel
        instrument={instrument}
        assignment={readyAssignment}
      />,
      {
        pluginInventoryState: {
          entries: [
            {
              id: "vst3:grand-piano",
              name: "Grand Piano",
              bundle_path: "/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3",
              binary_path:
                "/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3/Contents/MacOS/Grand Piano",
              source_root: "/Library/Audio/Plug-Ins/VST3",
              availability: "available",
              validation_failure: null,
            },
          ],
        },
      },
    );

    expect(screen.getByText("Selected instrument")).toBeInTheDocument();
    expect(screen.getByText("Violin")).toBeInTheDocument();
    expect(
      screen.getByRole("status", { name: "Playback status: Ready" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Grand Piano")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /focus row assignment/i }),
    ).toBeNull();
  });

  it("uses one primary in-score recovery handoff for non-ready instruments", () => {
    const assignmentButton = document.createElement("button");
    assignmentButton.textContent = "Assign plugin";
    assignmentButton.scrollIntoView = vi.fn();

    const wrapper = document.createElement("div");
    wrapper.setAttribute("data-assignment-select-for", "5");
    wrapper.appendChild(assignmentButton);
    document.body.appendChild(wrapper);

    renderWithProviders(
      <PlaybackStatusPanel
        instrument={instrument}
        assignment={unavailableAssignment}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Focus row assignment for Violin" }),
    );

    expect(assignmentButton.scrollIntoView).toHaveBeenCalled();
    expect(assignmentButton).toHaveFocus();

    wrapper.remove();
  });
});
