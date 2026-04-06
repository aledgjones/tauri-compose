import { mdiDelete, mdiPlus } from "@mdi/js";
import { FC } from "react";
import { ExpressionMapAction } from "../../services/types";
import { IconButton } from "../../ui/buttons/icon-button";
import { Input } from "../../ui/input";
import { Select } from "../../ui/select";
import { actionTypeOptions } from "./model";

type Props = {
  title: string;
  actions: ExpressionMapAction[];
  actionAriaPrefix: string;
  addButtonAriaLabel: string;
  emptyStateTitle?: string;
  emptyStateDetail?: string;
  helperText?: string;
  onAddAction: () => void;
  onSetActionType: (
    actionId: string,
    actionType: ExpressionMapAction["type"],
  ) => void;
  onUpdateAction: (action: ExpressionMapAction) => void;
  onRemoveAction: (actionId: string) => void;
  canRemoveAction?: (actionCount: number) => boolean;
};

export const ExpressionMapActionSection: FC<Props> = ({
  title,
  actions,
  actionAriaPrefix,
  addButtonAriaLabel,
  emptyStateTitle,
  emptyStateDetail,
  helperText,
  onAddAction,
  onSetActionType,
  onUpdateAction,
  onRemoveAction,
  canRemoveAction = (actionCount) => actionCount > 1,
}) => {
  const clampMidiValue = (value: number) => Math.max(0, Math.min(127, value));

  return (
    <div className="bg-neutral-900 gap-0.5 flex flex-col">
      <div className="flex items-center bg-neutral-800">
        <div className="flex h-9 flex-1 items-center px-2 text-sm font-medium text-white">
          {title}
        </div>
        <IconButton
          path={mdiPlus}
          variant="filled"
          onClick={onAddAction}
          aria-label={addButtonAriaLabel}
          title={addButtonAriaLabel}
        />
      </div>

      {helperText ? (
        <div className="px-4 py-3 text-sm text-neutral-400">{helperText}</div>
      ) : null}

      {actions.length === 0 ? (
        <div className="flex min-h-32 items-center justify-center px-6 text-center">
          <div>
            <div className="text-sm text-neutral-200">
              {emptyStateTitle ?? "No actions yet."}
            </div>
            {emptyStateDetail ? (
              <div className="mt-2 text-sm text-neutral-400">
                {emptyStateDetail}
              </div>
            ) : null}
          </div>
        </div>
      ) : (
        actions.map((action, actionIndex) => (
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
              onChange={(value) =>
                onSetActionType(action.id, value as ExpressionMapAction["type"])
              }
              ariaLabel={`${actionAriaPrefix} action ${actionIndex + 1} type`}
              variant="filled"
            />

            {action.type === "keyswitch_note" ? (
              <>
                <Input
                  type="number"
                  className="flex-1"
                  ariaLabel={`${actionAriaPrefix} action ${actionIndex + 1} keyswitch note`}
                  value={action.midi_note}
                  onChange={(value) => {
                    onUpdateAction({
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
                  ariaLabel={`${actionAriaPrefix} action ${actionIndex + 1} keyswitch velocity`}
                  value={action.velocity}
                  onChange={(value) => {
                    onUpdateAction({
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
            ) : null}

            {action.type === "control_change" ? (
              <>
                <Input
                  type="number"
                  className="flex-1"
                  ariaLabel={`${actionAriaPrefix} action ${actionIndex + 1} controller`}
                  value={action.controller}
                  onChange={(value) => {
                    onUpdateAction({
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
                  ariaLabel={`${actionAriaPrefix} action ${actionIndex + 1} value`}
                  value={action.value}
                  onChange={(value) => {
                    onUpdateAction({
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
            ) : null}

            {action.type === "note_length_transform" ? (
              <Input
                type="number"
                ariaLabel={`${actionAriaPrefix} action ${actionIndex + 1} note-length transform ratio`}
                value={action.ratio_percent}
                onChange={(value) => {
                  onUpdateAction({
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
            ) : null}

            {action.type === "no_op" && (
              <div className="flex h-9 flex-1 bg-neutral-800" />
            )}

            <IconButton
              path={mdiDelete}
              variant="filled"
              onClick={() => onRemoveAction(action.id)}
              isDisabled={!canRemoveAction(actions.length)}
              aria-label={`Remove ${actionAriaPrefix.toLowerCase()} action ${actionIndex + 1}`}
              title="Remove Action"
            />
          </div>
        ))
      )}
    </div>
  );
};
