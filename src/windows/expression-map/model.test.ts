import { describe, expect, it } from "vitest";
import { getDefaultRuleFallbackName, UNNAMED_RULE_LABEL } from "./model";

describe("expression map model helpers", () => {
  it("uses an unnamed rule fallback label for blank rule names", () => {
    expect(getDefaultRuleFallbackName()).toBe(UNNAMED_RULE_LABEL);
  });
});
