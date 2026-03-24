import { FC, useMemo, useState } from "react";
import {
  useAppState,
  usePluginInventoryState,
} from "../../../../../services/state";
import { PluginInventoryEntry } from "../../../../../services/types";
import { actions } from "../../../../../services/actions";
import { Select, SelectOption } from "../../../../../ui/select";

const isUsablePreparationPlugin = (plugin: PluginInventoryEntry) => {
  return plugin.availability === "available";
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
};

const EMPTY_PLUGIN_VALUE = "__empty_slot__";

export const PlaybackRackPanel: FC<PlaybackRackPanelProps> = ({
  createError = null,
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
      await actions.pluginInventory.openEditorPoc(plugin.id);
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
        ) : null}

        {slotIds.map((slotId) => {
          const slot = rack?.by_id[slotId] ?? null;
          const slotIndex = (rack?.order.indexOf(slotId) ?? -1) + 1;
          const isAssigning = assigningSlotId === slotId;
          const selectedPluginId = slot?.target?.plugin_id ?? null;
          const selectedPlugin =
            selectedPluginId !== null
              ? (pluginById.get(selectedPluginId) ?? null)
              : null;
          const isOpeningEditor = openingEditorSlotId === slotId;
          const showOpenPluginButton = canOpenPluginEditor(selectedPlugin);
          const selectedPickerValue =
            selectedPluginId !== null &&
            pluginOptions.some((option) => option.value === selectedPluginId)
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
                <Select
                  value={selectedPickerValue}
                  options={pluginOptions}
                  onChange={(pluginId) =>
                    handleSlotPluginChange(slotId, pluginId)
                  }
                  variant="filled"
                  placeholder="--"
                  disabled={
                    isAssigning ||
                    (pluginOptions.length === 1 && selectedPluginId === null)
                  }
                />

                {showOpenPluginButton && selectedPlugin ? (
                  <button
                    type="button"
                    onClick={() =>
                      handleOpenPluginEditor(slotId, selectedPlugin)
                    }
                    disabled={isOpeningEditor}
                    className="border border-sky-700/70 bg-sky-500/10 px-3 py-2 text-xs uppercase tracking-[0.14em] text-sky-100 hover:bg-sky-500/20 disabled:cursor-default disabled:border-sky-900/70 disabled:bg-sky-500/5 disabled:text-sky-100/50"
                    aria-label={`Open plugin UI for ${selectedPlugin.name}`}
                  >
                    {isOpeningEditor ? "Opening UI..." : "Open plugin UI"}
                  </button>
                ) : null}

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

        {slotIds.length > 0 ? (
          <div className="bg-neutral-900 flex-1 min-w-8" />
        ) : null}
      </div>
    </div>
  );
};
