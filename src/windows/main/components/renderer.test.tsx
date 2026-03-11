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
});
