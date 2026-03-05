import { describe, it, expect } from "vitest";
import { PlaybackLine } from "./playback-line";
import { renderWithProviders } from "../../../services/test-utils";
import type { TickConfig } from "../hooks/use-ticks";

const ticks: TickConfig = {
  tickWidth: 2,
  entries: [],
  length: 128,
};

describe("PlaybackLine", () => {
  it("renders without crashing", () => {
    const { container } = renderWithProviders(
      <PlaybackLine ticks={ticks} height={200} />,
      { transportState: { playing: false, current_tick: 0, tempo: 120 } },
    );
    expect(container.firstChild).not.toBeNull();
  });

  it("positions the line at x=0 when current_tick is 0", () => {
    const { container } = renderWithProviders(
      <PlaybackLine ticks={ticks} height={200} />,
      { transportState: { playing: false, current_tick: 0, tempo: 120 } },
    );
    const line = container.firstChild as HTMLElement;
    expect(line.style.transform).toBe("translateX(0px)");
  });

  it("positions the line based on current_tick × tickWidth", () => {
    // tick 10, tickWidth 2 → x = 20px
    const { container } = renderWithProviders(
      <PlaybackLine ticks={ticks} height={200} />,
      { transportState: { playing: true, current_tick: 10, tempo: 120 } },
    );
    const line = container.firstChild as HTMLElement;
    expect(line.style.transform).toBe("translateX(20px)");
  });

  it("applies the height prop via inline style", () => {
    const { container } = renderWithProviders(
      <PlaybackLine ticks={ticks} height={400} />,
      { transportState: { playing: false, current_tick: 0, tempo: 120 } },
    );
    const line = container.firstChild as HTMLElement;
    expect(line.style.height).toBe("400px");
  });
});
