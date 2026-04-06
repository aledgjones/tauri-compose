import { mdiDelete, mdiPlus } from "@mdi/js";
import { FC } from "react";
import {
  Articulation,
  ExpressionMapAction,
  ExpressionMapCondition,
  ExpressionMapEqualityComparator,
  ExpressionMapNoteLengthComparator,
  ExpressionMapRule,
} from "../../services/types";
import { IconButton } from "../../ui/buttons/icon-button";
import { Input } from "../../ui/input";
import { Select } from "../../ui/select";
import {
  actionTypeOptions,
  articulationOptions,
  conditionTypeOptions,
  equalityComparatorOptions,
  noteDurationOptions,
  noteLengthComparatorOptions,
} from "./model";

type Props = {
  index: number;
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
  index,
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
  const clampMidiValue = (value: number) => Math.max(0, Math.min(127, value));

  const getConditionComparatorOptions = (condition: ExpressionMapCondition) =>
    condition.type === "articulation"
      ? equalityComparatorOptions
      : noteLengthComparatorOptions;

  return (
    <article data-rule-index={index} className="flex flex-col gap-0.5">
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
              ariaLabel={`Rule ${index + 1} condition ${conditionIndex + 1} type`}
              variant="filled"
            />

            <Select
              className="w-24"
              value={condition.comparator}
              options={getConditionComparatorOptions(condition).map(
                (option) => ({
                  value: option.value,
                  label: option.label,
                }),
              )}
              onChange={(value) => {
                if (condition.type === "articulation") {
                  onUpdateCondition(rule.id, {
                    ...condition,
                    comparator: value as ExpressionMapEqualityComparator,
                  });
                  return;
                }

                onUpdateCondition(rule.id, {
                  ...condition,
                  comparator: value as ExpressionMapNoteLengthComparator,
                });
              }}
              ariaLabel={`Rule ${index + 1} condition ${conditionIndex + 1} comparator`}
              variant="filled"
            />

            {condition.type === "articulation" ? (
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
                ariaLabel={`Rule ${index + 1} condition ${conditionIndex + 1} articulation value`}
                variant="filled"
                placeholder="Select articulation"
              />
            ) : (
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
                ariaLabel={`Rule ${index + 1} condition ${conditionIndex + 1} note duration value`}
                variant="filled"
              />
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

      <div className="bg-neutral-900 gap-0.5 flex flex-col">
        <div className="flex items-center bg-neutral-800">
          <div className="flex h-9 flex-1 items-center px-2 text-sm font-medium text-white">
            Actions
          </div>
          <IconButton
            path={mdiPlus}
            variant="filled"
            onClick={() => onAddAction(rule.id)}
            aria-label="Add Action"
            title="Add Action"
          />
        </div>

        {rule.actions.map((action, actionIndex) => (
          <div key={action.id} className="flex gap-0.5">
            <div className="flex w-8 items-center justify-center text-sm text-neutral-300">
              {actionIndex + 1}
            </div>

            <Select
              className="w-52"
              value={action.type}
              options={actionTypeOptions.map((option) => ({
                value: option.value,
                label: option.label,
              }))}
              onChange={(value) => onSetActionType(rule.id, action.id, value)}
              ariaLabel={`Rule ${index + 1} action ${actionIndex + 1} type`}
              variant="filled"
            />

            {action.type === "keyswitch_note" && (
              <>
                <Input
                  type="number"
                  className="flex-1"
                  ariaLabel={`Rule ${index + 1} action ${actionIndex + 1} keyswitch note`}
                  value={action.midi_note}
                  onChange={(value) => {
                    onUpdateAction(rule.id, {
                      ...action,
                      midi_note: clampMidiValue(value ?? 24),
                    });
                  }}
                  defaultValue={24}
                  step={1}
                  decimalPlaces={0}
                  min={0}
                  max={127}
                />
                <Input
                  type="number"
                  className="flex-1"
                  ariaLabel={`Rule ${index + 1} action ${actionIndex + 1} keyswitch velocity`}
                  value={action.velocity}
                  onChange={(value) => {
                    onUpdateAction(rule.id, {
                      ...action,
                      velocity: clampMidiValue(value ?? 127),
                    });
                  }}
                  defaultValue={127}
                  step={1}
                  decimalPlaces={0}
                  min={0}
                  max={127}
                />
              </>
            )}

            {action.type === "control_change" && (
              <>
                <Input
                  type="number"
                  className="flex-1"
                  ariaLabel={`Rule ${index + 1} action ${actionIndex + 1} controller`}
                  value={action.controller}
                  onChange={(value) => {
                    onUpdateAction(rule.id, {
                      ...action,
                      controller: clampMidiValue(value ?? 1),
                    });
                  }}
                  defaultValue={1}
                  step={1}
                  decimalPlaces={0}
                  min={0}
                  max={127}
                />
                <Input
                  type="number"
                  className="flex-1"
                  ariaLabel={`Rule ${index + 1} action ${actionIndex + 1} value`}
                  value={action.value}
                  onChange={(value) => {
                    onUpdateAction(rule.id, {
                      ...action,
                      value: clampMidiValue(value ?? 64),
                    });
                  }}
                  defaultValue={64}
                  step={1}
                  decimalPlaces={0}
                  min={0}
                  max={127}
                />
              </>
            )}

            {action.type === "note_length_transform" && (
              <Input
                type="number"
                ariaLabel={`Rule ${index + 1} action ${actionIndex + 1} note-length transform ratio`}
                value={action.ratio_percent}
                onChange={(value) => {
                  onUpdateAction(rule.id, {
                    ...action,
                    ratio_percent: Math.max(1, value ?? 100),
                  });
                }}
                defaultValue={100}
                step={1}
                decimalPlaces={0}
                min={1}
                units="%"
                className="flex-1"
              />
            )}

            {action.type === "no_op" && (
              <div className="flex h-9 items-center px-2 flex-1 text-sm text-neutral-500 bg-neutral-800 italic">
                Plugin default playback
              </div>
            )}

            <IconButton
              path={mdiDelete}
              variant="filled"
              onClick={() => onRemoveAction(rule.id, action.id)}
              isDisabled={rule.actions.length === 1}
              aria-label={`Remove action ${actionIndex + 1}`}
              title="Remove Action"
            />
          </div>
        ))}
      </div>
    </article>
  );
};
