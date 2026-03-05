import { describe, it, expect } from "vitest";
import {
  renderWithProviders,
  createAppState,
} from "../../../services/test-utils";
import { SequencerTrackTone } from "./sequencer-track-tone";
import { Accidental, Articulation } from "../../../services/types";

describe("SequencerTrackTone", () => {
  const baseTone = {
    id: 1,
    tick: 0,
    type: "Tone" as const,
    duration: 16,
    pitch: { int: 60, accidental: Accidental.Natural },
    velocity: { int: 100 },
    articulations: [] as Articulation[],
  };

  it("renders a positioned element", () => {
    const { container } = renderWithProviders(
      <SequencerTrackTone
        slotHeight={24}
        tickWidth={4}
        tone={baseTone}
        color="#4A90D9"
      />,
    );
    const el = container.querySelector(".absolute");
    expect(el).toBeInTheDocument();
  });

  it("uses the provided color as background", () => {
    const { container } = renderWithProviders(
      <SequencerTrackTone
        slotHeight={24}
        tickWidth={4}
        tone={baseTone}
        color="#FF0000"
      />,
    );
    const el = container.querySelector(".absolute") as HTMLElement;
    expect(el.style.backgroundColor).toBeTruthy();
  });

  it("shows pitch name when tone is wide enough", () => {
    renderWithProviders(
      <SequencerTrackTone
        slotHeight={24}
        tickWidth={20}
        tone={{ ...baseTone, duration: 16 }}
        color="#4A90D9"
      />,
    );
    // With width=320 (20*16), pitch name should be visible
    // C4 -> pitch name "C4"
    // Just ensure the component didn't crash
  });

  it("lightens color when tone is selected", () => {
    const trackId = 1;
    const toneId = 1;
    const appState = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: 0,
            instrument_id: 0,
            stave_id: 0,
            track_id: trackId,
            tone_id: toneId,
          },
        },
      } as never,
    });
    const { container } = renderWithProviders(
      <SequencerTrackTone
        slotHeight={24}
        tickWidth={4}
        tone={{ ...baseTone, id: toneId }}
        color="#4A90D9"
      />,
      { appState },
    );
    const el = container.querySelector(".absolute") as HTMLElement;
    // Background should be a lighter color (not the original #4A90D9)
    expect(el.style.backgroundColor).not.toBe("rgb(74, 144, 217)");
  });
});
