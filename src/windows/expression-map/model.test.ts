import { describe, expect, it } from "vitest";
import {
  conditionTypeOptions,
  getDefaultRuleFallbackName,
  playingTechniqueOptions,
  UNNAMED_RULE_LABEL,
} from "./model";

describe("expression map model helpers", () => {
  it("uses an unnamed rule fallback label for blank rule names", () => {
    expect(getDefaultRuleFallbackName()).toBe(UNNAMED_RULE_LABEL);
  });

  it("offers the bounded playing-technique condition type and value labels", () => {
    expect(conditionTypeOptions).toContainEqual({
      value: "playing_technique",
      label: "Playing Technique",
    });
    expect(playingTechniqueOptions).toEqual([
      { value: 0, label: "pizz" },
      { value: 1, label: "arco" },
    ]);
  });
});
