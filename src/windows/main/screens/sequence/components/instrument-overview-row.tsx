import { FC, useMemo, useState } from "react";
import {
  Instrument,
  PlaybackRouteAddress,
  PlaybackRouteChoice,
  PlaybackRouteTarget,
} from "../../../../../services/types";
import { IconButton } from "../../../../../ui/buttons/icon-button";
import { mdiCableData, mdiSizeM, mdiSizeS } from "@mdi/js";
import { TokenizedText } from "../../../../../ui/text";
import classNames from "classnames";
import {
  useAppState,
  usePluginInventoryState,
} from "../../../../../services/state";
import { actions } from "../../../../../services/actions";
import { Select, SelectOption } from "../../../../../ui/select";
import { PlaybackReadinessBadge } from "./playback-readiness";
import {
  getPlaybackRackSlotLabel,
  getPlaybackReadinessPresentation,
} from "./playback-readiness-presentation";

interface InstrumentOverviewRowProps {
  instrument: Instrument;
  color: string;
  isDisabled: boolean;
}

type RouteOption = {
  value: string;
  slotId: number | null;
};

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

export const InstrumentOverviewRow: FC<InstrumentOverviewRowProps> = ({
  instrument,
  color,
  isDisabled,
}) => {
  const state = useAppState();
  const pluginInventory = usePluginInventoryState();
  const rack = state.score.playback?.rack ?? null;
  const routeChoicesBySlotId = state.route_target_options_by_slot_id ?? {};
  const routeState =
    state.resolved_playback_route_by_instrument_id[instrument.id];
  const presentation = getPlaybackReadinessPresentation(
    routeState,
    instrument,
    pluginInventory.entries,
    rack,
  );
  const [isUpdatingRoute, setIsUpdatingRoute] = useState(false);
  const [routeError, setRouteError] = useState<string | null>(null);

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
      options.push({
        value: String(slotId),
        label: getPlaybackRackSlotLabel(
          slotId,
          rack,
          pluginInventory.entries,
          rack.by_id[slotId]?.target?.plugin_id ?? null,
        ),
      });
    });

    return options;
  }, [pluginInventory.entries, rack]);

  const selectedRouteOptionValue = useMemo(() => {
    if (routeState?.slot_id != null && rack?.by_id[routeState.slot_id]) {
      return String(routeState.slot_id);
    }

    return "unrouted";
  }, [rack, routeState]);

  const hasVisibleSlotOptions = routeSelectOptions.some(
    (option) => option.value !== "unrouted",
  );

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
        availableChoices.length === 0 ||
        availableChoices.some((choice) =>
          routesMatchChoice(choice, currentRoute.target, currentRoute.address),
        )
      ) {
        return currentRoute;
      }
    }

    if (availableChoices.length === 0) {
      return {
        target: { mode: 0, index: null },
        address: { mode: 0, index: null },
      };
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

    if (value === selectedRouteOptionValue) {
      return;
    }

    setRouteError(null);

    setIsUpdatingRoute(true);

    try {
      await actions.ui.sequencer.setInstrument(instrument.id);

      if (value === "unrouted") {
        await actions.score.instrument.unassignRoute(instrument.id);
        return;
      }

      const selectedOption = routeOptions.find(
        (option) => option.value === value,
      );

      if (!selectedOption || selectedOption.slotId == null) {
        throw new Error("Selected slot is no longer available.");
      }

      const routePayload = resolveRoutePayload(selectedOption.slotId);

      await actions.score.instrument.assignSlotRoute(
        instrument.id,
        selectedOption.slotId,
        routePayload.target,
        routePayload.address,
      );
    } catch (error) {
      setRouteError(
        error instanceof Error ? error.message : "Failed to update route.",
      );
    } finally {
      setIsUpdatingRoute(false);
    }
  };

  return (
    <div
      tabIndex={-1}
      className={classNames(
        "flex flex-col p-2 gap-2 border-l-4 bg-neutral-900",
        { "opacity-50": isDisabled },
      )}
      style={{ borderLeftColor: isDisabled ? "#444" : color }}
    >
      {/* Instrument name */}
      <div className="flex items-center justify-between flex-1 min-w-0">
        <div className="min-w-0 flex flex-1 items-center gap-2">
          <div className="text-sm text-white truncate">
            <TokenizedText>{instrument.long_name}</TokenizedText>
            {instrument.count && <span> {instrument.count}</span>}
          </div>
          <PlaybackReadinessBadge
            presentation={presentation}
            className="max-w-full shrink"
          />
        </div>

        {!isDisabled && (
          <div className="flex items-end gap-2">
            <IconButton
              path={mdiSizeS}
              variant="filled"
              size={0.9}
              isSelected={instrument.solo}
              onClick={() =>
                actions.score.instrument.setSolo(
                  instrument.id,
                  !instrument.solo,
                )
              }
            />
            <IconButton
              path={mdiSizeM}
              variant="filled"
              size={0.9}
              isSelected={instrument.mute}
              onClick={() =>
                actions.score.instrument.setMute(
                  instrument.id,
                  !instrument.mute,
                )
              }
            />
          </div>
        )}
      </div>

      <div className="flex flex-col gap-1.5 text-xs text-neutral-300">
        <Select
          value={selectedRouteOptionValue}
          options={routeSelectOptions}
          onChange={handleRouteChange}
          variant="filled"
          icon={mdiCableData}
          placeholder={hasVisibleSlotOptions ? "Select slot" : "No score slots"}
          disabled={
            isDisabled ||
            isUpdatingRoute ||
            (!hasVisibleSlotOptions && selectedRouteOptionValue === "unrouted")
          }
        />
        {routeError ? (
          <div
            role="alert"
            className="border border-amber-800/70 bg-amber-500/10 px-3 py-2 text-xs text-amber-100"
          >
            Route update failed: {routeError}
          </div>
        ) : null}
      </div>
    </div>
  );
};
