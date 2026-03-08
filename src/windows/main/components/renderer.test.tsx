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
});
