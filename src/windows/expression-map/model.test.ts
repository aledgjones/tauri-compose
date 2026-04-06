import { describe, expect, it } from "vitest";
import {
  Articulation,
  ExpressionMapEqualityComparator,
} from "../../services/types";
import {
  getDefaultRuleFallbackName,
  formatDefaultRuleName,
  getNextDefaultRuleName,
} from "./model";

const accentCondition = {
  id: "accent-condition",
  type: "articulation" as const,
  comparator: ExpressionMapEqualityComparator.Equals,
  articulation: Articulation.Accent,
};

const noOpAction = {
  id: "no-op-action",
  type: "no_op" as const,
};

describe("expression map model helpers", () => {
  it("formats fallback rule labels with spaces", () => {
    expect(formatDefaultRuleName(1)).toBe("Rule 1");
    expect(formatDefaultRuleName(4)).toBe("Rule 4");
  });

  it("reuses the smallest unused default rule name", () => {
    expect(
      getNextDefaultRuleName([
        {
          id: "rule-2",
          name: "Rule 2",
          conditions: [accentCondition],
          actions: [noOpAction],
        },
        {
          id: "rule-custom",
          name: "Custom",
          conditions: [accentCondition],
          actions: [noOpAction],
        },
        {
          id: "rule-7",
          name: "Rule 7",
          conditions: [accentCondition],
          actions: [noOpAction],
        },
      ]),
    ).toBe("Rule 1");
  });

  it("treats blank fallback rules as reserved when deriving the next default name", () => {
    expect(
      getNextDefaultRuleName([
        {
          id: "blank-rule",
          name: "",
          conditions: [accentCondition],
          actions: [noOpAction],
        },
        {
          id: "rule-2",
          name: "Rule 2",
          conditions: [accentCondition],
          actions: [noOpAction],
        },
      ]),
    ).toBe("Rule 3");
  });

  it("ignores non-canonical Rule names when reserving generated slots", () => {
    expect(
      getNextDefaultRuleName([
        {
          id: "rule-01",
          name: "Rule01",
          conditions: [accentCondition],
          actions: [noOpAction],
        },
      ]),
    ).toBe("Rule 1");
  });

  it("treats legacy no-space Rule names as occupied generated slots", () => {
    expect(
      getNextDefaultRuleName([
        {
          id: "rule-1-nospace",
          name: "Rule1",
          conditions: [accentCondition],
          actions: [noOpAction],
        },
      ]),
    ).toBe("Rule 2");
  });

  it("derives blank fallback labels from the same smallest-unused policy", () => {
    expect(
      getDefaultRuleFallbackName(
        [
          {
            id: "rule-2",
            name: "Rule 2",
            conditions: [accentCondition],
            actions: [noOpAction],
          },
          {
            id: "blank-1",
            name: "",
            conditions: [accentCondition],
            actions: [noOpAction],
          },
          {
            id: "blank-2",
            name: "",
            conditions: [accentCondition],
            actions: [noOpAction],
          },
        ],
        1,
      ),
    ).toBe("Rule 1");

    expect(
      getDefaultRuleFallbackName(
        [
          {
            id: "rule-2",
            name: "Rule 2",
            conditions: [accentCondition],
            actions: [noOpAction],
          },
          {
            id: "blank-1",
            name: "",
            conditions: [accentCondition],
            actions: [noOpAction],
          },
          {
            id: "blank-2",
            name: "",
            conditions: [accentCondition],
            actions: [noOpAction],
          },
        ],
        2,
      ),
    ).toBe("Rule 3");
  });
});
