import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { confirm } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useState } from "react";
import { AppBar } from "../../components/app-bar";
import { actions } from "../../services/actions";
import { useExpressionMapLibraryState } from "../../services/state";
import {
  ExpressionMap,
  ExpressionMapAction,
  ExpressionMapCondition,
} from "../../services/types";
import { ExpressionMapEditorPane } from "./expression-map-editor-pane";
import { ExpressionMapLibraryPane } from "./expression-map-library-pane";
import { matchesExpressionMapSearch } from "./model";

const readInitialExpressionMapId = () => {
  const params = new URLSearchParams(window.location.search);
  return params.get("expression_map_id");
};

export const Main = () => {
  const expressionMapLibrary = useExpressionMapLibraryState();
  const [searchTerm, setSearchTerm] = useState("");
  const [selectedExpressionMapId, setSelectedExpressionMapId] = useState<
    string | null
  >(() => readInitialExpressionMapId());
  const [createdExpressionMap, setCreatedExpressionMap] =
    useState<ExpressionMap | null>(null);

  useEffect(() => {
    const show = async () => {
      await getCurrentWindow().show();
    };

    show();
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    listen<{ expression_map_id: string }>(
      "soloc://expression-map/select",
      (event) => {
        setSelectedExpressionMapId(event.payload.expression_map_id || null);
      },
    ).then((cleanup) => {
      unlisten = cleanup;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const visibleExpressionMaps = useMemo(() => {
    return expressionMapLibrary.order
      .map(
        (expressionMapId) =>
          expressionMapLibrary.by_id[expressionMapId] ?? null,
      )
      .filter(
        (expressionMap): expressionMap is ExpressionMap =>
          expressionMap != null,
      )
      .filter((expressionMap) =>
        matchesExpressionMapSearch(expressionMap.name, searchTerm),
      )
      .sort((left, right) => {
        const nameComparison = left.name.localeCompare(right.name, undefined, {
          sensitivity: "base",
        });

        if (nameComparison !== 0) {
          return nameComparison;
        }

        return left.id.localeCompare(right.id, undefined, {
          sensitivity: "base",
        });
      });
  }, [expressionMapLibrary.by_id, expressionMapLibrary.order, searchTerm]);

  const effectiveSelectedExpressionMapId =
    selectedExpressionMapId ?? visibleExpressionMaps[0]?.id ?? null;

  const selectedExpressionMapFromStore =
    (effectiveSelectedExpressionMapId
      ? expressionMapLibrary.by_id[effectiveSelectedExpressionMapId]
      : null) ?? null;

  const selectedExpressionMap =
    selectedExpressionMapFromStore ??
    (effectiveSelectedExpressionMapId != null &&
    createdExpressionMap?.id === effectiveSelectedExpressionMapId
      ? createdExpressionMap
      : null);

  const handleCreate = async () => {
    const created = await actions.expressionMaps.create();
    setCreatedExpressionMap(created);
    setSelectedExpressionMapId(created.id);
  };

  const handleDeleteExpressionMap = async (expressionMapId: string) => {
    const expressionMap = expressionMapLibrary.by_id[expressionMapId];

    if (!expressionMap) {
      return;
    }

    const shouldDelete = await confirm(
      `Delete expression map \"${expressionMap.name}\"?`,
    );

    if (!shouldDelete) {
      return;
    }

    setSelectedExpressionMapId((currentExpressionMapId) =>
      currentExpressionMapId === expressionMapId
        ? null
        : currentExpressionMapId,
    );
    await actions.expressionMaps.delete(expressionMapId);
  };

  const handleRenameFromEditor = async (name: string) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.setName(selectedExpressionMap.id, name);
  };

  const handleChangeCreatorFromEditor = async (creator: string) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.setCreator(selectedExpressionMap.id, creator);
  };

  const handleChangeVersionFromEditor = async (version: number) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.setVersion(selectedExpressionMap.id, version);
  };

  const handleSetRuleName = async (ruleId: string, name: string) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.setRuleName(
      selectedExpressionMap.id,
      ruleId,
      name,
    );
  };

  const handleAddRule = async () => {
    if (!selectedExpressionMap) {
      return null;
    }

    const updatedExpressionMap = await actions.expressionMaps.addRule(
      selectedExpressionMap.id,
    );

    if (!updatedExpressionMap) {
      return null;
    }

    const updatedRules = updatedExpressionMap?.rules ?? [];
    return updatedRules[0]?.id ?? null;
  };

  const handleMoveRule = async (ruleId: string, newIndex: number) => {
    if (!selectedExpressionMap) {
      return;
    }

    const currentIndex = selectedExpressionMap.rules.findIndex(
      (rule) => rule.id === ruleId,
    );

    if (currentIndex < 0 || currentIndex === newIndex) {
      return;
    }

    await actions.expressionMaps.moveRule(
      selectedExpressionMap.id,
      ruleId,
      newIndex,
    );
  };

  const handleRemoveRule = async (ruleId: string) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.removeRule(selectedExpressionMap.id, ruleId);
  };

  const handleAddCondition = async (ruleId: string) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.addCondition(
      selectedExpressionMap.id,
      ruleId,
      "no_op",
    );
  };

  const handleSetConditionType = async (
    ruleId: string,
    conditionId: string,
    conditionType: ExpressionMapCondition["type"],
  ) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.setConditionType(
      selectedExpressionMap.id,
      ruleId,
      conditionId,
      conditionType,
    );
  };

  const handleUpdateCondition = async (
    ruleId: string,
    condition: ExpressionMapCondition,
  ) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.updateCondition(
      selectedExpressionMap.id,
      ruleId,
      condition,
    );
  };

  const handleRemoveCondition = async (ruleId: string, conditionId: string) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.removeCondition(
      selectedExpressionMap.id,
      ruleId,
      conditionId,
    );
  };

  const handleAddAction = async (ruleId: string) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.addAction(
      selectedExpressionMap.id,
      ruleId,
      "no_op",
    );
  };

  const handleSetActionType = async (
    ruleId: string,
    actionId: string,
    actionType: ExpressionMapAction["type"],
  ) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.setActionType(
      selectedExpressionMap.id,
      ruleId,
      actionId,
      actionType,
    );
  };

  const handleUpdateAction = async (
    ruleId: string,
    action: ExpressionMapAction,
  ) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.updateAction(
      selectedExpressionMap.id,
      ruleId,
      action,
    );
  };

  const handleRemoveAction = async (ruleId: string, actionId: string) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.removeAction(
      selectedExpressionMap.id,
      ruleId,
      actionId,
    );
  };

  const handleAddDefaultAction = async () => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.addDefaultAction(
      selectedExpressionMap.id,
      "no_op",
    );
  };

  const handleSetDefaultActionType = async (
    actionId: string,
    actionType: ExpressionMapAction["type"],
  ) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.setDefaultActionType(
      selectedExpressionMap.id,
      actionId,
      actionType,
    );
  };

  const handleUpdateDefaultAction = async (action: ExpressionMapAction) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.updateDefaultAction(
      selectedExpressionMap.id,
      action,
    );
  };

  const handleRemoveDefaultAction = async (actionId: string) => {
    if (!selectedExpressionMap) {
      return;
    }

    await actions.expressionMaps.removeDefaultAction(
      selectedExpressionMap.id,
      actionId,
    );
  };

  return (
    <div className="grid h-dvh grid-rows-[28px_1fr] gap-0.5 overflow-hidden bg-neutral-950 p-0.75 text-white">
      <AppBar title="Expression Map" />

      <div className="grid min-h-0 grid-cols-[320px_minmax(0,1fr)] gap-0.5 overflow-hidden bg-neutral-950">
        <ExpressionMapLibraryPane
          searchTerm={searchTerm}
          onSearchTermChange={setSearchTerm}
          expressionMaps={visibleExpressionMaps}
          selectedExpressionMapId={effectiveSelectedExpressionMapId}
          onSelectExpressionMap={setSelectedExpressionMapId}
          onDeleteExpressionMap={handleDeleteExpressionMap}
          onCreate={handleCreate}
        />
        <ExpressionMapEditorPane
          expressionMap={selectedExpressionMap}
          onChangeName={handleRenameFromEditor}
          onChangeCreator={handleChangeCreatorFromEditor}
          onChangeVersion={handleChangeVersionFromEditor}
          onAddRule={handleAddRule}
          onSetRuleName={handleSetRuleName}
          onMoveRule={handleMoveRule}
          onRemoveRule={handleRemoveRule}
          onAddCondition={handleAddCondition}
          onSetConditionType={handleSetConditionType}
          onUpdateCondition={handleUpdateCondition}
          onRemoveCondition={handleRemoveCondition}
          onAddAction={handleAddAction}
          onSetActionType={handleSetActionType}
          onUpdateAction={handleUpdateAction}
          onRemoveAction={handleRemoveAction}
          onAddDefaultAction={handleAddDefaultAction}
          onSetDefaultActionType={handleSetDefaultActionType}
          onUpdateDefaultAction={handleUpdateDefaultAction}
          onRemoveDefaultAction={handleRemoveDefaultAction}
        />
      </div>
    </div>
  );
};
