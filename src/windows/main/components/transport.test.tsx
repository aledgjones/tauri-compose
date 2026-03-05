import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import { Transport } from "./transport";
import { renderWithProviders } from "../../../services/test-utils";

vi.mock("../../../services/actions", () => ({
  actions: {
    transport: {
      play: vi.fn().mockResolvedValue(undefined),
      pause: vi.fn().mockResolvedValue(undefined),
      stop: vi.fn().mockResolvedValue(undefined),
      seek: vi.fn().mockResolvedValue(undefined),
    },
  },
}));

import { actions } from "../../../services/actions";

describe("Transport", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders play button when not playing", () => {
    renderWithProviders(<Transport />, {
      transportState: { playing: false, current_tick: 0, tempo: 120 },
    });
    // The play button renders an SVG icon — we check buttons are rendered
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(3); // skip-previous, play/pause, stop
  });

  it("renders current position display", () => {
    renderWithProviders(<Transport />, {
      transportState: { playing: false, current_tick: 0, tempo: 120 },
    });
    // Position 0 shows as "1.1.000"
    expect(screen.getByText("1.1.000")).toBeInTheDocument();
  });

  it("calls actions.transport.play when play button clicked (not playing)", () => {
    renderWithProviders(<Transport />, {
      transportState: { playing: false, current_tick: 0, tempo: 120 },
    });
    const buttons = screen.getAllByRole("button");
    // play/pause button is the 2nd button (index 1 after skip-previous)
    fireEvent.click(buttons[1]);
    expect(actions.transport.play).toHaveBeenCalledOnce();
  });

  it("calls actions.transport.pause when play button clicked while playing", () => {
    renderWithProviders(<Transport />, {
      transportState: { playing: true, current_tick: 0, tempo: 120 },
    });
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[1]);
    expect(actions.transport.pause).toHaveBeenCalledOnce();
  });

  it("calls actions.transport.stop when stop button clicked", () => {
    renderWithProviders(<Transport />, {
      transportState: { playing: false, current_tick: 0, tempo: 120 },
    });
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[2]); // stop button (index 2)
    expect(actions.transport.stop).toHaveBeenCalledOnce();
  });

  it("calls actions.transport.seek(0) when skip-previous button clicked", () => {
    renderWithProviders(<Transport />, {
      transportState: { playing: false, current_tick: 128, tempo: 120 },
    });
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[0]); // skip-previous is the first button
    expect(actions.transport.seek).toHaveBeenCalledWith(0);
  });

  it("formats a mid-flow tick position correctly", () => {
    // tick 160 = bar 2, beat 1, sub 000 in 4/4 (160 / 128 = 1 full bar, remainder 32 → beat 2)
    // bar=Math.floor(160/128)+1=2, beat=Math.floor((160%128)/32)+1=2, sub=160%32=0
    renderWithProviders(<Transport />, {
      transportState: { playing: false, current_tick: 160, tempo: 120 },
    });
    expect(screen.getByText("2.2.000")).toBeInTheDocument();
  });

  it("shows metronome toggle button", () => {
    renderWithProviders(<Transport />, {
      transportState: { playing: false, current_tick: 0, tempo: 120 },
    });
    const buttons = screen.getAllByRole("button");
    // Should have at least 4 buttons: skip-prev, play/pause, stop, metronome
    expect(buttons.length).toBeGreaterThanOrEqual(4);
  });
});
