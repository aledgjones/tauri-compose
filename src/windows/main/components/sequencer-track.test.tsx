import { describe, it, expect, vi, beforeEach } from "vitest";
import { fireEvent } from "@testing-library/react";
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
} from "../../../services/types";
import { KeyType } from "../hooks/use-slots";

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
  staves: { by_id: {} },
});

const makeTrack = (tones: object[] = []): Track =>
  ({
    id: 5,
    entries: {
      by_tick: {},
      by_id: Object.fromEntries(
        tones.map((t, i) => [
          i + 1,
          { id: i + 1, type: "Tone", tick: i * 32, duration: 16, ...t },
        ]),
      ),
    },
  }) as unknown as Track;

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
    // Each slot gets an absolute div
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
    // Should not throw
    fireEvent.pointerDown(div, { clientX: 100, clientY: 200 });
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
    expect(container.querySelector(".bg-neutral-500")).toBeInTheDocument();
    expect(container.querySelector(".bg-neutral-800")).toBeInTheDocument();
  });

  it("renders with selection state without crashing", () => {
    const appState = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: 1,
            instrument_id: 1,
            stave_id: 0,
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
});
