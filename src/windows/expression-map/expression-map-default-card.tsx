import { FC } from "react";
import { ExpressionMapAction } from "../../services/types";
import { ExpressionMapActionSection } from "./expression-map-action-section";

type Props = {
  actions: ExpressionMapAction[];
  onAddAction: () => void;
  onSetActionType: (
    actionId: string,
    actionType: ExpressionMapAction["type"],
  ) => void;
  onUpdateAction: (action: ExpressionMapAction) => void;
  onRemoveAction: (actionId: string) => void;
};

export const ExpressionMapDefaultCard: FC<Props> = ({
  actions,
  onAddAction,
  onSetActionType,
  onUpdateAction,
  onRemoveAction,
}) => {
  return (
    <article className="flex flex-col gap-0.5">
      <div className="bg-neutral-900">
        <div className="flex items-center bg-neutral-800">
          <div className="flex h-9 flex-1 items-center px-2 text-sm font-medium text-white">
            Default Rule
          </div>
        </div>
        <div className="p-4 text-sm text-neutral-400">
          These actions run when none of the other rules match.
        </div>
      </div>

      <ExpressionMapActionSection
        title="Actions"
        actions={actions}
        actionAriaPrefix="Default Rule"
        addButtonAriaLabel="Add Default Action"
        onAddAction={onAddAction}
        onSetActionType={onSetActionType}
        onUpdateAction={onUpdateAction}
        onRemoveAction={onRemoveAction}
      />
    </article>
  );
};
