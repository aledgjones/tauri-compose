import classNames from "classnames";
import { FC, useMemo, useState } from "react";
import {
  useAppState,
  usePluginInventoryState,
} from "../../../../../services/state";
import {
  Instrument,
  PluginInventoryEntry,
  ResolvedPlaybackRoute,
} from "../../../../../services/types";
import { actions } from "../../../../../services/actions";
import { Select, SelectOption } from "../../../../../ui/select";
import { TokenizedText } from "../../../../../ui/text";
import { PlaybackReadinessBadge } from "./playback-readiness";
import {
  getPlaybackRackSlotLabel,
  getPlaybackReadinessPresentation,
} from "./playback-readiness-presentation";

const isUsablePreparationPlugin = (plugin: PluginInventoryEntry) => {
  return (
    plugin.availability === "available" &&
    plugin.validation_failure?.kind !== "unsupported_editor_host"
  );
};

const compareUsablePlugins = (
  left: PluginInventoryEntry,
  right: PluginInventoryEntry,
) => {
  const nameComparison = left.name.localeCompare(right.name, undefined, {
    sensitivity: "base",
  });

  if (nameComparison !== 0) {
    return nameComparison;
  }

  return left.id.localeCompare(right.id);
};

type PlaybackRackPanelProps = {
  createError?: string | null;
  instrument: Instrument | null;
  route: ResolvedPlaybackRoute | null;
};

export const PlaybackRackPanel: FC<PlaybackRackPanelProps> = ({
  createError = null,
  instrument,
  route,
}) => {
  const state = useAppState();
  const pluginInventory = usePluginInventoryState();
  const rack = state.score.playback?.rack ?? null;
  const usablePlugins = useMemo(
    () =>
      pluginInventory.entries
        .filter(isUsablePreparationPlugin)
        .sort(compareUsablePlugins),
    [pluginInventory.entries],
  );
  const pluginOptions = useMemo<SelectOption<string>[]>(
    () =>
      usablePlugins.map((plugin) => ({ value: plugin.id, label: plugin.name })),
    [usablePlugins],
  );
  const [assigningSlotId, setAssigningSlotId] = useState<number | null>(null);
  const [assignErrorBySlotId, setAssignErrorBySlotId] = useState<
    Record<number, string>
  >({});

  const presentation = useMemo(() => {
    if (!instrument) {
      return null;
    }

    return getPlaybackReadinessPresentation(
      route,
      instrument,
      pluginInventory.entries,
      rack,
    );
  }, [instrument, pluginInventory.entries, rack, route]);

  const instrumentLabel = instrument
    ? instrument.count
      ? `${instrument.long_name} ${instrument.count}`
      : instrument.long_name
    : null;
  const focusedSlotId = route?.slot_id ?? null;
  const focusedSlotMissing =
    focusedSlotId !== null && rack?.by_id[focusedSlotId] === undefined;
  const slotIds = useMemo(() => rack?.order ?? [], [rack]);

  const handleSlotPluginChange = async (slotId: number, pluginId: string) => {
    if (assigningSlotId !== null) {
      return;
    }

    if (!rack?.by_id[slotId]) {
      setAssignErrorBySlotId((current) => ({
        ...current,
        [slotId]: "This slot is no longer available.",
      }));
      return;
    }

    if (!pluginOptions.some((option) => option.value === pluginId)) {
      setAssignErrorBySlotId((current) => ({
        ...current,
        [slotId]: "Selected plugin is no longer available.",
      }));
      return;
    }

    setAssignErrorBySlotId((current) => {
      const next = { ...current };
      delete next[slotId];
      return next;
    });
    setAssigningSlotId(slotId);

    try {
      await actions.score.playback.slot.assignPlugin(slotId, pluginId);
    } catch (error) {
      setAssignErrorBySlotId((current) => ({
        ...current,
        [slotId]:
          error instanceof Error ? error.message : "Failed to assign plugin.",
      }));
    } finally {
      setAssigningSlotId(null);
    }
  };

  return (
    <div
      data-rack-panel-root
      tabIndex={-1}
      className="flex h-full flex-col overflow-hidden bg-neutral-950 text-white"
    >
      {createError ? (
        <div
          role="alert"
          className="border-b border-amber-800/70 bg-amber-500/10 px-3 py-2 text-xs text-amber-100"
        >
          Slot creation failed: {createError}
        </div>
      ) : null}

      <div className="flex h-full w-full overflow-auto gap-0.5">
        {slotIds.length === 0 && !focusedSlotMissing ? (
          <div className="flex h-full w-full items-center justify-center bg-neutral-900 px-6 text-center">
            <div>
              <div className="text-sm text-neutral-200">
                No score slots yet.
              </div>
              <div className="mt-2 text-sm text-neutral-400">
                Use the add button to create an empty slot, then choose an
                available plugin on that slot.
              </div>
            </div>
          </div>
        ) : null}

        {slotIds.map((slotId) => {
          const slot = rack?.by_id[slotId] ?? null;
          const slotLabel = getPlaybackRackSlotLabel(
            slotId,
            rack,
            pluginInventory.entries,
            slot?.target?.plugin_id ?? null,
          );
          const isFocused = focusedSlotId === slotId;
          const isAssigning = assigningSlotId === slotId;
          const selectedPluginId = slot?.target?.plugin_id ?? null;
          const hasUsableAssignedValue =
            selectedPluginId !== null &&
            pluginOptions.some((option) => option.value === selectedPluginId);

          return (
            <div
              key={slotId}
              data-rack-slot-id={slotId}
              tabIndex={-1}
              className={classNames(
                "flex h-full min-w-64 w-64 flex-col bg-neutral-900",
                {
                  "ring-1 ring-sky-500/70": isFocused,
                },
              )}
            >
              <div className="p-3 pb-2">
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0 flex-1">
                    <div className="text-[11px] uppercase tracking-[0.18em] text-neutral-500">
                      {slot?.target ? "Assigned slot" : "Empty slot"}
                    </div>
                    <div className="mt-1 truncate text-sm font-semibold text-neutral-100">
                      <TokenizedText>{slotLabel}</TokenizedText>
                    </div>
                  </div>
                  {isFocused && presentation ? (
                    <PlaybackReadinessBadge presentation={presentation} />
                  ) : null}
                </div>
              </div>

              <div className="flex flex-1 flex-col gap-3 px-3 pb-3">
                <Select
                  value={hasUsableAssignedValue ? selectedPluginId : null}
                  options={pluginOptions}
                  onChange={(pluginId) =>
                    handleSlotPluginChange(slotId, pluginId)
                  }
                  variant="filled"
                  label="Plugin"
                  placeholder={
                    pluginOptions.length > 0
                      ? "Select available plugin"
                      : "No available plugins"
                  }
                  disabled={pluginOptions.length === 0 || isAssigning}
                />

                <div className="text-xs text-neutral-400">
                  {slot?.target
                    ? "This slot targets a plugin in the current score rack and can be updated from here."
                    : "This slot is ready for a plugin assignment from the available inventory."}
                </div>

                {assignErrorBySlotId[slotId] ? (
                  <div
                    role="alert"
                    className="border border-amber-800/70 bg-amber-500/10 px-3 py-2 text-xs text-amber-100"
                  >
                    Plugin assignment failed: {assignErrorBySlotId[slotId]}
                  </div>
                ) : null}

                {isFocused && instrumentLabel && presentation ? (
                  <div className="mt-auto rounded border border-sky-900/70 bg-neutral-950 px-3 py-3">
                    <div className="text-[11px] uppercase tracking-[0.18em] text-sky-100/80">
                      Focused instrument route
                    </div>
                    <div className="mt-1 truncate text-sm text-neutral-100">
                      <TokenizedText>{instrumentLabel}</TokenizedText>
                    </div>
                    <div className="mt-2 text-xs text-neutral-300">
                      {presentation.target_summary}
                    </div>
                    {presentation.address_summary ? (
                      <div className="mt-1 text-xs text-neutral-400">
                        {presentation.address_summary}
                      </div>
                    ) : null}
                  </div>
                ) : null}
              </div>
            </div>
          );
        })}

        {focusedSlotMissing && instrumentLabel && presentation ? (
          <div className="flex h-full min-w-64 w-64 flex-col bg-neutral-900 px-4 py-4">
            <div className="text-[11px] uppercase tracking-[0.18em] text-amber-200">
              Missing routed slot
            </div>
            <div className="mt-2 text-sm text-neutral-100">
              <TokenizedText>{instrumentLabel}</TokenizedText>
            </div>
            <div className="mt-2 text-sm text-neutral-300">
              {presentation.explanation}
            </div>
          </div>
        ) : null}

        {slotIds.length > 0 || focusedSlotMissing ? (
          <div className="bg-neutral-900 flex-1 min-w-8" />
        ) : null}
      </div>
    </div>
  );
};
