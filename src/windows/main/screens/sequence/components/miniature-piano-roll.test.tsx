import { describe, it, expect } from "vitest";
import {
  renderWithProviders,
  createAppState,
} from "../../../../../services/test-utils";
import { MiniaturePianoRoll } from "./miniature-piano-roll";
import {
  Flow,
  Instrument,
  InstrumentType,
} from "../../../../../services/types";

const makeFlow = (overrides = {}): Flow => ({
  id: 1,
  title: "Test",
  player_ids: [],
  length: 512,
  master_id: 10,
  staves: { by_id: {} },
  ...overrides,
});

const makeInstrument = (overrides: Partial<Instrument> = {}): Instrument => ({
  id: 5,
  definition_id: "piano",
  instrument_type: InstrumentType.Melodic,
  long_name: "Piano",
  short_name: "Pno.",
  stave_ids: [],
  count: null,
  assigned_plugin_id: null,
  volume: 100,
  pan: 64,
  mute: false,
  solo: false,
  ...overrides,
});

const baseTicks = {
  length: 512,
  tickWidth: 0.5,
  entries: Array.from({ length: 512 }, (_, i) => ({
    type: "sub" as const,
    x: i * 0.5,
  })),
};

describe("MiniaturePianoRoll", () => {
  it("renders without crashing when instrument has no staves", () => {
    const { container } = renderWithProviders(
      <MiniaturePianoRoll
        instrument={makeInstrument()}
        flow={makeFlow()}
        ticks={baseTicks}
        color="#4A90D9"
        isDisabled={false}
      />,
    );
    expect(container.firstChild).toBeInTheDocument();
  });

  it("applies opacity-50 when isDisabled=true", () => {
    const { container } = renderWithProviders(
      <MiniaturePianoRoll
        instrument={makeInstrument()}
        flow={makeFlow()}
        ticks={baseTicks}
        color="#4A90D9"
        isDisabled={true}
      />,
    );
    expect(container.firstChild).toHaveClass("opacity-50");
  });

  it("does not apply opacity-50 when isDisabled=false", () => {
    const { container } = renderWithProviders(
      <MiniaturePianoRoll
        instrument={makeInstrument()}
        flow={makeFlow()}
        ticks={baseTicks}
        color="#4A90D9"
        isDisabled={false}
      />,
    );
    expect(container.firstChild).not.toHaveClass("opacity-50");
  });

  it("renders bar markers from tick entries", () => {
    const ticksWithBar = {
      ...baseTicks,
      entries: [
        { type: "bar" as const, x: 0 },
        { type: "beat" as const, x: 100 },
        { type: "sub" as const, x: 200 },
      ],
    };
    const { container } = renderWithProviders(
      <MiniaturePianoRoll
        instrument={makeInstrument()}
        flow={makeFlow()}
        ticks={ticksWithBar}
        color="#4A90D9"
        isDisabled={false}
      />,
    );
    // Bar dividers rendered as bg-neutral-800 divs
    expect(
      container.querySelectorAll(".bg-neutral-800").length,
    ).toBeGreaterThanOrEqual(1);
  });

  it("renders tones from tracks in state", () => {
    const staveId = 20;
    const trackId = 30;
    const flow = makeFlow({
      staves: {
        by_id: {
          [staveId]: { id: staveId, track_ids: [trackId] },
        },
      },
    });
    const instrument = makeInstrument({ stave_ids: [staveId] });
    const appState = createAppState({
      score: {
        tracks: {
          by_id: {
            [trackId]: {
              id: trackId,
              entries: {
                by_tick: {},
                by_id: {
                  1: {
                    id: 1,
                    type: "Tone",
                    tick: 0,
                    duration: 32,
                    pitch: { int: 60, accidental: 0 },
                    velocity: { int: 100 },
                    articulations: [],
                  },
                  2: {
                    id: 2,
                    type: "Tone",
                    tick: 64,
                    duration: 32,
                    pitch: { int: 62, accidental: 0 },
                    velocity: { int: 100 },
                    articulations: [],
                  },
                },
              },
            },
          },
        },
      } as never,
    });
    const { container } = renderWithProviders(
      <MiniaturePianoRoll
        instrument={instrument}
        flow={flow}
        ticks={baseTicks}
        color="#FF0000"
        isDisabled={false}
      />,
      { appState },
    );
    // 2 tones + 2 active range background highlights → opacity-80 and opacity-20 divs
    const opacityDivs = container.querySelectorAll(".opacity-80");
    expect(opacityDivs.length).toBe(2);
  });

  it("merges overlapping tone ranges into single active-range highlight", () => {
    const staveId = 20;
    const trackId = 30;
    const flow = makeFlow({
      staves: {
        by_id: {
          [staveId]: { id: staveId, track_ids: [trackId] },
        },
      },
    });
    const instrument = makeInstrument({ stave_ids: [staveId] });
    // Two overlapping tones
    const appState = createAppState({
      score: {
        tracks: {
          by_id: {
            [trackId]: {
              id: trackId,
              entries: {
                by_tick: {},
                by_id: {
                  1: {
                    id: 1,
                    type: "Tone",
                    tick: 0,
                    duration: 64,
                    pitch: { int: 60, accidental: 0 },
                    velocity: { int: 100 },
                    articulations: [],
                  },
                  2: {
                    id: 2,
                    type: "Tone",
                    tick: 32,
                    duration: 64,
                    pitch: { int: 64, accidental: 0 },
                    velocity: { int: 100 },
                    articulations: [],
                  },
                },
              },
            },
          },
        },
      } as never,
    });
    const { container } = renderWithProviders(
      <MiniaturePianoRoll
        instrument={instrument}
        flow={flow}
        ticks={baseTicks}
        color="#FF0000"
        isDisabled={false}
      />,
      { appState },
    );
    // Merged into 1 background highlight
    const bgHighlights = container.querySelectorAll(".opacity-20");
    expect(bgHighlights.length).toBe(1);
  });
});
