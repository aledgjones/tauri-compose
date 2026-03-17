import { describe, it, expect, vi, beforeEach } from "vitest";
import { fireEvent, waitFor } from "@testing-library/react";
import {
  renderWithProviders,
  createAppState,
} from "../../../services/test-utils";
import { SequencerTrack } from "./sequencer-track";
import {
  Accidental,
  Articulation,
  Flow,
  NoteDuration,
  Tool,
  Track,
  Tuplet,
} from "../../../services/types";
import { KeyType } from "../hooks/use-slots";
import { actions } from "../../../services/actions";

vi.mock("../../../services/actions", () => ({
  actions: {
    score: {
      tone: {
        create: vi.fn().mockResolvedValue(1),
        remove: vi.fn().mockResolvedValue(undefined),
        slice: vi.fn().mockResolvedValue(2),
        setDuration: vi.fn().mockResolvedValue(undefined),
        setPitch: vi.fn().mockResolvedValue(undefined),
        shift: vi.fn().mockResolvedValue(undefined),
        setPositionAndDuration: vi.fn().mockResolvedValue(undefined),
        unifyArticulations: vi.fn().mockResolvedValue(undefined),
      },
    },
    ui: {
      sequencer: {
        selectTone: vi.fn().mockResolvedValue(undefined),
        selectRest: vi.fn().mockResolvedValue(undefined),
        selectTuplet: vi.fn().mockResolvedValue(undefined),
        clearSelection: vi.fn().mockResolvedValue(undefined),
      },
    },
    sampler: { audition: vi.fn() },
  },
}));

const makeFlow = (): Flow => ({
  id: 1,
  title: "Test",
  player_ids: [],
  length: 1024,
  master_id: 10,
  staves: {
    by_id: {
      20: {
        id: 20,
        lines: [1, 1, 1, 1, 1],
        master_id: 10,
        track_ids: [5],
      },
    },
  },
});

const makeTrack = (entries: Array<object> = []): Track =>
  ({
    id: 5,
    entries: {
      by_tick: {},
      by_id: Object.fromEntries(
        entries.map((entry, index) => {
          const id = index + 1;

          return [
            id,
            {
              id,
              type: "Tone",
              tick: index * 32,
              duration: 16,
              ...entry,
            },
          ];
        }),
      ),
    },
  }) as Track;

const makeTuplet = (overrides: Partial<Tuplet> = {}): Tuplet => ({
  id: 11,
  tick: 96,
  type: "Tuplet",
  note_count: 3,
  in_space_of: 2,
  base_note_value: NoteDuration.Eighth,
  groupings: [3],
  entries: { by_tick: {}, by_id: {} },
  ...overrides,
});

const makeTupletTone = (id: number, tick: number, duration = 16) => ({
  id,
  tick,
  type: "Tone" as const,
  duration,
  pitch: { int: 72, accidental: Accidental.Natural },
  velocity: { int: 100 },
  articulations: [] as Articulation[],
});

const makeTrackWithTuplet = (tuplet: Tuplet): Track =>
  ({
    id: 5,
    entries: {
      by_tick: {},
      by_id: {
        [tuplet.id]: tuplet,
      },
    },
  }) as Track;

const makeTicks = () => ({
  length: 512,
  tickWidth: 4,
  entries: Array.from({ length: 512 }, (_, i) => ({
    type: "sub" as const,
    x: i * 4,
  })),
});

const makeSlots = () => ({
  height: 1200,
  slotHeight: 10,
  pattern: Array.from({ length: 120 }, (_, i) => ({
    type:
      i % 12 === 1 ||
      i % 12 === 3 ||
      i % 12 === 6 ||
      i % 12 === 8 ||
      i % 12 === 10
        ? KeyType.Black
        : KeyType.White,
    height: 10,
    top: i * 10,
  })),
});

const defaultProps = {
  flow: makeFlow(),
  track: makeTrack(),
  ticks: makeTicks(),
  slots: makeSlots(),
  tool: Tool.Select,
  snapDuration: NoteDuration.Eighth,
  instrumentId: 1,
  color: "#4A90D9",
  audition: false,
  pluginId: null,
};

describe("SequencerTrack", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders without crashing", () => {
    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} />,
    );
    expect(container.firstChild).toBeInTheDocument();
  });

  it("renders slot background rows", () => {
    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} />,
    );
    const absolutes = container.querySelectorAll(".absolute");
    expect(absolutes.length).toBeGreaterThan(0);
  });

  it("renders tones inside the track", () => {
    const track = makeTrack([
      {
        pitch: { int: 60, accidental: Accidental.Natural },
        velocity: { int: 100 },
        articulations: [] as Articulation[],
      },
      {
        pitch: { int: 64, accidental: Accidental.Natural },
        velocity: { int: 100 },
        articulations: [] as Articulation[],
      },
    ]);
    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} track={track} />,
    );
    const absolutes = container.querySelectorAll(".absolute");
    expect(absolutes.length).toBeGreaterThan(2);
  });

  it("handles pointerDown with Write tool without crashing", () => {
    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} tool={Tool.Write} />,
    );
    const div = container.firstChild as HTMLElement;
    fireEvent.pointerDown(div, { clientX: 100, clientY: 200 });
  });

  it("writes tuplet tones through the track-level action seam with local timing context", async () => {
    const tuplet = makeTuplet({ tick: 96 });
    const { container } = renderWithProviders(
      <SequencerTrack
        {...defaultProps}
        tool={Tool.Write}
        track={makeTrackWithTuplet(tuplet)}
      />,
    );

    const target = container.querySelector(
      '[data-tuplet-id="11"]',
    ) as HTMLElement;
    fireEvent.pointerDown(target, { clientX: 440, clientY: 200 });

    await waitFor(() => {
      expect(actions.score.tone.create).toHaveBeenCalledWith(
        5,
        16,
        16,
        111,
        null,
        100,
        [],
        11,
      );
    });

    expect(actions.ui.sequencer.selectTone).toHaveBeenCalledWith(
      1,
      5,
      20,
      1,
      1,
      11,
    );
  });

  it("renders child tones inside a tuplet while keeping slot markers visible", () => {
    const tuplet = makeTuplet({
      tick: 96,
      entries: {
        by_tick: { 16: [101] },
        by_id: { 101: makeTupletTone(101, 16) },
      },
    });

    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} track={makeTrackWithTuplet(tuplet)} />,
    );

    expect(container.querySelector('[data-tone-id="101"]')).toBeInTheDocument();
    expect(
      container.querySelectorAll('[data-tuplet-slot-marker="11"]'),
    ).toHaveLength(4);
  });

  it("selects a child tuplet tone when the tone itself is clicked", async () => {
    const tuplet = makeTuplet({
      tick: 96,
      entries: {
        by_tick: { 16: [101] },
        by_id: { 101: makeTupletTone(101, 16) },
      },
    });

    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} track={makeTrackWithTuplet(tuplet)} />,
    );

    const target = container.querySelector(
      '[data-tone-id="101"]',
    ) as HTMLElement;
    fireEvent.pointerDown(target, { clientX: 452, clientY: 200 });

    await waitFor(() => {
      expect(actions.ui.sequencer.selectTone).toHaveBeenCalledWith(
        101,
        5,
        20,
        1,
        1,
        11,
      );
    });

    expect(actions.ui.sequencer.selectTuplet).not.toHaveBeenCalled();
  });

  it("writes through tuplet context when clicking an existing child tone", async () => {
    const tuplet = makeTuplet({
      tick: 96,
      entries: {
        by_tick: { 16: [101] },
        by_id: { 101: makeTupletTone(101, 16) },
      },
    });

    const { container } = renderWithProviders(
      <SequencerTrack
        {...defaultProps}
        tool={Tool.Write}
        track={makeTrackWithTuplet(tuplet)}
      />,
    );

    const target = container.querySelector(
      '[data-tone-id="101"]',
    ) as HTMLElement;
    fireEvent.pointerDown(target, { clientX: 452, clientY: 200 });

    await waitFor(() => {
      expect(actions.score.tone.create).toHaveBeenCalledWith(
        5,
        16,
        16,
        111,
        null,
        100,
        [],
        11,
      );
    });
  });

  it("extends a newly created tuplet-owned tone while dragging", async () => {
    const tuplet = makeTuplet({ tick: 96 });
    const { container } = renderWithProviders(
      <SequencerTrack
        {...defaultProps}
        tool={Tool.Write}
        track={makeTrackWithTuplet(tuplet)}
      />,
    );

    const target = container.querySelector(
      '[data-tuplet-id="11"]',
    ) as HTMLElement;
    fireEvent.pointerDown(target, { clientX: 440, clientY: 200 });

    await waitFor(() => {
      expect(actions.score.tone.create).toHaveBeenCalledWith(
        5,
        16,
        16,
        111,
        null,
        100,
        [],
        11,
      );
    });

    fireEvent.pointerMove(document, { clientX: 500, clientY: 200 });

    await waitFor(() => {
      expect(actions.score.tone.setDuration).toHaveBeenCalledWith(5, 1, 32, 11);
    });
  });

  it("lets an off-grid tuplet note extend exactly to the tuplet end boundary before the next host snap", async () => {
    const tuplet = makeTuplet({ tick: 8 });
    const { container } = renderWithProviders(
      <SequencerTrack
        {...defaultProps}
        tool={Tool.Write}
        track={makeTrackWithTuplet(tuplet)}
      />,
    );

    const target = container.querySelector(
      '[data-tuplet-id="11"]',
    ) as HTMLElement;
    fireEvent.pointerDown(target, { clientX: 35, clientY: 200 });

    await waitFor(() => {
      expect(actions.score.tone.create).toHaveBeenCalledWith(
        5,
        0,
        16,
        111,
        null,
        100,
        [],
        11,
      );
    });

    fireEvent.pointerMove(document, { clientX: 159, clientY: 200 });

    await waitFor(() => {
      expect(actions.score.tone.setDuration).toHaveBeenCalledWith(5, 1, 48, 11);
    });
  });

  it("removes tuplet-owned tones through the shared erase seam", async () => {
    const tuplet = makeTuplet({
      tick: 96,
      entries: {
        by_tick: { 16: [101] },
        by_id: { 101: makeTupletTone(101, 16) },
      },
    });

    const { container } = renderWithProviders(
      <SequencerTrack
        {...defaultProps}
        tool={Tool.Erase}
        track={makeTrackWithTuplet(tuplet)}
      />,
    );

    const target = container.querySelector(
      '[data-tone-id="101"]',
    ) as HTMLElement;
    fireEvent.pointerDown(target, { clientX: 452, clientY: 200 });

    await waitFor(() => {
      expect(actions.score.tone.remove).toHaveBeenCalledWith(5, 11, 101);
    });

    expect(actions.ui.sequencer.clearSelection).toHaveBeenCalled();
  });

  it("keeps empty tuplet slot markers visible after a child note is removed", async () => {
    const occupiedTuplet = makeTuplet({
      tick: 96,
      entries: {
        by_tick: { 16: [101] },
        by_id: { 101: makeTupletTone(101, 16) },
      },
    });

    const { container, rerender } = renderWithProviders(
      <SequencerTrack
        {...defaultProps}
        tool={Tool.Erase}
        track={makeTrackWithTuplet(occupiedTuplet)}
      />,
    );

    const target = container.querySelector(
      '[data-tone-id="101"]',
    ) as HTMLElement;
    fireEvent.pointerDown(target, { clientX: 452, clientY: 200 });

    await waitFor(() => {
      expect(actions.score.tone.remove).toHaveBeenCalledWith(5, 11, 101);
    });

    rerender(
      <SequencerTrack
        {...defaultProps}
        tool={Tool.Erase}
        track={makeTrackWithTuplet(makeTuplet({ tick: 96 }))}
      />,
    );

    expect(
      container.querySelector('[data-tone-id="101"]'),
    ).not.toBeInTheDocument();
    expect(
      container.querySelectorAll('[data-tuplet-slot-marker="11"]'),
    ).toHaveLength(4);
  });

  it("handles pointerDown with Select tool without crashing", () => {
    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} tool={Tool.Select} />,
    );
    const div = container.firstChild as HTMLElement;
    fireEvent.pointerDown(div, { clientX: 100, clientY: 200 });
  });

  it("handles pointerDown with Erase tool without crashing", () => {
    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} tool={Tool.Erase} />,
    );
    const div = container.firstChild as HTMLElement;
    fireEvent.pointerDown(div, { clientX: 100, clientY: 200 });
  });

  it("handles pointerDown with Slice tool without crashing", () => {
    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} tool={Tool.Slice} />,
    );
    const div = container.firstChild as HTMLElement;
    fireEvent.pointerDown(div, { clientX: 100, clientY: 200 });
  });

  it("renders tick bar lines for bar and beat entries", () => {
    const ticks = {
      length: 4,
      tickWidth: 100,
      entries: [
        { type: "bar" as const, x: 0 },
        { type: "beat" as const, x: 100 },
        { type: "sub" as const, x: 50 },
      ],
    };
    const slots = makeSlots();
    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} ticks={ticks} slots={slots} />,
    );
    const barLine = container.querySelector('[data-sequencer-tick-line="bar"]');
    const beatLine = container.querySelector(
      '[data-sequencer-tick-line="beat"]',
    );

    expect(barLine).toBeInTheDocument();
    expect(barLine?.className).toContain("bg-neutral-500");
    expect(beatLine).toBeInTheDocument();
    expect(beatLine?.className).toContain("bg-neutral-700");
  });

  it("renders with selection state without crashing", () => {
    const appState = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: 1,
            instrument_id: 1,
            stave_id: 20,
            track_id: 5,
            tone_id: 1,
          },
        },
      } as never,
    });
    const track = makeTrack([
      {
        pitch: { int: 60, accidental: Accidental.Natural },
        velocity: { int: 100 },
        articulations: [] as Articulation[],
      },
    ]);
    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} track={track} />,
      { appState },
    );
    expect(container.firstChild).toBeInTheDocument();
  });

  it("renders an empty tuplet region with compressed slot markers", () => {
    const tuplet = makeTuplet({ tick: 0 });
    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} track={makeTrackWithTuplet(tuplet)} />,
    );

    expect(
      container.querySelector('[data-tuplet-id="11"]'),
    ).toBeInTheDocument();
    expect(
      container.querySelectorAll('[data-tuplet-slot-marker="11"]'),
    ).toHaveLength(4);
  });

  it("renders tuplet markers with a bar color only when aligned to a main bar start", () => {
    const tuplet = makeTuplet({ tick: 0 });
    const ticks = {
      length: 16,
      tickWidth: 4,
      entries: [
        { type: "bar" as const, x: 0 },
        { type: "beat" as const, x: 32 },
        { type: "bar" as const, x: 64 },
      ],
    };

    const { container } = renderWithProviders(
      <SequencerTrack
        {...defaultProps}
        ticks={ticks}
        track={makeTrackWithTuplet(tuplet)}
      />,
    );

    const firstMarker = container.querySelector(
      '[data-tuplet-slot-marker="11"][data-tuplet-slot-index="0"]',
    );
    const secondMarker = container.querySelector(
      '[data-tuplet-slot-marker="11"][data-tuplet-slot-index="1"]',
    );

    expect(firstMarker?.className).toContain("border-neutral-500");
    expect(firstMarker?.className).not.toContain("border-neutral-700");
    expect(secondMarker?.className).toContain("border-neutral-700");
  });

  it("suppresses host beat lines inside a tuplet region so local markers own the pattern", () => {
    const tuplet = makeTuplet({ tick: 32 });
    const ticks = {
      length: 16,
      tickWidth: 4,
      entries: [
        { type: "bar" as const, x: 0 },
        { type: "beat" as const, x: 32 * 4 },
        { type: "beat" as const, x: 40 * 4 },
        { type: "beat" as const, x: 64 * 4 },
      ],
    };

    const { container } = renderWithProviders(
      <SequencerTrack
        {...defaultProps}
        ticks={ticks}
        track={makeTrackWithTuplet(tuplet)}
      />,
    );

    expect(
      container.querySelectorAll('[data-sequencer-tick-line="beat"]'),
    ).toHaveLength(1);
  });

  it("clears selection when empty tuplet region space is clicked with the select tool", () => {
    const tuplet = makeTuplet();
    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} track={makeTrackWithTuplet(tuplet)} />,
    );

    const target = container.querySelector(
      '[data-tuplet-id="11"]',
    ) as HTMLElement;
    fireEvent.pointerDown(target, { clientX: 390, clientY: 120 });

    expect(actions.ui.sequencer.clearSelection).toHaveBeenCalled();
    expect(actions.ui.sequencer.selectTuplet).not.toHaveBeenCalled();
    expect(actions.ui.sequencer.selectTone).not.toHaveBeenCalled();
  });

  it("ignores tuplet clicks when erase tool is active", () => {
    const tuplet = makeTuplet();
    const { container } = renderWithProviders(
      <SequencerTrack
        {...defaultProps}
        tool={Tool.Erase}
        track={makeTrackWithTuplet(tuplet)}
      />,
    );

    const target = container.querySelector(
      '[data-tuplet-id="11"]',
    ) as HTMLElement;
    fireEvent.pointerDown(target, { clientX: 390, clientY: 120 });

    expect(actions.score.tone.remove).not.toHaveBeenCalled();
    expect(actions.ui.sequencer.clearSelection).not.toHaveBeenCalled();
  });

  it("renders a host-owned tone with piecewise width when it crosses a tuplet boundary", () => {
    const tuplet = makeTuplet({ tick: 32 });
    const track = makeTrackWithTuplet(tuplet);

    track.entries.by_id[301] = {
      id: 301,
      tick: 16,
      type: "Tone",
      duration: 32,
      pitch: { int: 60, accidental: Accidental.Natural },
      velocity: { int: 100 },
      articulations: [] as Articulation[],
    };

    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} track={track} />,
    );

    const tone = container.querySelector('[data-tone-id="301"]') as HTMLElement;

    expect(tone).toBeInTheDocument();
    expect(parseFloat(tone.style.left)).toBeCloseTo(65, 5);
    expect(parseFloat(tone.style.width)).toBeCloseTo(105.6666667, 5);
  });

  it("drags a tuplet-owned tone through the shared shift seam with tuplet context", async () => {
    const tuplet = makeTuplet({
      tick: 96,
      entries: {
        by_tick: { 16: [101] },
        by_id: { 101: makeTupletTone(101, 16) },
      },
    });

    const { container } = renderWithProviders(
      <SequencerTrack {...defaultProps} track={makeTrackWithTuplet(tuplet)} />,
    );

    const target = container.querySelector(
      '[data-tone-id="101"]',
    ) as HTMLElement;
    fireEvent.pointerDown(target, { clientX: 452, clientY: 200 });

    await waitFor(() => {
      expect(actions.ui.sequencer.selectTone).toHaveBeenCalledWith(
        101,
        5,
        20,
        1,
        1,
        11,
      );
    });

    fireEvent.pointerMove(document, { clientX: 474, clientY: 200 });

    await waitFor(() => {
      expect(actions.score.tone.shift).toHaveBeenCalledWith(5, 101, 32, 11, 11);
    });
  });
});
