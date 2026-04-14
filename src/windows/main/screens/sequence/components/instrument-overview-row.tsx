import { FC, useMemo, useState } from "react";
import {
  ExpressionMap,
  Instrument,
  PluginInventoryEntry,
  PlaybackRouteAddress,
  PlaybackRouteChoice,
  PlaybackRouteTarget,
  ResolvedExpressionMapLinkStatus,
} from "../../../../../services/types";
import { IconButton } from "../../../../../ui/buttons/icon-button";
import {
  mdiCableData,
  mdiOpenInNew,
  mdiSitemapOutline,
  mdiSizeM,
  mdiSizeS,
} from "@mdi/js";
import { TokenizedText } from "../../../../../ui/text";
import classNames from "classnames";
import {
  useAppState,
  useExpressionMapLibraryState,
  usePluginInventoryState,
} from "../../../../../services/state";
import { actions } from "../../../../../services/actions";
import { Select, SelectOption } from "../../../../../ui/select";
import { PlaybackReadinessBadge } from "./playback-readiness";
import {
  getPlaybackRackSlotLabel,
  getPlaybackReadinessPresentation,
} from "./playback-readiness-presentation";
import { getInstrumentOverviewRowHeight } from "./instrument-overview-layout";

interface InstrumentOverviewRowProps {
  instrument: Instrument;
  color: string;
  isDisabled: boolean;
  isRoutingExpanded?: boolean;
  onToggleRouting?: () => void;
}

type RouteOption = {
  value: string;
  slotId: number | null;
};

const UNASSIGNED_EXPRESSION_MAP_VALUE = "unassigned";

const routesMatchChoice = (
  choice: PlaybackRouteChoice,
  target: PlaybackRouteTarget,
  address: PlaybackRouteAddress,
) => {
  return (
    choice.target.mode === target.mode &&
    choice.target.index === target.index &&
    choice.address.mode === address.mode &&
    choice.address.index === address.index
  );
};

const isSupportedRouteOption = (
  availableChoices: PlaybackRouteChoice[],
  routeState:
    | {
        slot_id: number | null;
        target: PlaybackRouteTarget | null;
        address: PlaybackRouteAddress | null;
      }
    | undefined,
  slotId: number,
  isEmptySlot: boolean,
) => {
  if (availableChoices.length === 0) {
    return isEmptySlot;
  }

  if (availableChoices.length === 1) {
    return true;
  }

  if (
    routeState?.slot_id === slotId &&
    routeState.target !== null &&
    routeState.address !== null
  ) {
    const currentTarget = routeState.target;
    const currentAddress = routeState.address;

    return availableChoices.some((choice) =>
      routesMatchChoice(choice, currentTarget, currentAddress),
    );
  }

  return false;
};

const canOpenPluginEditor = (plugin: PluginInventoryEntry | null) => {
  return (
    plugin !== null &&
    plugin.availability === "available" &&
    plugin.editor_host_validation?.status !== "blocked"
  );
};

export const InstrumentOverviewRow: FC<InstrumentOverviewRowProps> = ({
  instrument,
  color,
  isDisabled,
}) => {
  const state = useAppState();
  const pluginInventory = usePluginInventoryState();
  const expressionMapLibrary = useExpressionMapLibraryState();
  const rack = state.score.playback?.rack ?? null;
  const routeChoicesBySlotId = useMemo(
    () => state.route_target_options_by_slot_id ?? {},
    [state.route_target_options_by_slot_id],
  );
  const routeState =
    state.resolved_playback_route_by_instrument_id[instrument.id];
  const supportedMixControls = routeState?.supported_mix_controls ?? {
    volume: false,
    pan: false,
    mute: false,
    solo: false,
  };
  const presentation = getPlaybackReadinessPresentation(
    routeState,
    instrument,
    pluginInventory.entries,
    rack,
  );
  const expressionMapLink = state.resolved_expression_map_by_instrument_id[
    instrument.id
  ] ?? {
    status: ResolvedExpressionMapLinkStatus.Unassigned,
    expression_map_id: null,
    expression_map_name: null,
  };
  const [isUpdatingRoute, setIsUpdatingRoute] = useState(false);
  const [isOpeningPluginEditor, setIsOpeningPluginEditor] = useState(false);
  const [isUpdatingExpressionMap, setIsUpdatingExpressionMap] = useState(false);
  const [isOpeningExpressionMapWindow, setIsOpeningExpressionMapWindow] =
    useState(false);

  const routeOptions = useMemo<RouteOption[]>(() => {
    const options: RouteOption[] = [
      {
        value: "unrouted",
        slotId: null,
      },
    ];

    if (!rack) {
      return options;
    }

    rack.order.forEach((slotId) => {
      options.push({
        value: String(slotId),
        slotId,
      });
    });

    return options;
  }, [rack]);

  const routeSelectOptions = useMemo<SelectOption<string>[]>(() => {
    const options: SelectOption<string>[] = [
      {
        value: "unrouted",
        label: "--",
      },
    ];

    if (!rack) {
      return options;
    }

    rack.order.forEach((slotId) => {
      const availableChoices = routeChoicesBySlotId[slotId] ?? [];

      options.push({
        value: String(slotId),
        label: getPlaybackRackSlotLabel(
          slotId,
          rack,
          pluginInventory.entries,
          rack.by_id[slotId]?.target?.plugin_id ?? null,
        ),
        disabled: !isSupportedRouteOption(
          availableChoices,
          routeState,
          slotId,
          rack.by_id[slotId]?.target == null,
        ),
      });
    });

    return options;
  }, [pluginInventory.entries, rack, routeChoicesBySlotId, routeState]);

  const selectedRouteOptionValue = useMemo(() => {
    if (routeState?.slot_id != null && rack?.by_id[routeState.slot_id]) {
      return String(routeState.slot_id);
    }

    return "unrouted";
  }, [rack, routeState]);

  const hasVisibleSlotOptions = routeSelectOptions.some(
    (option) => option.value !== "unrouted",
  );

  const selectedSlotId = routeState?.slot_id ?? null;
  const selectedPlugin = useMemo(() => {
    if (selectedSlotId == null) {
      return null;
    }

    const pluginId = rack?.by_id[selectedSlotId]?.target?.plugin_id ?? null;
    if (pluginId == null) {
      return null;
    }

    return (
      pluginInventory.entries.find((entry) => entry.id === pluginId) ?? null
    );
  }, [pluginInventory.entries, rack, selectedSlotId]);
  const showOpenPluginButton = canOpenPluginEditor(selectedPlugin);
  const selectedExpressionMapId = expressionMapLink.expression_map_id ?? null;
  const selectedExpressionMap =
    (selectedExpressionMapId
      ? expressionMapLibrary.by_id[selectedExpressionMapId]
      : null) ?? null;
  const expressionMapSelectOptions = useMemo<SelectOption<string>[]>(() => {
    return [
      {
        value: UNASSIGNED_EXPRESSION_MAP_VALUE,
        label: "--",
      },
      ...expressionMapLibrary.order
        .map(
          (expressionMapId: string) =>
            expressionMapLibrary.by_id[expressionMapId] ?? null,
        )
        .filter(
          (expressionMap): expressionMap is ExpressionMap =>
            expressionMap != null,
        )
        .map((expressionMap: ExpressionMap) => ({
          value: expressionMap.id,
          label: expressionMap.name,
        })),
    ];
  }, [expressionMapLibrary.by_id, expressionMapLibrary.order]);
  const selectedExpressionMapValue =
    selectedExpressionMapId ?? UNASSIGNED_EXPRESSION_MAP_VALUE;
  const hasExpressionMapRoute = routeState?.slot_id != null;
  const expressionMapSelectionLabel = selectedExpressionMap?.name ?? "--";
  const rowHeight = getInstrumentOverviewRowHeight();

  const resolveRoutePayload = (
    slotId: number,
  ): {
    target: PlaybackRouteTarget;
    address: PlaybackRouteAddress;
  } => {
    const availableChoices = routeChoicesBySlotId[slotId] ?? [];
    if (
      routeState?.slot_id === slotId &&
      routeState.target !== null &&
      routeState.address !== null
    ) {
      const currentRoute = {
        target: routeState.target,
        address: routeState.address,
      };

      if (
        availableChoices.some((choice) =>
          routesMatchChoice(choice, currentRoute.target, currentRoute.address),
        )
      ) {
        return currentRoute;
      }
    }

    if (availableChoices.length === 0) {
      if (rack?.by_id[slotId]?.target == null) {
        return {
          target: { mode: 0, index: null },
          address: { mode: 0, index: null },
        };
      }

      throw new Error("Selected slot is no longer routable.");
    }

    if (availableChoices.length === 1) {
      return {
        target: availableChoices[0].target,
        address: availableChoices[0].address,
      };
    }

    throw new Error(
      "Selected slot needs a specific route target that this picker cannot choose yet.",
    );
  };

  const handleRouteChange = async (value: string) => {
    if (isUpdatingRoute || isDisabled) {
      return;
    }

    setIsUpdatingRoute(true);

    try {
      actions.ui.sequencer.setInstrument(instrument.id);

      if (value === "unrouted") {
        await actions.score.instrument.unassignRoute(instrument.id);
        return;
      }

      const selectedRoute = routeOptions.find(
        (option) => option.value === value,
      );
      if (selectedRoute?.slotId == null) {
        throw new Error("Selected playback slot is unavailable.");
      }

      const routePayload = resolveRoutePayload(selectedRoute.slotId);
      await actions.score.instrument.assignSlotRoute(
        instrument.id,
        selectedRoute.slotId,
        routePayload.target,
        routePayload.address,
      );
    } catch (error) {
      console.error(
        `Route update failed: ${error instanceof Error ? error.message : "Failed to update route."}`,
      );
    } finally {
      setIsUpdatingRoute(false);
    }
  };

  const handleOpenPluginEditor = async () => {
    if (
      isOpeningPluginEditor ||
      isDisabled ||
      selectedPlugin == null ||
      selectedSlotId == null ||
      !showOpenPluginButton
    ) {
      return;
    }

    setIsOpeningPluginEditor(true);

    try {
      await actions.pluginInventory.openEditorHost(selectedPlugin.id, {
        slot_id: selectedSlotId,
      });
    } catch (error) {
      console.error(
        `Plugin UI failed: ${error instanceof Error ? error.message : "Failed to open the plugin UI."}`,
      );
    } finally {
      setIsOpeningPluginEditor(false);
    }
  };

  const handleExpressionMapChange = async (value: string) => {
    if (isUpdatingExpressionMap || isDisabled || !hasExpressionMapRoute) {
      return;
    }

    setIsUpdatingExpressionMap(true);

    try {
      await actions.score.instrument.setExpressionMap(
        instrument.id,
        value === UNASSIGNED_EXPRESSION_MAP_VALUE ? null : value,
      );
    } catch (error) {
      console.error(
        `Expression Map update failed: ${error instanceof Error ? error.message : "Failed to update expression map."}`,
      );
    } finally {
      setIsUpdatingExpressionMap(false);
    }
  };

  const handleOpenExpressionMapWindow = async () => {
    if (
      isOpeningExpressionMapWindow ||
      isDisabled ||
      selectedExpressionMap == null
    ) {
      return;
    }

    setIsOpeningExpressionMapWindow(true);

    try {
      await actions.openExpressionMapWindow(selectedExpressionMap.id);
    } catch (error) {
      console.error(
        `Expression Map shell failed: ${error instanceof Error ? error.message : "Failed to open the expression map shell."}`,
      );
    } finally {
      setIsOpeningExpressionMapWindow(false);
    }
  };

  return (
    <div
      tabIndex={-1}
      className={classNames(
        "flex flex-col p-2 gap-2 border-l-4 bg-neutral-900",
        { "opacity-50": isDisabled },
      )}
      style={{
        borderLeftColor: isDisabled ? "#444" : color,
        height: rowHeight,
      }}
    >
      <div className="flex min-h-9 items-center justify-between min-w-0">
        <div className="min-w-0 flex flex-1 items-center gap-2">
          <div className="truncate text-sm text-white">
            <TokenizedText>{instrument.long_name}</TokenizedText>
            {instrument.count && <span> {instrument.count}</span>}
          </div>
        </div>

        <div className="flex items-end gap-2">
          <IconButton
            path={mdiSizeS}
            variant="filled"
            size={0.9}
            isSelected={instrument.solo}
            isDisabled={isDisabled || !supportedMixControls.solo}
            aria-label={`Solo ${instrument.long_name}`}
            title={`Solo ${instrument.long_name}`}
            onClick={() =>
              actions.score.instrument.setSolo(instrument.id, !instrument.solo)
            }
          />
          <IconButton
            path={mdiSizeM}
            variant="filled"
            size={0.9}
            isSelected={instrument.mute}
            isDisabled={isDisabled || !supportedMixControls.mute}
            aria-label={`Mute ${instrument.long_name}`}
            title={`Mute ${instrument.long_name}`}
            onClick={() =>
              actions.score.instrument.setMute(instrument.id, !instrument.mute)
            }
          />
        </div>
      </div>

      <div className="flex flex-col gap-2 text-xs text-neutral-300">
        <div className="flex min-w-0 items-stretch gap-2">
          <Select
            className="min-w-0 flex-1"
            value={selectedRouteOptionValue}
            options={routeSelectOptions}
            onChange={handleRouteChange}
            ariaLabel={
              selectedRouteOptionValue === "unrouted"
                ? "--"
                : (routeSelectOptions.find(
                    (option) => option.value === selectedRouteOptionValue,
                  )?.label ?? "--")
            }
            variant="filled"
            icon={mdiCableData}
            iconSize={0.8}
            placeholder="--"
            endAdornment={
              <PlaybackReadinessBadge presentation={presentation} />
            }
            disabled={
              isDisabled ||
              isUpdatingRoute ||
              (!hasVisibleSlotOptions &&
                selectedRouteOptionValue === "unrouted")
            }
          />
          <IconButton
            path={mdiOpenInNew}
            variant="filled"
            size={0.8}
            className="shrink-0"
            isDisabled={
              isDisabled ||
              isOpeningPluginEditor ||
              !showOpenPluginButton ||
              selectedPlugin == null
            }
            onClick={handleOpenPluginEditor}
            aria-label={
              selectedPlugin
                ? `Open plugin UI for ${selectedPlugin.name}`
                : "Open plugin UI"
            }
            title={
              selectedPlugin
                ? `Open plugin UI for ${selectedPlugin.name}`
                : "Open plugin UI"
            }
          />
        </div>

        <div className="flex min-w-0 items-stretch gap-2">
          <Select
            className="min-w-0 flex-1"
            value={selectedExpressionMapValue}
            options={expressionMapSelectOptions}
            onChange={handleExpressionMapChange}
            ariaLabel={`Expression Map: ${expressionMapSelectionLabel}`}
            variant="filled"
            icon={mdiSitemapOutline}
            iconSize={0.8}
            placeholder="--"
            disabled={
              isDisabled ||
              isUpdatingExpressionMap ||
              !hasExpressionMapRoute ||
              (expressionMapSelectOptions.length === 1 &&
                selectedExpressionMapValue === UNASSIGNED_EXPRESSION_MAP_VALUE)
            }
          />
          <IconButton
            path={mdiOpenInNew}
            variant="filled"
            size={0.8}
            className="shrink-0"
            isDisabled={
              isDisabled ||
              isOpeningExpressionMapWindow ||
              selectedExpressionMap == null
            }
            onClick={handleOpenExpressionMapWindow}
            aria-label={
              selectedExpressionMap
                ? `Open expression map workflow for ${selectedExpressionMap.name}`
                : "Open expression map workflow"
            }
            title={
              selectedExpressionMap
                ? `Open expression map workflow for ${selectedExpressionMap.name}`
                : "Open expression map workflow"
            }
          />
        </div>
      </div>
    </div>
  );
};
