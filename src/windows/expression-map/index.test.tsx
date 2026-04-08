import { listen } from "@tauri-apps/api/event";
import { act, fireEvent, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderWithProviders } from "../../services/test-utils";
import {
  Articulation,
  ExpressionMapEqualityComparator,
  ExpressionMapLibraryState,
  ExpressionMapNoteLengthComparator,
  NoteDuration,
} from "../../services/types";
import { Main } from "./index";

const windowMocks = vi.hoisted(() => ({
  expressionMapMock: {
    id: "legato",
    name: "Legato",
    creator: "Solo Compose",
    version: 3,
    default_actions: [{ id: "default-action", type: "no_op" }],
    rules: [
      {
        id: "accent-keyswitch",
        name: "Accent Keyswitch",
        conditions: [
          {
            id: "accent-condition",
            type: "articulation",
            comparator: "equals",
            articulation: 4,
          },
        ],
        actions: [
          {
            id: "accent-action",
            type: "keyswitch_note",
            midi_note: 24,
            velocity: 96,
          },
        ],
      },
      {
        id: "fallback-rule",
        name: "Rule 2",
        conditions: [
          {
            id: "fallback-condition",
            type: "written_note_length",
            comparator: "less_than",
            note_duration: 8,
          },
        ],
        actions: [{ id: "fallback-action", type: "no_op" }],
      },
    ],
  },
  showMock: vi.fn().mockResolvedValue(undefined),
  createMock: vi.fn().mockResolvedValue({
    id: "11111111-1111-4111-8111-111111111111",
    name: "New Expression Map",
    creator: "",
    version: 1,
    default_actions: [{ id: "new-default-action", type: "no_op" }],
    rules: [],
  }),
  deleteMock: vi.fn().mockResolvedValue(undefined),
  setNameMock: vi.fn().mockResolvedValue(undefined),
  setCreatorMock: vi.fn().mockResolvedValue(undefined),
  setVersionMock: vi.fn().mockResolvedValue(undefined),
  addRuleMock: vi.fn().mockResolvedValue(undefined),
  setRuleNameMock: vi.fn().mockResolvedValue(undefined),
  moveRuleMock: vi.fn().mockResolvedValue(undefined),
  removeRuleMock: vi.fn().mockResolvedValue(undefined),
  addConditionMock: vi.fn().mockResolvedValue(undefined),
  setConditionTypeMock: vi.fn().mockResolvedValue(undefined),
  updateConditionMock: vi.fn().mockResolvedValue(undefined),
  removeConditionMock: vi.fn().mockResolvedValue(undefined),
  addActionMock: vi.fn().mockResolvedValue(undefined),
  setActionTypeMock: vi.fn().mockResolvedValue(undefined),
  updateActionMock: vi.fn().mockResolvedValue(undefined),
  removeActionMock: vi.fn().mockResolvedValue(undefined),
  addDefaultActionMock: vi.fn().mockResolvedValue(undefined),
  setDefaultActionTypeMock: vi.fn().mockResolvedValue(undefined),
  updateDefaultActionMock: vi.fn().mockResolvedValue(undefined),
  removeDefaultActionMock: vi.fn().mockResolvedValue(undefined),
  sortableCreateMock: vi.fn(),
  confirmMock: vi.fn().mockResolvedValue(true),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    show: windowMocks.showMock,
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  confirm: windowMocks.confirmMock,
}));

vi.mock("sortablejs", () => ({
  default: {
    create: windowMocks.sortableCreateMock,
  },
}));

vi.mock("../../services/actions", () => ({
  actions: {
    expressionMaps: {
      create: windowMocks.createMock,
      setName: windowMocks.setNameMock,
      setCreator: windowMocks.setCreatorMock,
      setVersion: windowMocks.setVersionMock,
      delete: windowMocks.deleteMock,
      addRule: windowMocks.addRuleMock,
      setRuleName: windowMocks.setRuleNameMock,
      moveRule: windowMocks.moveRuleMock,
      removeRule: windowMocks.removeRuleMock,
      addCondition: windowMocks.addConditionMock,
      setConditionType: windowMocks.setConditionTypeMock,
      updateCondition: windowMocks.updateConditionMock,
      removeCondition: windowMocks.removeConditionMock,
      addAction: windowMocks.addActionMock,
      setActionType: windowMocks.setActionTypeMock,
      updateAction: windowMocks.updateActionMock,
      removeAction: windowMocks.removeActionMock,
      addDefaultAction: windowMocks.addDefaultActionMock,
      setDefaultActionType: windowMocks.setDefaultActionTypeMock,
      updateDefaultAction: windowMocks.updateDefaultActionMock,
      removeDefaultAction: windowMocks.removeDefaultActionMock,
    },
  },
}));

const expressionMapLibraryState: ExpressionMapLibraryState = {
  order: ["legato", "staccato"],
  by_id: {
    legato: {
      id: "legato",
      name: "Legato",
      creator: "Solo Compose",
      version: 3,
      default_actions: [{ id: "default-action", type: "no_op" }],
      rules: [
        {
          id: "accent-keyswitch",
          name: "Accent Keyswitch",
          conditions: [
            {
              id: "accent-condition",
              type: "articulation",
              comparator: ExpressionMapEqualityComparator.Equals,
              articulation: Articulation.Accent,
            },
          ],
          actions: [
            {
              id: "accent-action",
              type: "keyswitch_note",
              midi_note: 24,
              velocity: 96,
            },
          ],
        },
        {
          id: "fallback-rule",
          name: "Rule 2",
          conditions: [
            {
              id: "fallback-condition",
              type: "written_note_length",
              comparator: ExpressionMapNoteLengthComparator.LessThan,
              note_duration: NoteDuration.Eighth,
            },
          ],
          actions: [{ id: "fallback-action", type: "no_op" }],
        },
      ],
    },
    staccato: {
      id: "staccato",
      name: "Staccato",
      creator: "",
      version: 1,
      default_actions: [{ id: "staccato-default-action", type: "no_op" }],
      rules: [],
    },
  },
};

describe("Expression Map Main", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    windowMocks.sortableCreateMock.mockReturnValue({ destroy: vi.fn() });
    windowMocks.confirmMock.mockResolvedValue(true);
    const resolveExpressionMapForId = (mapId: string) => {
      switch (mapId) {
        case "11111111-1111-4111-8111-111111111111":
          return {
            id: "11111111-1111-4111-8111-111111111111",
            name: "New Expression Map",
            creator: "",
            version: 1,
            default_actions: [{ id: "new-default-action", type: "no_op" }],
            rules: [],
          };
        default:
          return expressionMapLibraryState.by_id[
            mapId as keyof typeof expressionMapLibraryState.by_id
          ];
      }
    };
    windowMocks.setNameMock.mockImplementation(async (mapId, name) => ({
      ...resolveExpressionMapForId(mapId),
      name,
    }));
    windowMocks.setCreatorMock.mockImplementation(async (mapId, creator) => ({
      ...resolveExpressionMapForId(mapId),
      creator,
    }));
    windowMocks.setVersionMock.mockImplementation(async (mapId, version) => ({
      ...resolveExpressionMapForId(mapId),
      version,
    }));
    windowMocks.addRuleMock.mockResolvedValue({
      ...expressionMapLibraryState.by_id.legato,
      rules: [
        ...expressionMapLibraryState.by_id.legato.rules,
        {
          id: "rule-1",
          name: "",
          conditions: [{ id: "rule-1-condition", type: "no_op" }],
          actions: [{ id: "rule-1-action", type: "no_op" }],
        },
      ],
    });
    windowMocks.setRuleNameMock.mockImplementation(
      async (_mapId, ruleId, name) => ({
        ...expressionMapLibraryState.by_id.legato,
        rules: expressionMapLibraryState.by_id.legato.rules.map((rule) =>
          rule.id === ruleId ? { ...rule, name } : rule,
        ),
      }),
    );
    windowMocks.moveRuleMock.mockResolvedValue({
      ...expressionMapLibraryState.by_id.legato,
      rules: [
        expressionMapLibraryState.by_id.legato.rules[1],
        expressionMapLibraryState.by_id.legato.rules[0],
      ],
    });
    windowMocks.removeRuleMock.mockResolvedValue({
      ...expressionMapLibraryState.by_id.legato,
      rules: [expressionMapLibraryState.by_id.legato.rules[1]],
    });
    windowMocks.addConditionMock.mockResolvedValue(
      expressionMapLibraryState.by_id.legato,
    );
    windowMocks.setConditionTypeMock.mockResolvedValue(
      expressionMapLibraryState.by_id.legato,
    );
    windowMocks.updateConditionMock.mockResolvedValue(
      expressionMapLibraryState.by_id.legato,
    );
    windowMocks.removeConditionMock.mockResolvedValue(
      expressionMapLibraryState.by_id.legato,
    );
    windowMocks.addActionMock.mockResolvedValue(
      expressionMapLibraryState.by_id.legato,
    );
    windowMocks.setActionTypeMock.mockResolvedValue(
      expressionMapLibraryState.by_id.legato,
    );
    windowMocks.updateActionMock.mockResolvedValue(
      expressionMapLibraryState.by_id.legato,
    );
    windowMocks.removeActionMock.mockResolvedValue(
      expressionMapLibraryState.by_id.legato,
    );
    windowMocks.addDefaultActionMock.mockResolvedValue(
      expressionMapLibraryState.by_id.legato,
    );
    windowMocks.setDefaultActionTypeMock.mockResolvedValue(
      expressionMapLibraryState.by_id.legato,
    );
    windowMocks.updateDefaultActionMock.mockResolvedValue(
      expressionMapLibraryState.by_id.legato,
    );
    windowMocks.removeDefaultActionMock.mockResolvedValue(
      expressionMapLibraryState.by_id.legato,
    );
    window.history.replaceState({}, "", "/expression-map.html");
    vi.stubGlobal("prompt", vi.fn().mockReturnValue("Renamed Legato"));
  });

  it("auto-selects the first available expression map for menu launches", () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    expect(screen.getByRole("textbox", { name: "Name" })).toHaveValue("Legato");
    expect(screen.getByRole("textbox", { name: "ID" })).toHaveValue("legato");
    expect(screen.getByRole("textbox", { name: "ID" })).toBeDisabled();
    expect(screen.getByRole("textbox", { name: "Creator" })).toHaveValue(
      "Solo Compose",
    );
    expect(screen.getByText("Default Rule")).toBeInTheDocument();
  });

  it("renders the expression map list in alphabetical order", () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState: {
        order: ["staccato", "legato", "accent"],
        by_id: {
          ...expressionMapLibraryState.by_id,
          accent: {
            id: "accent",
            name: "Accent",
            creator: "Solo Compose",
            version: 1,
            default_actions: [{ id: "accent-default-action", type: "no_op" }],
            rules: [],
          },
        },
      },
    });

    const listButtons = screen.getAllByRole("button").filter((button) => {
      const label = button.textContent?.trim() ?? "";
      return ["Accent", "Legato", "Staccato"].includes(label);
    });

    expect(listButtons.map((button) => button.textContent?.trim())).toEqual([
      "Accent",
      "Legato",
      "Staccato",
    ]);
    expect(screen.getByRole("textbox", { name: "Name" })).toHaveValue("Accent");
  });

  it("retargets the shared workflow window when the selection event arrives", async () => {
    let selectionListener:
      | ((event: { payload: { expression_map_id: string } }) => void)
      | undefined;

    vi.mocked(listen).mockImplementation(async (eventName, callback) => {
      if (eventName === "soloc://expression-map/select") {
        selectionListener = callback as (event: {
          payload: { expression_map_id: string };
        }) => void;
      }

      return () => {};
    });

    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    selectionListener?.({ payload: { expression_map_id: "staccato" } });

    await waitFor(() => {
      expect(screen.getByRole("textbox", { name: "Name" })).toHaveValue(
        "Staccato",
      );
    });
  });

  it("auto-saves right-pane name edits", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    fireEvent.change(screen.getByRole("textbox", { name: "Name" }), {
      target: { value: "Legato Deluxe" },
    });

    await waitFor(() => {
      expect(windowMocks.setNameMock).toHaveBeenCalledWith(
        "legato",
        "Legato Deluxe",
      );
    });
  });

  it("auto-saves creator and version metadata edits", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    fireEvent.change(screen.getByRole("textbox", { name: "Creator" }), {
      target: { value: "Aled" },
    });
    fireEvent.change(screen.getByRole("textbox", { name: "Version" }), {
      target: { value: "4" },
    });
    fireEvent.blur(screen.getByRole("textbox", { name: "Version" }));

    await waitFor(() => {
      expect(windowMocks.setCreatorMock).toHaveBeenCalledWith("legato", "Aled");
      expect(windowMocks.setVersionMock).toHaveBeenCalledWith("legato", 4);
    });
  });

  it("supports the left-pane create action", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    fireEvent.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() => {
      expect(windowMocks.createMock).toHaveBeenCalledTimes(1);
    });
  });

  it("keeps a newly created expression map editable before store refresh", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    fireEvent.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() => {
      expect(screen.getByRole("textbox", { name: "Name" })).toHaveValue(
        "New Expression Map",
      );
    });

    fireEvent.change(screen.getByRole("textbox", { name: "Name" }), {
      target: { value: "Fresh Map" },
    });

    await waitFor(() => {
      expect(windowMocks.setNameMock).toHaveBeenCalledWith(
        "11111111-1111-4111-8111-111111111111",
        "Fresh Map",
      );
    });
  });

  it("shows an inline delete button for the selected expression map", () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    expect(
      screen.getByRole("button", { name: "Remove expression map Legato" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Remove expression map Staccato" }),
    ).not.toBeInTheDocument();
  });

  it("deletes the selected expression map from the left pane", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    fireEvent.click(
      screen.getByRole("button", { name: "Remove expression map Legato" }),
    );

    await waitFor(() => {
      expect(windowMocks.confirmMock).toHaveBeenCalledWith(
        'Delete expression map "Legato"?',
      );
      expect(windowMocks.deleteMock).toHaveBeenCalledWith("legato");
    });
  });

  it("does not delete when the native confirmation is canceled", async () => {
    windowMocks.confirmMock.mockResolvedValueOnce(false);

    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    fireEvent.click(
      screen.getByRole("button", { name: "Remove expression map Legato" }),
    );

    await waitFor(() => {
      expect(windowMocks.confirmMock).toHaveBeenCalledWith(
        'Delete expression map "Legato"?',
      );
    });

    expect(windowMocks.deleteMock).not.toHaveBeenCalled();
  });

  it("adds rules with the smallest unused default name", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    fireEvent.click(screen.getByRole("button", { name: "Add Rule" }));

    await waitFor(() => {
      expect(windowMocks.addRuleMock).toHaveBeenCalledWith("legato");
    });
  });

  it("adds a condition to the selected rule", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    fireEvent.click(screen.getByRole("button", { name: "Add Condition" }));

    await waitFor(() => {
      expect(windowMocks.addConditionMock).toHaveBeenCalledWith(
        "legato",
        "accent-keyswitch",
        "no_op",
      );
    });
  });

  it("adds an action to the selected rule", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    fireEvent.click(screen.getByRole("button", { name: "Add Action" }));

    await waitFor(() => {
      expect(windowMocks.addActionMock).toHaveBeenCalledWith(
        "legato",
        "accent-keyswitch",
        "no_op",
      );
    });
  });

  it("keeps sequential add operations on the latest in-memory rule state", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    fireEvent.click(screen.getByRole("button", { name: "Add Condition" }));

    await waitFor(() => {
      expect(windowMocks.addConditionMock).toHaveBeenCalledTimes(1);
    });

    fireEvent.click(screen.getByRole("button", { name: "Add Action" }));

    await waitFor(() => {
      expect(windowMocks.addActionMock).toHaveBeenCalledTimes(1);
    });

    expect(windowMocks.addConditionMock).toHaveBeenCalledWith(
      "legato",
      "accent-keyswitch",
      "no_op",
    );
    expect(windowMocks.addActionMock).toHaveBeenCalledWith(
      "legato",
      "accent-keyswitch",
      "no_op",
    );
  });

  it("renders a no-op condition as a placeholder row without extra controls", () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState: {
        order: ["noop-map"],
        by_id: {
          "noop-map": {
            id: "noop-map",
            name: "NoOp Map",
            creator: "Solo Compose",
            version: 1,
            default_actions: [{ id: "noop-default-action", type: "no_op" }],
            rules: [
              {
                id: "noop-rule",
                name: "Rule 1",
                conditions: [{ id: "noop-condition", type: "no_op" }],
                actions: [{ id: "noop-action", type: "no_op" }],
              },
            ],
          },
        },
      },
    });

    expect(
      screen.getByRole("button", { name: "Rule 1 condition 1 type" }),
    ).toHaveTextContent("--");
    expect(screen.queryByText("Select articulation")).not.toBeInTheDocument();
  });

  it("reuses the smallest available default rule name after gaps", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState: {
        order: ["gap-map"],
        by_id: {
          "gap-map": {
            id: "gap-map",
            name: "Gap Map",
            creator: "Solo Compose",
            version: 1,
            default_actions: [{ id: "gap-default-action", type: "no_op" }],
            rules: [
              {
                id: "gap-rule-2",
                name: "Rule 2",
                conditions: [
                  {
                    id: "gap-condition-2",
                    type: "articulation",
                    comparator: ExpressionMapEqualityComparator.Equals,
                    articulation: Articulation.Accent,
                  },
                ],
                actions: [
                  {
                    id: "gap-action-2",
                    type: "keyswitch_note",
                    midi_note: 24,
                    velocity: 96,
                  },
                ],
              },
            ],
          },
        },
      },
    });

    fireEvent.click(screen.getByRole("button", { name: "Add Rule" }));

    await waitFor(() => {
      expect(windowMocks.addRuleMock).toHaveBeenCalledWith("gap-map");
    });
  });

  it("keeps legacy no-space rule names reserved when adding a new rule", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState: {
        order: ["legacy-map"],
        by_id: {
          "legacy-map": {
            id: "legacy-map",
            name: "Legacy Map",
            creator: "Solo Compose",
            version: 1,
            default_actions: [{ id: "legacy-default-action", type: "no_op" }],
            rules: [
              {
                id: "legacy-rule-1",
                name: "Rule1",
                conditions: [
                  {
                    id: "legacy-condition-1",
                    type: "articulation",
                    comparator: ExpressionMapEqualityComparator.Equals,
                    articulation: Articulation.Accent,
                  },
                ],
                actions: [
                  {
                    id: "legacy-action-1",
                    type: "keyswitch_note",
                    midi_note: 24,
                    velocity: 96,
                  },
                ],
              },
            ],
          },
        },
      },
    });

    fireEvent.click(screen.getByRole("button", { name: "Add Rule" }));

    await waitFor(() => {
      expect(windowMocks.addRuleMock).toHaveBeenCalledWith("legacy-map");
    });
  });

  it("persists rule reordering through the shared auto-save seam", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    const sortableOptions = windowMocks.sortableCreateMock.mock
      .calls[0]?.[1] as {
      onEnd?: (event: { oldIndex?: number; newIndex?: number }) => void;
    };

    await act(async () => {
      sortableOptions.onEnd?.({ oldIndex: 0, newIndex: 1 });
    });

    await waitFor(() => {
      expect(windowMocks.moveRuleMock).toHaveBeenCalledWith(
        "legato",
        "accent-keyswitch",
        1,
      );
    });
  });

  it("removes the selected rule from the rule list via the bin icon", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    expect(
      screen.queryByRole("button", { name: "Remove rule 1" }),
    ).not.toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", { name: "Remove rule 1 from list" }),
    );

    await waitFor(() => {
      expect(windowMocks.removeRuleMock).toHaveBeenCalledWith(
        "legato",
        "accent-keyswitch",
      );
    });
  });

  it("auto-saves rule name edits", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    const ruleDetailsSection =
      screen.getByText("Rule Details").parentElement?.nextElementSibling;

    expect(ruleDetailsSection).not.toBeNull();
    expect(ruleDetailsSection).toHaveTextContent("Name");

    fireEvent.change(screen.getByRole("textbox", { name: "Rule Name" }), {
      target: { value: "Lead Accent" },
    });

    await waitFor(() => {
      expect(windowMocks.setRuleNameMock).toHaveBeenCalledWith(
        "legato",
        "accent-keyswitch",
        "Lead Accent",
      );
    });
  });

  it("persists cleared rule names for fallback display", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    fireEvent.change(screen.getByRole("textbox", { name: "Rule Name" }), {
      target: { value: "" },
    });

    await waitFor(() => {
      expect(windowMocks.setRuleNameMock).toHaveBeenCalledWith(
        "legato",
        "accent-keyswitch",
        "",
      );
    });
  });

  it("renders blank rule names as Unnamed Rule in the rule list", () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState: {
        order: ["unnamed-map"],
        by_id: {
          "unnamed-map": {
            id: "unnamed-map",
            name: "Unnamed Map",
            creator: "Solo Compose",
            version: 1,
            default_actions: [{ id: "unnamed-default-action", type: "no_op" }],
            rules: [
              {
                id: "unnamed-rule",
                name: "",
                conditions: [{ id: "unnamed-condition", type: "no_op" }],
                actions: [{ id: "unnamed-action", type: "no_op" }],
              },
            ],
          },
        },
      },
    });

    expect(screen.getByText("Unnamed Rule")).toBeInTheDocument();
  });

  it("does not render a separate default rule block", () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    expect(screen.getByText("Default Rule")).toBeInTheDocument();
  });

  it("adds a default action from the fixed default playback card", async () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    fireEvent.click(screen.getByRole("button", { name: "Default Rule" }));
    fireEvent.click(screen.getByRole("button", { name: "Add Default Action" }));

    await waitFor(() => {
      expect(windowMocks.addDefaultActionMock).toHaveBeenCalledWith(
        "legato",
        "no_op",
      );
    });
  });

  it("only highlights default playback when the default entry is selected", () => {
    renderWithProviders(<Main />, {
      expressionMapLibraryState,
    });

    fireEvent.click(screen.getByRole("button", { name: "Default Rule" }));

    expect(
      screen.queryByRole("button", { name: "Remove rule 1 from list" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Add Default Action" }),
    ).toBeInTheDocument();
  });
});
