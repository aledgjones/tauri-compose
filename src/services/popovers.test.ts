import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  parseBarlinePopover,
  parseClefPopover,
  parseKeySignaturePopover,
  parseTempoPopover,
  parseTimeSignaturePopover,
} from "./popovers";
import {
  BarlineDrawType,
  ClefDrawType,
  KeySignatureMode,
  NoteDuration,
  TimeSignatureDrawType,
} from "./types";

// ---------------------------------------------------------------------------
// Mock the entire actions module — popovers have no logic beyond parse +
// dispatch, so we test that the right method is called with the right args.
// ---------------------------------------------------------------------------
vi.mock("./actions", () => ({
  actions: {
    score: {
      barline: {
        create: vi.fn(),
        clearAtTick: vi.fn(),
      },
      clef: {
        create: vi.fn(),
        clear: vi.fn(),
      },
      keySignature: {
        create: vi.fn(),
        clearAtTick: vi.fn(),
      },
      timeSignature: {
        create: vi.fn(),
        clearAtTick: vi.fn(),
      },
      tempo: {
        create: vi.fn(),
        clearAtTick: vi.fn(),
      },
    },
  },
}));

// Import after mock is registered
import { actions } from "./actions";

const FLOW_ID = 1;
const TICK = 128;
const STAVE_ID = 10;

// ---------------------------------------------------------------------------
// parseBarlinePopover
// ---------------------------------------------------------------------------
describe("parseBarlinePopover", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('clears barline when input is "clear"', () => {
    parseBarlinePopover("clear", FLOW_ID, TICK);
    expect(actions.score.barline.clearAtTick).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
    );
  });

  it.each(["|", "single", "normal"])(
    'creates Single barline for "%s"',
    (input) => {
      parseBarlinePopover(input, FLOW_ID, TICK);
      expect(actions.score.barline.create).toHaveBeenCalledWith(
        FLOW_ID,
        TICK,
        BarlineDrawType.Single,
      );
    },
  );

  it.each(["||", "double"])('creates Double barline for "%s"', (input) => {
    parseBarlinePopover(input, FLOW_ID, TICK);
    expect(actions.score.barline.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      BarlineDrawType.Double,
    );
  });

  it.each(["|||", "triple"])('creates Triple barline for "%s"', (input) => {
    parseBarlinePopover(input, FLOW_ID, TICK);
    expect(actions.score.barline.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      BarlineDrawType.Triple,
    );
  });

  it.each([":", "dashed"])('creates Dashed barline for "%s"', (input) => {
    parseBarlinePopover(input, FLOW_ID, TICK);
    expect(actions.score.barline.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      BarlineDrawType.Dashed,
    );
  });

  it.each(["|:", "start"])('creates StartRepeat barline for "%s"', (input) => {
    parseBarlinePopover(input, FLOW_ID, TICK);
    expect(actions.score.barline.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      BarlineDrawType.StartRepeat,
    );
  });

  it.each([":|", "end"])('creates EndRepeat barline for "%s"', (input) => {
    parseBarlinePopover(input, FLOW_ID, TICK);
    expect(actions.score.barline.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      BarlineDrawType.EndRepeat,
    );
  });

  it.each([":|:", ":||:", "end-start", "endstart"])(
    'creates EndStartRepeat barline for "%s"',
    (input) => {
      parseBarlinePopover(input, FLOW_ID, TICK);
      expect(actions.score.barline.create).toHaveBeenCalledWith(
        FLOW_ID,
        TICK,
        BarlineDrawType.EndStartRepeat,
      );
    },
  );

  it("throws for unknown input", () => {
    expect(() => parseBarlinePopover("xyz", FLOW_ID, TICK)).toThrow(
      "Unknown input: xyz",
    );
  });
});

// ---------------------------------------------------------------------------
// parseClefPopover
// ---------------------------------------------------------------------------
describe("parseClefPopover", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('clears clef when input is "clear"', () => {
    parseClefPopover("clear", FLOW_ID, TICK, STAVE_ID);
    expect(actions.score.clef.clear).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      STAVE_ID,
    );
  });

  it.each(["treble", "g", "G", "sol"])('creates G clef for "%s"', (input) => {
    parseClefPopover(input, FLOW_ID, TICK, STAVE_ID);
    expect(actions.score.clef.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      STAVE_ID,
      67,
      2,
      ClefDrawType.G,
    );
  });

  it.each(["bass", "f", "F", "fa"])('creates F clef for "%s"', (input) => {
    parseClefPopover(input, FLOW_ID, TICK, STAVE_ID);
    expect(actions.score.clef.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      STAVE_ID,
      53,
      -2,
      ClefDrawType.F,
    );
  });

  it.each(["alto", "ca", "CA"])('creates C alto clef for "%s"', (input) => {
    parseClefPopover(input, FLOW_ID, TICK, STAVE_ID);
    expect(actions.score.clef.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      STAVE_ID,
      60,
      0,
      ClefDrawType.C,
    );
  });

  it.each(["tenor", "ct", "CT"])('creates C tenor clef for "%s"', (input) => {
    parseClefPopover(input, FLOW_ID, TICK, STAVE_ID);
    expect(actions.score.clef.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      STAVE_ID,
      60,
      -2,
      ClefDrawType.C,
    );
  });

  it("throws for unknown input", () => {
    expect(() => parseClefPopover("bass2", FLOW_ID, TICK, STAVE_ID)).toThrow(
      "Unknown input: bass2",
    );
  });
});

// ---------------------------------------------------------------------------
// parseKeySignaturePopover
// ---------------------------------------------------------------------------
describe("parseKeySignaturePopover", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('clears key signature when input is "clear"', () => {
    parseKeySignaturePopover("clear", FLOW_ID, TICK);
    expect(actions.score.keySignature.clearAtTick).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
    );
  });

  // Major sharps
  it.each([
    ["C#", 7],
    ["7s", 7],
    ["7#", 7],
    ["F#", 6],
    ["G", 1],
    ["1s", 1],
  ])('creates Major key with offset %i for "%s"', (input, offset) => {
    parseKeySignaturePopover(input, FLOW_ID, TICK);
    expect(actions.score.keySignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      KeySignatureMode.Major,
      offset,
    );
  });

  // Major open/atonal
  it.each(["open", "atonal", "C", "0s", "0f", "0#", "0b"])(
    'creates Major key with offset 0 for "%s"',
    (input) => {
      parseKeySignaturePopover(input, FLOW_ID, TICK);
      expect(actions.score.keySignature.create).toHaveBeenCalledWith(
        FLOW_ID,
        TICK,
        KeySignatureMode.Major,
        0,
      );
    },
  );

  // Major flats
  it.each([
    ["F", -1],
    ["1f", -1],
    ["1b", -1],
    ["Cb", -7],
    ["7f", -7],
    ["7b", -7],
  ])('creates Major key with offset %i for "%s"', (input, offset) => {
    parseKeySignaturePopover(input, FLOW_ID, TICK);
    expect(actions.score.keySignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      KeySignatureMode.Major,
      offset,
    );
  });

  // Minor keys
  it('creates Minor key with offset 7 for "a#"', () => {
    parseKeySignaturePopover("a#", FLOW_ID, TICK);
    expect(actions.score.keySignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      KeySignatureMode.Minor,
      7,
    );
  });

  it('creates Minor key with offset 0 for "a"', () => {
    parseKeySignaturePopover("a", FLOW_ID, TICK);
    expect(actions.score.keySignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      KeySignatureMode.Minor,
      0,
    );
  });

  it('creates Minor key with offset -7 for "ab"', () => {
    parseKeySignaturePopover("ab", FLOW_ID, TICK);
    expect(actions.score.keySignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      KeySignatureMode.Minor,
      -7,
    );
  });

  // Major sharps 2-5
  it.each([
    ["B", 5],
    ["5s", 5],
    ["5#", 5],
    ["E", 4],
    ["4s", 4],
    ["4#", 4],
    ["A", 3],
    ["3s", 3],
    ["3#", 3],
    ["D", 2],
    ["2s", 2],
    ["2#", 2],
  ] as [string, number][])('Major sharp: "%s" → offset %i', (input, offset) => {
    parseKeySignaturePopover(input, FLOW_ID, TICK);
    expect(actions.score.keySignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      KeySignatureMode.Major,
      offset,
    );
  });

  // Major flats 2-6
  it.each([
    ["Bb", -2],
    ["2f", -2],
    ["2b", -2],
    ["Eb", -3],
    ["3f", -3],
    ["3b", -3],
    ["Ab", -4],
    ["4f", -4],
    ["4b", -4],
    ["Db", -5],
    ["5f", -5],
    ["5b", -5],
    ["Gb", -6],
    ["6f", -6],
    ["6b", -6],
  ] as [string, number][])('Major flat: "%s" → offset %i', (input, offset) => {
    parseKeySignaturePopover(input, FLOW_ID, TICK);
    expect(actions.score.keySignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      KeySignatureMode.Major,
      offset,
    );
  });

  // Minor keys remaining
  it.each([
    ["d#", 6],
    ["g#", 5],
    ["c#", 4],
    ["f#", 3],
    ["b", 2],
    ["e", 1],
    ["d", -1],
    ["g", -2],
    ["c", -3],
    ["f", -4],
    ["bb", -5],
    ["eb", -6],
  ] as [string, number][])('Minor key: "%s" → offset %i', (input, offset) => {
    parseKeySignaturePopover(input, FLOW_ID, TICK);
    expect(actions.score.keySignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      KeySignatureMode.Minor,
      offset,
    );
  });

  it("throws for unknown input", () => {
    expect(() => parseKeySignaturePopover("xyz", FLOW_ID, TICK)).toThrow(
      "Unknown input: xyz",
    );
  });
});

describe("parseTempoPopover", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('clears tempo when input is "clear"', () => {
    parseTempoPopover("clear", FLOW_ID, TICK);
    expect(actions.score.tempo.clearAtTick).toHaveBeenCalledWith(FLOW_ID, TICK);
  });

  it("creates a named tempo", () => {
    parseTempoPopover("Allegro", FLOW_ID, TICK);
    expect(actions.score.tempo.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      "Allegro",
      NoteDuration.Quarter,
      0,
      130,
      false,
      true,
      false,
      false,
    );
  });

  it("creates a named tempo with metronome mark", () => {
    parseTempoPopover("Allegro q=120", FLOW_ID, TICK);
    expect(actions.score.tempo.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      "Allegro",
      NoteDuration.Quarter,
      0,
      120,
      false,
      true,
      true,
      false,
    );
  });

  it("creates a parenthesized metronome mark", () => {
    parseTempoPopover("Allegro (q=120)", FLOW_ID, TICK);
    expect(actions.score.tempo.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      "Allegro",
      NoteDuration.Quarter,
      0,
      120,
      false,
      true,
      true,
      true,
    );
  });

  it("creates a metronome-only tempo", () => {
    parseTempoPopover("q=120", FLOW_ID, TICK);
    expect(actions.score.tempo.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      "",
      NoteDuration.Quarter,
      0,
      120,
      false,
      false,
      true,
      false,
    );
  });

  it("throws for invalid tempo input", () => {
    expect(() => parseTempoPopover("Vivace", FLOW_ID, TICK)).toThrow(
      "Unknown input: Vivace",
    );
  });
});

// ---------------------------------------------------------------------------
// parseTimeSignaturePopover
// ---------------------------------------------------------------------------
describe("parseTimeSignaturePopover", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('parses "4/4" → Quarter beats', () => {
    parseTimeSignaturePopover("4/4", FLOW_ID, TICK);
    expect(actions.score.timeSignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      4,
      NoteDuration.Quarter,
      TimeSignatureDrawType.Regular,
    );
  });

  it('parses "6/8" → Eighth beats', () => {
    parseTimeSignaturePopover("6/8", FLOW_ID, TICK);
    expect(actions.score.timeSignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      6,
      NoteDuration.Eighth,
      TimeSignatureDrawType.Regular,
    );
  });

  it('parses "3/4" → Quarter beats', () => {
    parseTimeSignaturePopover("3/4", FLOW_ID, TICK);
    expect(actions.score.timeSignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      3,
      NoteDuration.Quarter,
      TimeSignatureDrawType.Regular,
    );
  });

  it('parses "2/2" → Half beats', () => {
    parseTimeSignaturePopover("2/2", FLOW_ID, TICK);
    expect(actions.score.timeSignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      2,
      NoteDuration.Half,
      TimeSignatureDrawType.Regular,
    );
  });

  it('clears time signature for "clear"', () => {
    parseTimeSignaturePopover("clear", FLOW_ID, TICK);
    expect(actions.score.timeSignature.clearAtTick).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
    );
  });

  it('creates CommonTime for "c"', () => {
    parseTimeSignaturePopover("c", FLOW_ID, TICK);
    expect(actions.score.timeSignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      4,
      NoteDuration.Quarter,
      TimeSignatureDrawType.CommonTime,
    );
  });

  it('creates SplitCommonTime for "cutc"', () => {
    parseTimeSignaturePopover("cutc", FLOW_ID, TICK);
    expect(actions.score.timeSignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      2,
      NoteDuration.Half,
      TimeSignatureDrawType.SplitCommonTime,
    );
  });

  it('parses "4/1" → Whole beats', () => {
    parseTimeSignaturePopover("4/1", FLOW_ID, TICK);
    expect(actions.score.timeSignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      4,
      NoteDuration.Whole,
      TimeSignatureDrawType.Regular,
    );
  });

  it('parses "4/16" → Sixteenth beats', () => {
    parseTimeSignaturePopover("4/16", FLOW_ID, TICK);
    expect(actions.score.timeSignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      4,
      NoteDuration.Sixteenth,
      TimeSignatureDrawType.Regular,
    );
  });

  it('parses "4/32" → ThirtySecond beats', () => {
    parseTimeSignaturePopover("4/32", FLOW_ID, TICK);
    expect(actions.score.timeSignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      4,
      NoteDuration.ThirtySecond,
      TimeSignatureDrawType.Regular,
    );
  });

  it('parses "4/64" → SixtyFourth beats', () => {
    parseTimeSignaturePopover("4/64", FLOW_ID, TICK);
    expect(actions.score.timeSignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      4,
      NoteDuration.SixtyFourth,
      TimeSignatureDrawType.Regular,
    );
  });

  it('parses "4/128" → HundredTwentyEighth beats', () => {
    parseTimeSignaturePopover("4/128", FLOW_ID, TICK);
    expect(actions.score.timeSignature.create).toHaveBeenCalledWith(
      FLOW_ID,
      TICK,
      4,
      NoteDuration.HudredTwentyEighth,
      TimeSignatureDrawType.Regular,
    );
  });

  it("throws for unknown input", () => {
    expect(() => parseTimeSignaturePopover("xyz", FLOW_ID, TICK)).toThrow(
      "Unknown input: xyz",
    );
  });
});
