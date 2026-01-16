import { Pitch, Accidental } from "./types";

export const naturalizePitch = (pitch: Pitch) => {
  switch (pitch.accidental) {
    case Accidental.DoubleSharp:
      return pitch.int - 2;
    case Accidental.Sharp:
      return pitch.int - 1;
    case Accidental.Natural:
      return pitch.int;
    case Accidental.Flat:
      return pitch.int + 1;
    case Accidental.DoubleFlat:
      return pitch.int + 2;
    default:
      return pitch.int;
  }
};

export const getPitchLetter = (pitch: Pitch) => {
  const base = naturalizePitch(pitch);

  switch (base % 12) {
    case 0:
      return "C";
    case 2:
      return "D";
    case 4:
      return "E";
    case 5:
      return "F";
    case 7:
      return "G";
    case 9:
      return "A";
    case 11:
      return "B";
    default:
      return "";
  }
};

export const getAccidentalString = (accidental: Accidental) => {
  switch (accidental) {
    case Accidental.DoubleSharp:
      return "${double-sharp}";
    case Accidental.Sharp:
      return "${sharp}";
    case Accidental.Natural:
      return "";
    case Accidental.Flat:
      return "${flat}";
    case Accidental.DoubleFlat:
      return "${double-flat}";
    default:
      return "";
  }
};

export const getPitchOctave = (pitch: Pitch) => {
  return Math.floor(pitch.int / 12) - 1;
};

/**
 * Converts a MIDI pitch object to a human-readable note name (e.g., C4, F♭3, G♯4)
 * @param pitch - The pitch object containing int and accidental properties
 * @returns The note name with octave and appropriate accidental symbol token
 */
export const getPitchName = (pitch: Pitch): string => {
  const letter = getPitchLetter(pitch);
  let accidental = getAccidentalString(pitch.accidental);
  const octave = getPitchOctave(pitch);

  return `${letter}${accidental}${octave}`;
};
