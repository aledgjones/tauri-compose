import { mdiDelete, mdiPlus } from "@mdi/js";
import { FC } from "react";
import {
  Articulation,
  ExpressionMapAction,
  ExpressionMapCondition,
  ExpressionMapEqualityComparator,
  ExpressionMapNoteLengthComparator,
  ExpressionMapRule,
  PlayingTechniqueKind,
} from "../../services/types";
import { Input } from "../../ui/input";
import { Select } from "../../ui/select";
import { IconButton } from "../../ui/buttons/icon-button";
import {
  articulationOptions,
  conditionTypeOptions,
  equalityComparatorOptions,
  noteDurationOptions,
  noteLengthComparatorOptions,
  playingTechniqueOptions,
} from "./model";
import { ExpressionMapActionSection } from "./expression-map-action-section";

type Props = {
  displayNumber: number;
  rule: ExpressionMapRule;
  onSetRuleName: (ruleId: string, name: string) => void;
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
};

export const ExpressionMapRuleCard: FC<Props> = ({
  displayNumber,
  rule,
  onSetRuleName,
  onAddCondition,
  onSetConditionType,
  onUpdateCondition,
  onRemoveCondition,
  onAddAction,
  onSetActionType,
  onUpdateAction,
  onRemoveAction,
}) => {
  return (
    <article data-rule-number={displayNumber} className="flex flex-col gap-0.5">
      <div className="bg-neutral-900">
        <div className="flex items-center bg-neutral-800">
          <div className="flex h-9 flex-1 items-center px-2 text-sm font-medium text-white">
            Rule Details
          </div>
        </div>
        <div className="flex flex-col gap-3 p-4">
          <Input
            type="text"
            label="Name"
            ariaLabel="Rule Name"
            value={rule.name}
            onChange={(name) => onSetRuleName(rule.id, name)}
            className="w-1/2"
          />
        </div>
      </div>

      <div className="bg-neutral-900 gap-0.5 flex flex-col">
        <div className="flex items-center bg-neutral-800">
          <div className="flex h-9 flex-1 items-center px-2 text-sm font-medium text-white">
            Conditions
          </div>
          <IconButton
            path={mdiPlus}
            variant="filled"
            onClick={() => onAddCondition(rule.id)}
            aria-label="Add Condition"
            title="Add Condition"
          />
        </div>
        {rule.conditions.map((condition, conditionIndex) => (
          <div key={condition.id} className="flex gap-0.5">
            <div className="flex w-8 items-center justify-center text-sm text-neutral-300">
              {conditionIndex + 1}
            </div>

            <Select
              className="w-52"
              value={condition.type}
              options={conditionTypeOptions.map((option) => ({
                value: option.value,
                label: option.label,
              }))}
              onChange={(value) =>
                onSetConditionType(rule.id, condition.id, value)
              }
              ariaLabel={`Rule ${displayNumber} condition ${conditionIndex + 1} type`}
              variant="filled"
            />

            {condition.type === "articulation" ? (
              <>
                <Select
                  className="w-24"
                  value={condition.comparator}
                  options={equalityComparatorOptions.map((option) => ({
                    value: option.value,
                    label: option.label,
                  }))}
                  onChange={(value) =>
                    onUpdateCondition(rule.id, {
                      ...condition,
                      comparator: value as ExpressionMapEqualityComparator,
                    })
                  }
                  ariaLabel={`Rule ${displayNumber} condition ${conditionIndex + 1} comparator`}
                  variant="filled"
                />
                <Select
                  className="flex-1"
                  value={condition.articulation}
                  options={articulationOptions.map((option) => ({
                    value: option.value,
                    label: option.label,
                  }))}
                  onChange={(value) =>
                    onUpdateCondition(rule.id, {
                      ...condition,
                      articulation: value as Articulation,
                    })
                  }
                  ariaLabel={`Rule ${displayNumber} condition ${conditionIndex + 1} articulation value`}
                  variant="filled"
                  placeholder="Select articulation"
                />
              </>
            ) : condition.type === "playing_technique" ? (
              <>
                <Select
                  className="w-24"
                  value={condition.comparator}
                  options={equalityComparatorOptions.map((option) => ({
                    value: option.value,
                    label: option.label,
                  }))}
                  onChange={(value) =>
                    onUpdateCondition(rule.id, {
                      ...condition,
                      comparator: value as ExpressionMapEqualityComparator,
                    })
                  }
                  ariaLabel={`Rule ${displayNumber} condition ${conditionIndex + 1} comparator`}
                  variant="filled"
                />
                <Select
                  className="flex-1"
                  value={condition.playing_technique}
                  options={playingTechniqueOptions.map((option) => ({
                    value: option.value,
                    label: option.label,
                  }))}
                  onChange={(value) =>
                    onUpdateCondition(rule.id, {
                      ...condition,
                      playing_technique: value as PlayingTechniqueKind,
                    })
                  }
                  ariaLabel={`Rule ${displayNumber} condition ${conditionIndex + 1} playing technique value`}
                  variant="filled"
                  placeholder="Select playing technique"
                />
              </>
            ) : condition.type === "written_note_length" ? (
              <>
                <Select
                  className="w-24"
                  value={condition.comparator}
                  options={noteLengthComparatorOptions.map((option) => ({
                    value: option.value,
                    label: option.label,
                  }))}
                  onChange={(value) =>
                    onUpdateCondition(rule.id, {
                      ...condition,
                      comparator: value as ExpressionMapNoteLengthComparator,
                    })
                  }
                  ariaLabel={`Rule ${displayNumber} condition ${conditionIndex + 1} comparator`}
                  variant="filled"
                />
                <Select
                  className="flex-1"
                  value={condition.note_duration}
                  options={noteDurationOptions.map((option) => ({
                    value: option.value,
                    label: option.label,
                  }))}
                  onChange={(value) =>
                    onUpdateCondition(rule.id, {
                      ...condition,
                      note_duration: value as typeof condition.note_duration,
                    })
                  }
                  ariaLabel={`Rule ${displayNumber} condition ${conditionIndex + 1} note duration value`}
                  variant="filled"
                />
              </>
            ) : null}

            {condition.type === "no_op" && (
              <div className="flex h-9 flex-1 bg-neutral-800" />
            )}

            <IconButton
              path={mdiDelete}
              variant="filled"
              onClick={() => onRemoveCondition(rule.id, condition.id)}
              isDisabled={rule.conditions.length === 1}
              aria-label={`Remove condition ${conditionIndex + 1}`}
              title="Remove Condition"
            />
          </div>
        ))}
      </div>

      <ExpressionMapActionSection
        title="Actions"
        actions={rule.actions}
        actionAriaPrefix={`Rule ${displayNumber}`}
        addButtonAriaLabel="Add Action"
        onAddAction={() => onAddAction(rule.id)}
        onSetActionType={(actionId, actionType) =>
          onSetActionType(rule.id, actionId, actionType)
        }
        onUpdateAction={(action) => onUpdateAction(rule.id, action)}
        onRemoveAction={(actionId) => onRemoveAction(rule.id, actionId)}
      />
    </article>
  );
};
