import { describe, expect, it, vi, beforeEach } from "vitest";
import { fireEvent, render, waitFor } from "@testing-library/react";
import { createWrapper, createAppState } from "../../../services/test-utils";
import { Renderer } from "./renderer";

vi.mock("../../../services/actions", () => ({
  actions: {
    ui: {
      renderer: {
        setBounds: vi.fn().mockResolvedValue(undefined),
        setPan: vi.fn().mockResolvedValue([0, 0]),
        setZoom: vi.fn().mockResolvedValue([100, 0, 0]),
      },
      sequencer: {
        selectTone: vi.fn().mockResolvedValue(undefined),
        selectRest: vi.fn().mockResolvedValue(undefined),
        selectTempo: vi.fn().mockResolvedValue(undefined),
        selectTimeSignature: vi.fn().mockResolvedValue(undefined),
        selectKeySignature: vi.fn().mockResolvedValue(undefined),
        selectClef: vi.fn().mockResolvedValue(undefined),
        selectPlayingTechnique: vi.fn().mockResolvedValue(undefined),
        selectTuplet: vi.fn().mockResolvedValue(undefined),
        clearSelection: vi.fn().mockResolvedValue(undefined),
      },
    },
  },
}));

vi.mock("../../../services/hit-test", () => ({
  hitTest: vi.fn(),
}));

vi.mock("../hooks/use-keyboard-shortcuts", () => ({
  useKeyboardShortcuts: vi.fn(),
}));

vi.mock("../../../ui/popover", () => ({
  Popover: () => null,
}));

import { actions } from "../../../services/actions";
import { hitTest } from "../../../services/hit-test";

class ResizeObserverMock {
  observe() {}
  disconnect() {}
}

describe("Renderer", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("ResizeObserver", ResizeObserverMock);
  });

  it("dispatches tempo selection when hit testing resolves a tempo target", async () => {
    vi.mocked(hitTest).mockResolvedValueOnce({
      type: "Tempo",
      flow_id: 4,
      tempo_id: 8,
      tick: 64,
    });

    const { container } = render(<Renderer />, {
      wrapper: createWrapper({ appState: createAppState() }),
    });

    const overlay = container.querySelector('[data-testid="renderer"] > div');
    expect(overlay).not.toBeNull();

    fireEvent.pointerDown(overlay!, { clientX: 10, clientY: 10 });

    await waitFor(() => {
      expect(actions.ui.sequencer.selectTempo).toHaveBeenCalledWith(8, 4, 64);
    });
  });

  it("dispatches tone selection when hit testing resolves a tone target in another visible flow", async () => {
    vi.mocked(hitTest).mockResolvedValueOnce({
      type: "Tone",
      flow_id: 20,
      track_id: 7,
      stave_id: 8,
      instrument_id: 9,
      tone_id: 10,
    });

    const { container } = render(<Renderer />, {
      wrapper: createWrapper({ appState: createAppState() }),
    });

    const overlay = container.querySelector('[data-testid="renderer"] > div');
    fireEvent.pointerDown(overlay!, { clientX: 10, clientY: 10 });

    await waitFor(() => {
      expect(actions.ui.sequencer.selectTone).toHaveBeenCalledWith(
        10,
        7,
        8,
        20,
        9,
        undefined,
      );
    });
  });

  it("forwards tuplet ownership when hit testing resolves a tuplet-owned tone target", async () => {
    vi.mocked(hitTest).mockResolvedValueOnce({
      type: "Tone",
      flow_id: 20,
      track_id: 7,
      stave_id: 8,
      instrument_id: 9,
      tone_id: 10,
      tuplet_id: 44,
    });

    const { container } = render(<Renderer />, {
      wrapper: createWrapper({ appState: createAppState() }),
    });

    const overlay = container.querySelector('[data-testid="renderer"] > div');
    fireEvent.pointerDown(overlay!, { clientX: 10, clientY: 10 });

    await waitFor(() => {
      expect(actions.ui.sequencer.selectTone).toHaveBeenCalledWith(
        10,
        7,
        8,
        20,
        9,
        44,
      );
    });
  });

  it("dispatches rest selection when hit testing resolves a rest target in another visible flow", async () => {
    vi.mocked(hitTest).mockResolvedValueOnce({
      type: "Rest",
      flow_id: 30,
      track_id: 11,
      stave_id: 12,
      instrument_id: 13,
      tick: 96,
    });

    const { container } = render(<Renderer />, {
      wrapper: createWrapper({ appState: createAppState() }),
    });

    const overlay = container.querySelector('[data-testid="renderer"] > div');
    fireEvent.pointerDown(overlay!, { clientX: 10, clientY: 10 });

    await waitFor(() => {
      expect(actions.ui.sequencer.selectRest).toHaveBeenCalledWith(
        96,
        11,
        12,
        30,
        13,
        undefined,
      );
    });
  });

  it("forwards tuplet ownership when hit testing resolves a tuplet-owned rest target", async () => {
    vi.mocked(hitTest).mockResolvedValueOnce({
      type: "Rest",
      flow_id: 30,
      track_id: 11,
      stave_id: 12,
      instrument_id: 13,
      tick: 96,
      tuplet_id: 55,
    });

    const { container } = render(<Renderer />, {
      wrapper: createWrapper({ appState: createAppState() }),
    });

    const overlay = container.querySelector('[data-testid="renderer"] > div');
    fireEvent.pointerDown(overlay!, { clientX: 10, clientY: 10 });

    await waitFor(() => {
      expect(actions.ui.sequencer.selectRest).toHaveBeenCalledWith(
        96,
        11,
        12,
        30,
        13,
        55,
      );
    });
  });

  it("dispatches time signature selection when hit testing resolves a time signature target", async () => {
    vi.mocked(hitTest).mockResolvedValueOnce({
      type: "TimeSignature",
      flow_id: 4,
      time_signature_id: 8,
      tick: 64,
    });

    const { container } = render(<Renderer />, {
      wrapper: createWrapper({ appState: createAppState() }),
    });

    const overlay = container.querySelector('[data-testid="renderer"] > div');
    fireEvent.pointerDown(overlay!, { clientX: 10, clientY: 10 });

    await waitFor(() => {
      expect(actions.ui.sequencer.selectTimeSignature).toHaveBeenCalledWith(
        4,
        8,
        64,
      );
    });
  });

  it("dispatches key signature selection when hit testing resolves a key signature target", async () => {
    vi.mocked(hitTest).mockResolvedValueOnce({
      type: "KeySignature",
      flow_id: 4,
      key_signature_id: 9,
      tick: 64,
    });

    const { container } = render(<Renderer />, {
      wrapper: createWrapper({ appState: createAppState() }),
    });

    const overlay = container.querySelector('[data-testid="renderer"] > div');
    fireEvent.pointerDown(overlay!, { clientX: 10, clientY: 10 });

    await waitFor(() => {
      expect(actions.ui.sequencer.selectKeySignature).toHaveBeenCalledWith(
        4,
        9,
        64,
      );
    });
  });

  it("dispatches clef selection when hit testing resolves a clef target", async () => {
    vi.mocked(hitTest).mockResolvedValueOnce({
      type: "Clef",
      flow_id: 4,
      stave_id: 3,
      clef_id: 10,
      tick: 64,
    });

    const { container } = render(<Renderer />, {
      wrapper: createWrapper({ appState: createAppState() }),
    });

    const overlay = container.querySelector('[data-testid="renderer"] > div');
    fireEvent.pointerDown(overlay!, { clientX: 10, clientY: 10 });

    await waitFor(() => {
      expect(actions.ui.sequencer.selectClef).toHaveBeenCalledWith(
        4,
        3,
        10,
        64,
      );
    });
  });

  it("dispatches playing-technique selection when hit testing resolves a playing-technique target", async () => {
    vi.mocked(hitTest).mockResolvedValueOnce({
      type: "PlayingTechnique",
      flow_id: 4,
      stave_id: 3,
      playing_technique_id: 10,
      tick: 64,
    });

    const { container } = render(<Renderer />, {
      wrapper: createWrapper({ appState: createAppState() }),
    });

    const overlay = container.querySelector('[data-testid="renderer"] > div');
    fireEvent.pointerDown(overlay!, { clientX: 10, clientY: 10 });

    await waitFor(() => {
      expect(actions.ui.sequencer.selectPlayingTechnique).toHaveBeenCalledWith(
        4,
        3,
        10,
        64,
      );
    });
  });

  it("dispatches tuplet selection when hit testing resolves a tuplet target", async () => {
    vi.mocked(hitTest).mockResolvedValueOnce({
      type: "Tuplet",
      flow_id: 6,
      instrument_id: 7,
      stave_id: 8,
      track_id: 9,
      tuplet_id: 10,
      tick: 64,
    });

    const { container } = render(<Renderer />, {
      wrapper: createWrapper({ appState: createAppState() }),
    });

    const overlay = container.querySelector('[data-testid="renderer"] > div');
    fireEvent.pointerDown(overlay!, { clientX: 10, clientY: 10 });

    await waitFor(() => {
      expect(actions.ui.sequencer.selectTuplet).toHaveBeenCalledWith(
        6,
        7,
        8,
        9,
        10,
        64,
      );
    });
  });

  it("routes wheel scrolling through the shared renderer pan path", async () => {
    vi.mocked(hitTest).mockResolvedValue(null);

    const { container } = render(<Renderer />, {
      wrapper: createWrapper({ appState: createAppState() }),
    });

    const overlay = container.querySelector('[data-testid="renderer"] > div');
    fireEvent.wheel(overlay!, { deltaX: 24, deltaY: -12 });

    await waitFor(() => {
      expect(actions.ui.renderer.setPan).toHaveBeenCalledWith(-24, 12);
    });
  });

  it("routes pointer-drag panning through the shared renderer pan path", async () => {
    vi.mocked(hitTest).mockResolvedValue(null);

    const { container } = render(<Renderer />, {
      wrapper: createWrapper({ appState: createAppState() }),
    });

    const overlay = container.querySelector('[data-testid="renderer"] > div');
    fireEvent.pointerDown(overlay!, { clientX: 50, clientY: 70 });

    await waitFor(() => {
      expect(hitTest).toHaveBeenCalled();
    });

    fireEvent.pointerMove(document, { clientX: 82, clientY: 58 });

    await waitFor(() => {
      expect(actions.ui.renderer.setPan).toHaveBeenCalledWith(32, -12);
    });
  });

  it("clears the prior selection when a pointer down misses and no drag occurs", async () => {
    vi.mocked(hitTest).mockResolvedValue(null);

    const { container } = render(<Renderer />, {
      wrapper: createWrapper({ appState: createAppState() }),
    });

    const overlay = container.querySelector('[data-testid="renderer"] > div');
    fireEvent.pointerDown(overlay!, { clientX: 25, clientY: 35 });

    await waitFor(() => {
      expect(hitTest).toHaveBeenCalled();
    });

    fireEvent.pointerUp(document);

    await waitFor(() => {
      expect(actions.ui.sequencer.clearSelection).toHaveBeenCalledTimes(1);
    });
  });
});
