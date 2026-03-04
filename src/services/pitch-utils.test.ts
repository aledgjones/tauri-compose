import { describe, it, expect } from "vitest";
import {
  naturalizePitch,
  getPitchLetter,
  getAccidentalString,
  getPitchOctave,
  getPitchName,
} from "./pitch-utils";
import { Accidental } from "./types";
import type { Pitch } from "./types";

const makePitch = (int: number, accidental: Accidental): Pitch => ({
  int,
  accidental,
});

describe("naturalizePitch", () => {
  it("subtracts 2 for DoubleSharp", () => {
    expect(naturalizePitch(makePitch(62, Accidental.DoubleSharp))).toBe(60);
  });

  it("subtracts 1 for Sharp", () => {
    expect(naturalizePitch(makePitch(61, Accidental.Sharp))).toBe(60);
  });

  it("returns int unchanged for Natural", () => {
    expect(naturalizePitch(makePitch(60, Accidental.Natural))).toBe(60);
  });

  it("adds 1 for Flat", () => {
    expect(naturalizePitch(makePitch(59, Accidental.Flat))).toBe(60);
  });

  it("adds 2 for DoubleFlat", () => {
    expect(naturalizePitch(makePitch(58, Accidental.DoubleFlat))).toBe(60);
  });
});

describe("getPitchLetter", () => {
  it("returns C for natural pitch mod 12 = 0 (e.g. int 60, Natural)", () => {
    expect(getPitchLetter(makePitch(60, Accidental.Natural))).toBe("C");
  });

  it("returns D for natural pitch mod 12 = 2", () => {
    expect(getPitchLetter(makePitch(62, Accidental.Natural))).toBe("D");
  });

  it("returns E for natural pitch mod 12 = 4", () => {
    expect(getPitchLetter(makePitch(64, Accidental.Natural))).toBe("E");
  });

  it("returns F for natural pitch mod 12 = 5", () => {
    expect(getPitchLetter(makePitch(65, Accidental.Natural))).toBe("F");
  });

  it("returns G for natural pitch mod 12 = 7", () => {
    expect(getPitchLetter(makePitch(67, Accidental.Natural))).toBe("G");
  });

  it("returns A for natural pitch mod 12 = 9", () => {
    expect(getPitchLetter(makePitch(69, Accidental.Natural))).toBe("A");
  });

  it("returns B for natural pitch mod 12 = 11", () => {
    expect(getPitchLetter(makePitch(71, Accidental.Natural))).toBe("B");
  });

  it("returns empty string for non-natural-note (e.g. C# not naturalized to natural note row)", () => {
    // int 61 (C#/Db). With Natural accidental, naturalized=61, 61%12=1 → ""
    expect(getPitchLetter(makePitch(61, Accidental.Natural))).toBe("");
  });
});

describe("getAccidentalString", () => {
  it("returns ${double-sharp} for DoubleSharp", () => {
    expect(getAccidentalString(Accidental.DoubleSharp)).toBe("${double-sharp}");
  });

  it("returns ${sharp} for Sharp", () => {
    expect(getAccidentalString(Accidental.Sharp)).toBe("${sharp}");
  });

  it("returns empty string for Natural", () => {
    expect(getAccidentalString(Accidental.Natural)).toBe("");
  });

  it("returns ${flat} for Flat", () => {
    expect(getAccidentalString(Accidental.Flat)).toBe("${flat}");
  });

  it("returns ${double-flat} for DoubleFlat", () => {
    expect(getAccidentalString(Accidental.DoubleFlat)).toBe("${double-flat}");
  });
});

describe("getPitchOctave", () => {
  it("returns 4 for MIDI 60 (middle C)", () => {
    expect(getPitchOctave(makePitch(60, Accidental.Natural))).toBe(4);
  });

  it("returns -1 for MIDI 0 (C-1)", () => {
    expect(getPitchOctave(makePitch(0, Accidental.Natural))).toBe(-1);
  });

  it("returns 8 for MIDI 108 (C8)", () => {
    expect(getPitchOctave(makePitch(108, Accidental.Natural))).toBe(8);
  });

  it("returns 5 for MIDI 72 (C5)", () => {
    expect(getPitchOctave(makePitch(72, Accidental.Natural))).toBe(5);
  });
});

describe("getPitchName", () => {
  it("returns 'C4' for middle C (int 60, Natural)", () => {
    expect(getPitchName(makePitch(60, Accidental.Natural))).toBe("C4");
  });

  it("returns 'G4' for G above middle C (int 67, Natural)", () => {
    expect(getPitchName(makePitch(67, Accidental.Natural))).toBe("G4");
  });

  it("includes sharp token for Sharp accidental", () => {
    // int 62 (D), sharp. naturalized=61 (C#) but actual pitch letter based on natural: 62-1=61, 61%12=1 → ""
    // Need a proper Sharp pitch: e.g. F# = int 66, Sharp → naturalized=65, 65%12=5 → "F"
    // octave = Math.floor(66/12) - 1 = 5 - 1 = 4
    const name = getPitchName(makePitch(66, Accidental.Sharp));
    expect(name).toBe("F${sharp}4");
  });

  it("includes flat token for Flat accidental", () => {
    // Bb = int 70, Flat → naturalized=71, 71%12=11 → "B", octave=floor(70/12)-1=5-1=4
    const name = getPitchName(makePitch(70, Accidental.Flat));
    expect(name).toBe("B${flat}4");
  });

  it("returns empty-letter string for non-natural-note pitch", () => {
    // int 61, Natural → naturalized=61, 61%12=1 → "", octave=Math.floor(61/12)-1=4
    expect(getPitchName(makePitch(61, Accidental.Natural))).toBe("4");
  });
});
