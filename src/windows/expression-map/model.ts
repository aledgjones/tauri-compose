import {
  Articulation,
  ExpressionMapEqualityComparator,
  ExpressionMapNoteLengthComparator,
  ExpressionMapRule,
  NoteDuration,
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

export const conditionTypeOptions = [
  { value: "articulation", label: "Articulation" },
  { value: "written_note_length", label: "Written Note Length" },
] as const;

export const actionTypeOptions = [
  { value: "keyswitch_note", label: "Keyswitch Note" },
  { value: "control_change", label: "CC Message" },
  { value: "note_length_transform", label: "Note-Length Transform" },
  { value: "no_op", label: "No Action" },
] as const;

export const formatDefaultRuleName = (ruleNumber: number) =>
  `Rule ${ruleNumber}`;

const matchCanonicalDefaultRuleName = (ruleName: string) =>
  ruleName.trim().match(/^Rule ?([1-9]\d*)$/);

const getUsedDefaultRuleNumbers = (rules: ExpressionMapRule[]) => {
  const usedRuleNumbers = new Set<number>();

  rules.forEach((rule) => {
    const match = matchCanonicalDefaultRuleName(rule.name);

    if (match) {
      usedRuleNumbers.add(Number(match[1]));
    }
  });

  return usedRuleNumbers;
};

const getSmallestUnusedRuleNumber = (usedRuleNumbers: Set<number>) => {
  let candidateRuleNumber = 1;

  while (usedRuleNumbers.has(candidateRuleNumber)) {
    candidateRuleNumber += 1;
  }

  return candidateRuleNumber;
};

const getAllocatedDefaultRuleNumbers = (rules: ExpressionMapRule[]) => {
  const usedRuleNumbers = getUsedDefaultRuleNumbers(rules);
  const allocatedRuleNumbers: Array<number | null> = rules.map(() => null);

  rules.forEach((rule, index) => {
    const match = matchCanonicalDefaultRuleName(rule.name);

    if (match) {
      allocatedRuleNumbers[index] = Number(match[1]);
      return;
    }

    if (rule.name.trim().length > 0) {
      return;
    }

    const nextRuleNumber = getSmallestUnusedRuleNumber(usedRuleNumbers);
    usedRuleNumbers.add(nextRuleNumber);
    allocatedRuleNumbers[index] = nextRuleNumber;
  });

  return allocatedRuleNumbers;
};

export const getNextDefaultRuleName = (rules: ExpressionMapRule[]) => {
  const allocatedRuleNumbers = getAllocatedDefaultRuleNumbers(rules);
  const usedRuleNumbers = new Set(
    allocatedRuleNumbers.filter(
      (ruleNumber): ruleNumber is number => ruleNumber != null,
    ),
  );

  return formatDefaultRuleName(getSmallestUnusedRuleNumber(usedRuleNumbers));
};

export const getDefaultRuleFallbackName = (
  rules: ExpressionMapRule[],
  ruleIndex: number,
) => {
  const allocatedRuleNumbers = getAllocatedDefaultRuleNumbers(rules);
  const allocatedRuleNumber = allocatedRuleNumbers[ruleIndex];

  return allocatedRuleNumber == null
    ? formatDefaultRuleName(ruleIndex + 1)
    : formatDefaultRuleName(allocatedRuleNumber);
};

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
