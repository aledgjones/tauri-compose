import {
  Articulation,
  ExpressionMapEqualityComparator,
  ExpressionMapNoteLengthComparator,
  NoteDuration,
  PlayingTechniqueKind,
} from "../../services/types";

export const articulationOptions = [
  { value: Articulation.Staccato, label: "Staccato" },
  { value: Articulation.Tenuto, label: "Tenuto" },
  { value: Articulation.Staccatissimo, label: "Staccatissimo" },
  { value: Articulation.StaccatoTenuto, label: "Staccato-Tenuto" },
  { value: Articulation.Accent, label: "Accent" },
  { value: Articulation.Marcato, label: "Marcato" },
  { value: Articulation.Stressed, label: "Stressed" },
  { value: Articulation.Unstressed, label: "Unstressed" },
] as const;

export const noteDurationOptions = [
  { value: NoteDuration.Whole, label: "Whole" },
  { value: NoteDuration.Half, label: "Half" },
  { value: NoteDuration.Quarter, label: "Quarter" },
  { value: NoteDuration.Eighth, label: "Eighth" },
  { value: NoteDuration.Sixteenth, label: "Sixteenth" },
  { value: NoteDuration.ThirtySecond, label: "Thirty-second" },
  { value: NoteDuration.SixtyFourth, label: "Sixty-fourth" },
  { value: NoteDuration.HudredTwentyEighth, label: "Hundred twenty-eighth" },
] as const;

export const noteLengthComparatorOptions = [
  {
    value: ExpressionMapNoteLengthComparator.LessThan,
    label: "<",
  },
  {
    value: ExpressionMapNoteLengthComparator.LessThanOrEqual,
    label: "<=",
  },
  {
    value: ExpressionMapNoteLengthComparator.Exactly,
    label: "==",
  },
  {
    value: ExpressionMapNoteLengthComparator.NotEqual,
    label: "!=",
  },
  {
    value: ExpressionMapNoteLengthComparator.GreaterThanOrEqual,
    label: ">=",
  },
  {
    value: ExpressionMapNoteLengthComparator.GreaterThan,
    label: ">",
  },
] as const;

export const equalityComparatorOptions = [
  { value: ExpressionMapEqualityComparator.Equals, label: "==" },
  { value: ExpressionMapEqualityComparator.NotEquals, label: "!=" },
] as const;

export const playingTechniqueOptions = [
  { value: PlayingTechniqueKind.Pizz, label: "pizz" },
  { value: PlayingTechniqueKind.Arco, label: "arco" },
] as const;

export const conditionTypeOptions = [
  { value: "articulation", label: "Articulation" },
  { value: "playing_technique", label: "Playing Technique" },
  { value: "written_note_length", label: "Written Note Length" },
  { value: "no_op", label: "--" },
] as const;

export const actionTypeOptions = [
  { value: "keyswitch_note", label: "Keyswitch Note" },
  { value: "control_change", label: "CC Message" },
  { value: "note_length_transform", label: "Note-Length Transform" },
  { value: "no_op", label: "--" },
] as const;

export const UNNAMED_RULE_LABEL = "Unnamed Rule";

export const getDefaultRuleFallbackName = () => UNNAMED_RULE_LABEL;

export const matchesExpressionMapSearch = (
  mapName: string,
  searchTerm: string,
) => {
  const normalizedSearchTerm = searchTerm.trim().toLowerCase();

  if (normalizedSearchTerm.length === 0) {
    return true;
  }

  return mapName.toLowerCase().includes(normalizedSearchTerm);
};
