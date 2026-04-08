import { mdiDelete, mdiPlus } from "@mdi/js";
import { FC, useEffect, useEffectEvent, useRef, useState } from "react";
import Sortable from "sortablejs";
import {
  ExpressionMap,
  ExpressionMapAction,
  ExpressionMapCondition,
  ExpressionMapRule,
} from "../../services/types";
import { IconButton } from "../../ui/buttons/icon-button";
import { ExpressionMapDefaultCard } from "./expression-map-default-card";
import { Input } from "../../ui/input";
import { ExpressionMapRuleCard } from "./expression-map-rule-card";
import { getDefaultRuleFallbackName } from "./model";

import "./expression-map-editor-pane.css";

const DEFAULT_PLAYBACK_ENTRY_ID = "__default_playback__";

type Props = {
  expressionMap: ExpressionMap | null;
  onChangeName: (name: string) => void;
  onChangeCreator: (creator: string) => void;
  onChangeVersion: (version: number) => void;
  onAddRule: () => Promise<string | null>;
  onSetRuleName: (ruleId: string, name: string) => void;
  onMoveRule: (ruleId: string, newIndex: number) => void;
  onRemoveRule: (ruleId: string) => void;
  onAddCondition: (ruleId: string) => void;
  onSetConditionType: (
    ruleId: string,
    conditionId: string,
    conditionType: ExpressionMapCondition["type"],
  ) => void;
  onUpdateCondition: (
    ruleId: string,
    condition: ExpressionMapCondition,
  ) => void;
  onRemoveCondition: (ruleId: string, conditionId: string) => void;
  onAddAction: (ruleId: string) => void;
  onSetActionType: (
    ruleId: string,
    actionId: string,
    actionType: ExpressionMapAction["type"],
  ) => void;
  onUpdateAction: (ruleId: string, action: ExpressionMapAction) => void;
  onRemoveAction: (ruleId: string, actionId: string) => void;
  onAddDefaultAction: () => void;
  onSetDefaultActionType: (
    actionId: string,
    actionType: ExpressionMapAction["type"],
  ) => void;
  onUpdateDefaultAction: (action: ExpressionMapAction) => void;
  onRemoveDefaultAction: (actionId: string) => void;
};

type SelectionState = {
  expressionMapId: string | null;
  entryId: string | null;
};

export const ExpressionMapEditorPane: FC<Props> = ({
  expressionMap,
  onChangeName,
  onChangeCreator,
  onChangeVersion,
  onAddRule,
  onSetRuleName,
  onMoveRule,
  onRemoveRule,
  onAddCondition,
  onSetConditionType,
  onUpdateCondition,
  onRemoveCondition,
  onAddAction,
  onSetActionType,
  onUpdateAction,
  onRemoveAction,
  onAddDefaultAction,
  onSetDefaultActionType,
  onUpdateDefaultAction,
  onRemoveDefaultAction,
}) => {
  const ruleListRef = useRef<HTMLDivElement | null>(null);
  const [selectionState, setSelectionState] = useState<SelectionState>({
    expressionMapId: null,
    entryId: null,
  });

  const expressionMapId = expressionMap?.id ?? null;

  const selectedEntryId =
    expressionMapId != null &&
    selectionState.expressionMapId === expressionMapId
      ? selectionState.entryId
      : null;

  const effectiveSelectedEntryId =
    expressionMap == null
      ? null
      : selectedEntryId === DEFAULT_PLAYBACK_ENTRY_ID
        ? DEFAULT_PLAYBACK_ENTRY_ID
        : selectedEntryId != null &&
            expressionMap.rules.some((rule) => rule.id === selectedEntryId)
          ? selectedEntryId
          : (expressionMap.rules[0]?.id ?? DEFAULT_PLAYBACK_ENTRY_ID);

  const selectedRuleId =
    effectiveSelectedEntryId != null &&
    effectiveSelectedEntryId !== DEFAULT_PLAYBACK_ENTRY_ID
      ? effectiveSelectedEntryId
      : null;

  const setSelectedRuleId = (ruleId: string | null) => {
    if (!expressionMap) {
      return;
    }

    setSelectionState({
      expressionMapId: expressionMap.id,
      entryId: ruleId,
    });
  };

  const setDefaultSelected = () => {
    if (!expressionMap) {
      return;
    }

    setSelectionState({
      expressionMapId: expressionMap.id,
      entryId: DEFAULT_PLAYBACK_ENTRY_ID,
    });
  };

  const handleSortEnd = useEffectEvent(
    (event: { oldIndex?: number; newIndex?: number }) => {
      const { oldIndex, newIndex } = event;

      if (!expressionMap || oldIndex === undefined || newIndex === undefined) {
        return;
      }

      setSelectionState({
        expressionMapId: expressionMap.id,
        entryId: effectiveSelectedEntryId,
      });

      const movedRuleId = expressionMap.rules[oldIndex]?.id;

      if (!movedRuleId) {
        return;
      }

      onMoveRule(movedRuleId, newIndex);
    },
  );

  useEffect(() => {
    if (!expressionMapId || !ruleListRef.current) {
      return;
    }

    const sortable = Sortable.create(ruleListRef.current, {
      group: "expression-map-rules",
      animation: 150,
      forceFallback: true,
      fallbackOnBody: true,
      fallbackClass: "drag-expression-map-rule",
      ghostClass: "ghost-expression-map-rule",
      delay: 150,
      onEnd: handleSortEnd,
    });

    return () => {
      sortable.destroy();
    };
  }, [expressionMapId]);

  if (!expressionMap) {
    return (
      <section className="flex min-h-0 items-center justify-center rounded-br-xl bg-neutral-900 px-6 text-center">
        <div>
          <div className="text-sm text-neutral-200">
            No expression map selected.
          </div>
          <div className="mt-2 text-sm text-neutral-400">
            Choose or create a reusable map in the library pane to start
            authoring rule behavior.
          </div>
        </div>
      </section>
    );
  }

  const selectedRule =
    selectedRuleId == null
      ? null
      : (expressionMap.rules.find((rule) => rule.id === selectedRuleId) ??
        null);

  const effectiveSelectionKind: "default" | "rule" =
    effectiveSelectedEntryId === DEFAULT_PLAYBACK_ENTRY_ID ? "default" : "rule";

  const selectedRuleDisplayNumber =
    selectedRule == null
      ? null
      : expressionMap.rules.findIndex((rule) => rule.id === selectedRule.id) +
        1;

  const formatRuleLabel = (rule: ExpressionMapRule) => {
    const trimmedName = rule.name.trim();

    return trimmedName.length > 0 ? trimmedName : getDefaultRuleFallbackName();
  };

  return (
    <section className="flex min-h-0 flex-col gap-0.5 overflow-auto rounded-br-xl">
      <div className="bg-neutral-900">
        <h3 className="bg-neutral-800 p-2 text-sm font-medium text-white">
          Expression Map Data
        </h3>
        <div className="flex flex-col gap-3 p-4">
          <Input
            type="text"
            label="Name"
            ariaLabel="Name"
            value={expressionMap.name}
            onChange={onChangeName}
            className="w-1/2"
          />
          <Input
            type="text"
            label="Creator"
            ariaLabel="Creator"
            value={expressionMap.creator}
            onChange={onChangeCreator}
            className="w-1/2"
          />
          <div className="flex w-1/2 gap-2">
            <Input
              type="text"
              label="ID"
              ariaLabel="ID"
              value={expressionMap.id}
              onChange={() => {}}
              disabled
              className="flex-1"
            />
            <Input
              type="number"
              label="Version"
              ariaLabel="Version"
              value={expressionMap.version}
              onChange={(value) => onChangeVersion(value ?? 1)}
              defaultValue={1}
              min={1}
              step={1}
              decimalPlaces={0}
              className="w-32"
            />
          </div>
        </div>
      </div>

      <div className="grid gap-0.5 xl:grid-cols-[280px_minmax(0,1fr)]">
        <div className="flex min-h-0 flex-col bg-neutral-900">
          <div className="flex items-center gap-0.5 bg-neutral-800">
            <div className="flex h-9 flex-1 items-center px-2 text-sm font-medium text-white">
              Rule List
            </div>
            <IconButton
              path={mdiPlus}
              variant="filled"
              onClick={() => {
                void onAddRule().then((ruleId) => {
                  if (ruleId != null) {
                    setSelectedRuleId(ruleId);
                  }
                });
              }}
              aria-label="Add Rule"
              title="Add Rule"
            />
          </div>

          {expressionMap.rules.length === 0 ? (
            <div className="px-6 py-6 text-center bg-neutral-900 text-sm text-neutral-400">
              Add a rule when you need conditional behavior.
            </div>
          ) : null}

          <div ref={ruleListRef} className="flex flex-col gap-0.5 pt-0.5">
            {expressionMap.rules.map((rule, index) => {
              const isSelected = rule.id === selectedRuleId;

              return (
                <div
                  key={rule.id}
                  className={`border-l-4 w-full pl-3 text-left ${
                    isSelected
                      ? "bg-neutral-800 border-sky-500"
                      : "bg-neutral-900 hover:bg-neutral-800 border-transparent"
                  }`}
                >
                  <div className="flex items-center gap-2 h-9">
                    <button
                      type="button"
                      onClick={() => setSelectedRuleId(rule.id)}
                      className="min-w-0 flex-1 text-left"
                    >
                      <div className="text-sm text-white">
                        {formatRuleLabel(rule)}
                      </div>
                    </button>
                    {isSelected ? (
                      <IconButton
                        path={mdiDelete}
                        variant="filled"
                        onClick={() => onRemoveRule(rule.id)}
                        onPointerDown={(event) => {
                          event.stopPropagation();
                        }}
                        aria-label={`Remove rule ${index + 1} from list`}
                        title="Remove Rule"
                      />
                    ) : null}
                  </div>
                </div>
              );
            })}
          </div>

          <div className="pt-0.5">
            <div
              className={`border-l-4 w-full pl-3 text-left ${
                effectiveSelectionKind === "default"
                  ? "bg-neutral-800 border-sky-500"
                  : "bg-neutral-900 hover:bg-neutral-800 border-transparent"
              }`}
            >
              <div className="flex items-center gap-2 h-9">
                <button
                  type="button"
                  onClick={setDefaultSelected}
                  className="min-w-0 flex-1 text-left"
                >
                  <div className="text-sm text-white">Default Rule</div>
                </button>
              </div>
            </div>
          </div>
        </div>

        <div className="min-w-0 flex flex-col gap-0.5">
          {effectiveSelectionKind === "default" ? (
            <ExpressionMapDefaultCard
              actions={expressionMap.default_actions}
              onAddAction={onAddDefaultAction}
              onSetActionType={onSetDefaultActionType}
              onUpdateAction={onUpdateDefaultAction}
              onRemoveAction={onRemoveDefaultAction}
            />
          ) : selectedRule && selectedRuleDisplayNumber != null ? (
            <ExpressionMapRuleCard
              key={selectedRule.id}
              displayNumber={selectedRuleDisplayNumber}
              rule={selectedRule}
              onSetRuleName={onSetRuleName}
              onAddCondition={onAddCondition}
              onSetConditionType={onSetConditionType}
              onUpdateCondition={onUpdateCondition}
              onRemoveCondition={onRemoveCondition}
              onAddAction={onAddAction}
              onSetActionType={onSetActionType}
              onUpdateAction={onUpdateAction}
              onRemoveAction={onRemoveAction}
            />
          ) : (
            <div className="flex min-h-48 items-center justify-center bg-neutral-900 px-6 text-center">
              <div>
                <div className="text-sm text-neutral-200">
                  No rule selected.
                </div>
                <div className="mt-2 text-sm text-neutral-400">
                  Choose a rule from the list to edit its conditions and
                  actions.
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </section>
  );
};
