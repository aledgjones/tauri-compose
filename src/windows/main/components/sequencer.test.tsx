import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import {
  renderWithProviders,
  createAppState,
} from "../../../services/test-utils";
import { SequencerToolbar, Sequencer } from "./sequencer";
import { NoteDuration, Tool } from "../../../services/types";

vi.mock("../../../services/actions", () => ({
  actions: {
    ui: {
      setActiveFlowId: vi.fn(),
      sequencer: {
        setTool: vi.fn(),
        setSnap: vi.fn(),
        setCoords: vi.fn(),
        setInstrument: vi.fn(),
        setTrack: vi.fn(),
      },
    },
  },
}));

import { actions } from "../../../services/actions";

const baseToolbarProps = {
  tool: Tool.Select,
  flowId: null,
  flowOptions: [],
  instrumentId: null,
  instruments: [],
  trackId: null,
  tracks: [],
  snapValue: NoteDuration.Eighth as NoteDuration,
};

describe("SequencerToolbar", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders without crashing", () => {
    const { container } = renderWithProviders(
      <SequencerToolbar {...baseToolbarProps} />,
    );
    expect(container).toBeInTheDocument();
  });

  it("renders all four tool buttons", () => {
    renderWithProviders(<SequencerToolbar {...baseToolbarProps} />);
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(4);
  });

  it("calls setTool(Select) when Select button clicked", () => {
    renderWithProviders(<SequencerToolbar {...baseToolbarProps} />);
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[0]);
    expect(actions.ui.sequencer.setTool).toHaveBeenCalledWith(Tool.Select);
  });

  it("calls setTool(Write) when Write button clicked", () => {
    renderWithProviders(<SequencerToolbar {...baseToolbarProps} />);
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[1]);
    expect(actions.ui.sequencer.setTool).toHaveBeenCalledWith(Tool.Write);
  });

  it("calls setTool(Slice) when Slice button clicked", () => {
    renderWithProviders(<SequencerToolbar {...baseToolbarProps} />);
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[2]);
    expect(actions.ui.sequencer.setTool).toHaveBeenCalledWith(Tool.Slice);
  });

  it("calls setTool(Erase) when Erase button clicked", () => {
    renderWithProviders(<SequencerToolbar {...baseToolbarProps} />);
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[3]);
    expect(actions.ui.sequencer.setTool).toHaveBeenCalledWith(Tool.Erase);
  });

  it("highlights the active tool button", () => {
    renderWithProviders(
      <SequencerToolbar {...baseToolbarProps} tool={Tool.Write} />,
    );
    // Write is index 1 — isSelected adds bg-sky-500
    const buttons = screen.getAllByRole("button");
    expect(buttons[1].className).toContain("bg-sky-500");
  });

  it("renders instrument select when instrumentId and instruments provided", () => {
    renderWithProviders(
      <SequencerToolbar
        {...baseToolbarProps}
        instrumentId={1}
        instruments={[{ value: 1, label: "Piano" }]}
      />,
    );
    expect(screen.getByText("Piano")).toBeInTheDocument();
  });

  it("renders track select when trackId and tracks provided", () => {
    renderWithProviders(
      <SequencerToolbar
        {...baseToolbarProps}
        trackId={5}
        tracks={[{ value: 5, label: "Voice 1" }]}
      />,
    );
    expect(screen.getByText("Voice 1")).toBeInTheDocument();
  });

  it("does not render instrument select when instrumentId is null", () => {
    renderWithProviders(
      <SequencerToolbar
        {...baseToolbarProps}
        instrumentId={null}
        instruments={[{ value: 1, label: "Piano" }]}
      />,
    );
    expect(screen.queryByText("Piano")).toBeNull();
  });

  it("renders a flow selector when multiple flow options exist", () => {
    renderWithProviders(
      <SequencerToolbar
        {...baseToolbarProps}
        flowId={2}
        flowOptions={[
          { value: 1, label: "1. Flow A" },
          { value: 2, label: "2. Flow B" },
        ]}
      />,
    );

    expect(
      screen.getByRole("button", { name: "2. Flow B" }),
    ).toBeInTheDocument();
  });
});

describe("Sequencer", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders 'no flows' message when flowId is null", () => {
    renderWithProviders(
      <Sequencer flowId={null} octaves={10} instruments={[]} />,
    );
    expect(
      screen.getByText("You don't have any flows yet..."),
    ).toBeInTheDocument();
  });

  it("renders 'no flows' message when flow not in state", () => {
    const appState = createAppState({
      score: { flows: { order: [], by_id: {} } } as never,
    });
    renderWithProviders(
      <Sequencer flowId={99} octaves={10} instruments={[]} />,
      { appState },
    );
    expect(
      screen.getByText("You don't have any flows yet..."),
    ).toBeInTheDocument();
  });

  it("renders 'no instruments' message when instrumentId is null", () => {
    const flow = {
      id: 1,
      title: "Flow 1",
      player_ids: [],
      length: 1024,
      master_id: 10,
      staves: { by_id: {} },
    };
    const appState = createAppState({
      score: { flows: { order: [1], by_id: { 1: flow } } } as never,
    });
    renderWithProviders(
      <Sequencer
        flowId={1}
        instrumentId={null}
        trackId={null}
        octaves={10}
        instruments={[]}
      />,
      { appState },
    );
    expect(
      screen.getByText("You don't have any instruments yet..."),
    ).toBeInTheDocument();
  });

  it("renders 'not in flow' message when trackId is null", () => {
    const flow = {
      id: 1,
      title: "Flow 1",
      player_ids: [],
      length: 1024,
      master_id: 10,
      staves: { by_id: {} },
    };
    const appState = createAppState({
      score: { flows: { order: [1], by_id: { 1: flow } } } as never,
    });
    renderWithProviders(
      <Sequencer
        flowId={1}
        instrumentId={5}
        trackId={null}
        octaves={10}
        instruments={[]}
      />,
      { appState },
    );
    expect(
      screen.getByText("This instrument is not included in this flow..."),
    ).toBeInTheDocument();
  });

  it("renders 'not in flow' when track not found in state", () => {
    const flow = {
      id: 1,
      title: "Flow 1",
      player_ids: [],
      length: 1024,
      master_id: 10,
      staves: { by_id: {} },
    };
    const appState = createAppState({
      score: {
        flows: { order: [1], by_id: { 1: flow } },
        tracks: { by_id: {} },
      } as never,
    });
    renderWithProviders(
      <Sequencer
        flowId={1}
        instrumentId={5}
        trackId={99}
        octaves={10}
        instruments={[]}
      />,
      { appState },
    );
    expect(
      screen.getByText("This instrument is not included in this flow..."),
    ).toBeInTheDocument();
  });
});
