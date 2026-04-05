import { FC, useMemo, useState } from "react";
import { mdiAlertCircleOutline, mdiOpenInNew } from "@mdi/js";
import {
  useAppState,
  usePluginInventoryState,
} from "../../../../../services/state";
import { PluginInventoryEntry } from "../../../../../services/types";
import { actions } from "../../../../../services/actions";
import { Icon } from "../../../../../ui/icon";
import { IconButton } from "../../../../../ui/buttons/icon-button";
import { Select, SelectOption } from "../../../../../ui/select";

const isUsablePlugin = (plugin: PluginInventoryEntry) => {
  return (
    plugin.availability === "available" &&
    plugin.editor_host_validation?.status !== "blocked"
  );
};

const canOpenPluginEditor = (plugin: PluginInventoryEntry | null) => {
  return (
    plugin !== null &&
    plugin.availability === "available" &&
    plugin.editor_host_validation?.status !== "blocked"
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
  searchTerm?: string;
};

const EMPTY_PLUGIN_VALUE = "__empty_slot__";
const UNAVAILABLE_ASSIGNED_PLUGIN_WARNING = "Assigned plugin is unavailable";

const getUnavailableSelectedPluginLabel = (
  selectedPluginId: string,
  selectedPlugin: PluginInventoryEntry | null,
) => {
  if (selectedPlugin !== null) {
    return selectedPlugin.name;
  }

  return `Missing plugin (${selectedPluginId})`;
};

const renderUnavailableAssignedPluginAdornment = (label: string) => {
  return (
    <span
      title={`${UNAVAILABLE_ASSIGNED_PLUGIN_WARNING}: ${label}`}
      className="text-amber-300"
    >
      <Icon path={mdiAlertCircleOutline} size={0.75} />
    </span>
  );
};

export const PlaybackRackPanel: FC<PlaybackRackPanelProps> = ({
  createError = null,
  searchTerm = "",
}) => {
  const state = useAppState();
  const pluginInventory = usePluginInventoryState();
  const rack = state.score.playback?.rack ?? null;
  const usablePlugins = useMemo(
    () =>
      pluginInventory.entries.filter(isUsablePlugin).sort(compareUsablePlugins),
    [pluginInventory.entries],
  );
  const pluginOptions = useMemo<SelectOption<string>[]>(
    () => [
      { value: EMPTY_PLUGIN_VALUE, label: "--" },
      ...usablePlugins.map((plugin) => ({
        value: plugin.id,
        label: plugin.name,
      })),
    ],
    [usablePlugins],
  );
  const pluginById = useMemo(
    () => new Map(pluginInventory.entries.map((plugin) => [plugin.id, plugin])),
    [pluginInventory.entries],
  );
  const [assigningSlotId, setAssigningSlotId] = useState<number | null>(null);
  const [assignErrorBySlotId, setAssignErrorBySlotId] = useState<
    Record<number, string>
  >({});
  const [openingEditorSlotId, setOpeningEditorSlotId] = useState<number | null>(
    null,
  );
  const [openErrorBySlotId, setOpenErrorBySlotId] = useState<
    Record<number, string>
  >({});
  const slotIds = useMemo(() => rack?.order ?? [], [rack]);
  const normalizedSearchTerm = searchTerm.trim().toLowerCase();

  const visibleSlotIds = useMemo(() => {
    if (normalizedSearchTerm.length === 0) {
      return slotIds;
    }

    return slotIds.filter((slotId) => {
      const selectedPluginId = rack?.by_id[slotId]?.target?.plugin_id ?? null;
      if (selectedPluginId == null) {
        return false;
      }

      const selectedPlugin = pluginById.get(selectedPluginId) ?? null;
      const searchLabel =
        selectedPlugin?.name ??
        getUnavailableSelectedPluginLabel(selectedPluginId, selectedPlugin);

      return searchLabel.toLowerCase().includes(normalizedSearchTerm);
    });
  }, [normalizedSearchTerm, pluginById, rack, slotIds]);

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
      if (pluginId === EMPTY_PLUGIN_VALUE) {
        await actions.score.playback.slot.unassignPlugin(slotId);
      } else {
        await actions.score.playback.slot.assignPlugin(slotId, pluginId);
      }
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

  const handleOpenPluginEditor = async (
    slotId: number,
    plugin: PluginInventoryEntry,
  ) => {
    if (openingEditorSlotId !== null || !canOpenPluginEditor(plugin)) {
      return;
    }

    setOpenErrorBySlotId((current) => {
      const next = { ...current };
      delete next[slotId];
      return next;
    });
    setOpeningEditorSlotId(slotId);

    try {
      await actions.pluginInventory.openEditorPoc(plugin.id, {
        slot_id: slotId,
      });
    } catch (error) {
      setOpenErrorBySlotId((current) => ({
        ...current,
        [slotId]:
          error instanceof Error
            ? error.message
            : "Failed to open the plugin UI.",
      }));
    } finally {
      setOpeningEditorSlotId(null);
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
        {slotIds.length === 0 ? (
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
        ) : visibleSlotIds.length === 0 ? (
          <div className="flex h-full w-full items-center justify-center bg-neutral-900 px-6 text-center">
            <div>
              <div className="text-sm text-neutral-200">No slots match.</div>
              <div className="mt-2 text-sm text-neutral-400">
                Try a different plugin name.
              </div>
            </div>
          </div>
        ) : null}

        {visibleSlotIds.map((slotId) => {
          const slot = rack?.by_id[slotId] ?? null;
          const slotIndex = (rack?.order.indexOf(slotId) ?? -1) + 1;
          const isAssigning = assigningSlotId === slotId;
          const selectedPluginId = slot?.target?.plugin_id ?? null;
          const selectedPlugin =
            selectedPluginId !== null
              ? (pluginById.get(selectedPluginId) ?? null)
              : null;
          const selectedPluginNeedsAttention =
            selectedPluginId !== null &&
            !pluginOptions.some((option) => option.value === selectedPluginId);
          const unavailableSelectedPluginLabel =
            selectedPluginId !== null
              ? getUnavailableSelectedPluginLabel(
                  selectedPluginId,
                  selectedPlugin,
                )
              : null;
          const unavailableSelectedPluginOption =
            selectedPluginId !== null &&
            selectedPluginNeedsAttention &&
            unavailableSelectedPluginLabel !== null
              ? {
                  value: selectedPluginId,
                  label: unavailableSelectedPluginLabel,
                  disabled: true,
                  endAdornment: renderUnavailableAssignedPluginAdornment(
                    unavailableSelectedPluginLabel,
                  ),
                }
              : null;
          const slotPluginOptions =
            unavailableSelectedPluginOption !== null
              ? [...pluginOptions, unavailableSelectedPluginOption]
              : pluginOptions;
          const isOpeningEditor = openingEditorSlotId === slotId;
          const showOpenPluginButton = canOpenPluginEditor(selectedPlugin);
          const selectedPickerValue =
            selectedPluginId !== null &&
            slotPluginOptions.some(
              (option) => option.value === selectedPluginId,
            )
              ? selectedPluginId
              : EMPTY_PLUGIN_VALUE;

          return (
            <div
              key={slotId}
              data-rack-slot-id={slotId}
              tabIndex={-1}
              className="flex h-full min-w-64 w-64 flex-col bg-neutral-900"
            >
              <div className="p-3 pb-2">
                <div className="text-[11px] uppercase tracking-[0.18em] text-neutral-500">
                  Slot {slotIndex > 0 ? slotIndex : slotId}
                </div>
              </div>

              <div className="flex flex-1 flex-col gap-3 px-3 pb-3">
                <div className="flex items-stretch gap-2">
                  <div className="min-w-0 flex-1">
                    <Select
                      value={selectedPickerValue}
                      options={slotPluginOptions}
                      endAdornment={
                        unavailableSelectedPluginOption !== null
                          ? renderUnavailableAssignedPluginAdornment(
                              unavailableSelectedPluginOption.label,
                            )
                          : null
                      }
                      onChange={(pluginId) =>
                        handleSlotPluginChange(slotId, pluginId)
                      }
                      variant="filled"
                      placeholder="--"
                      disabled={
                        isAssigning ||
                        (pluginOptions.length === 1 &&
                          selectedPluginId === null)
                      }
                    />
                  </div>

                  {showOpenPluginButton && selectedPlugin ? (
                    <IconButton
                      path={mdiOpenInNew}
                      variant="filled"
                      size={0.8}
                      className="shrink-0"
                      isDisabled={isOpeningEditor}
                      onClick={() =>
                        handleOpenPluginEditor(slotId, selectedPlugin)
                      }
                      aria-label={`Open plugin UI for ${selectedPlugin.name}`}
                    />
                  ) : null}
                </div>

                {assignErrorBySlotId[slotId] ? (
                  <div
                    role="alert"
                    className="border border-amber-800/70 bg-amber-500/10 px-3 py-2 text-xs text-amber-100"
                  >
                    Plugin assignment failed: {assignErrorBySlotId[slotId]}
                  </div>
                ) : null}

                {openErrorBySlotId[slotId] ? (
                  <div
                    role="alert"
                    className="border border-amber-800/70 bg-amber-500/10 px-3 py-2 text-xs text-amber-100"
                  >
                    Plugin UI failed: {openErrorBySlotId[slotId]}
                  </div>
                ) : null}
              </div>
            </div>
          );
        })}

        {visibleSlotIds.length > 0 ? (
          <div className="bg-neutral-900 flex-1 min-w-8" />
        ) : null}
      </div>
    </div>
  );
};
