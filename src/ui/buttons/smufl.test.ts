import { describe, it, expect } from "vitest";
import { SMuFL } from "./smufl";

describe("SMuFL", () => {
  it("Staccato has the correct Unicode codepoint", () => {
    expect(SMuFL.Staccato).toBe("\u{E4A2}");
  });

  it("Tenuto has the correct Unicode codepoint", () => {
    expect(SMuFL.Tenuto).toBe("\u{E4A4}");
  });

  it("Staccatissimo has the correct Unicode codepoint", () => {
    expect(SMuFL.Staccatissimo).toBe("\u{E4A6}");
  });

  it("StaccatoTenuto has the correct Unicode codepoint", () => {
    expect(SMuFL.StaccatoTenuto).toBe("\u{E4B2}");
  });

  it("Accent has the correct Unicode codepoint", () => {
    expect(SMuFL.Accent).toBe("\u{E4A0}");
  });

  it("Marcato has the correct Unicode codepoint", () => {
    expect(SMuFL.Marcato).toBe("\u{E4AC}");
  });

  it("Stressed has the correct Unicode codepoint", () => {
    expect(SMuFL.Stressed).toBe("\u{E4B6}");
  });

  it("Unstressed has the correct Unicode codepoint", () => {
    expect(SMuFL.Unstressed).toBe("\u{E4B8}");
  });

  it("Sharp has the correct Unicode codepoint", () => {
    expect(SMuFL.Sharp).toBe("\u{E262}");
  });

  it("Natural has the correct Unicode codepoint", () => {
    expect(SMuFL.Natural).toBe("\u{E261}");
  });

  it("Flat has the correct Unicode codepoint", () => {
    expect(SMuFL.Flat).toBe("\u{E260}");
  });

  it("WholeNote has the correct Unicode codepoint", () => {
    expect(SMuFL.WholeNote).toBe("\u{E1D2}");
  });

  it("HalfNote has the correct Unicode codepoint", () => {
    expect(SMuFL.HalfNote).toBe("\u{E1D3}");
  });

  it("QuarterNote has the correct Unicode codepoint", () => {
    expect(SMuFL.QuarterNote).toBe("\u{E1D5}");
  });

  it("EighthNote has the correct Unicode codepoint", () => {
    expect(SMuFL.EighthNote).toBe("\u{E1D7}");
  });

  it("SixteenthNote has the correct Unicode codepoint", () => {
    expect(SMuFL.SixteenthNote).toBe("\u{E1D9}");
  });

  it("ThirtySecondNote has the correct Unicode codepoint", () => {
    expect(SMuFL.ThirtySecondNote).toBe("\u{E1DB}");
  });

  it("all note duration glyphs are distinct strings", () => {
    const noteGlyphs = [
      SMuFL.WholeNote,
      SMuFL.HalfNote,
      SMuFL.QuarterNote,
      SMuFL.EighthNote,
      SMuFL.SixteenthNote,
      SMuFL.ThirtySecondNote,
    ];
    const unique = new Set(noteGlyphs);
    expect(unique.size).toBe(noteGlyphs.length);
  });

  it("all articulation glyphs are distinct strings", () => {
    const articulationGlyphs = [
      SMuFL.Staccato,
      SMuFL.Tenuto,
      SMuFL.Staccatissimo,
      SMuFL.StaccatoTenuto,
      SMuFL.Accent,
      SMuFL.Marcato,
      SMuFL.Stressed,
      SMuFL.Unstressed,
    ];
    const unique = new Set(articulationGlyphs);
    expect(unique.size).toBe(articulationGlyphs.length);
  });
});
