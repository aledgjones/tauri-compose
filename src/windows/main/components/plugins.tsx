import { FC, useMemo, useState } from "react";
import { IconButton } from "../../../ui/buttons/icon-button";
import {
  mdiAlertCircleOutline,
  mdiCheckCircleOutline,
  mdiMagnify,
  mdiOpenInNew,
  mdiRefresh,
} from "@mdi/js";
import { usePluginInventoryState } from "../../../services/state";
import { TokenizedText } from "../../../ui/text";
import { actions } from "../../../services/actions";
import { Input } from "../../../ui/input";
import { Switch } from "../../../ui/switch";
import {
  PluginEditorHostValidation,
  PluginInventoryEntry,
} from "../../../services/types";
import { Icon } from "../../../ui/icon";
import classNames from "classnames";

const pluginBadgeTones = [
  "border-sky-700/70 bg-sky-500/10 text-sky-100",
  "border-fuchsia-700/70 bg-fuchsia-500/10 text-fuchsia-100",
  "border-cyan-700/70 bg-cyan-500/10 text-cyan-100",
  "border-violet-700/70 bg-violet-500/10 text-violet-100",
  "border-emerald-700/70 bg-emerald-500/10 text-emerald-100",
  "border-rose-700/70 bg-rose-500/10 text-rose-100",
];

const getPluginBadgeLabel = (name: string) => {
  const words = name.match(/[A-Za-z0-9]+/g) ?? [];

  if (words.length === 0) {
    return "?";
  }

  if (words.length === 1) {
    return words[0].slice(0, 2).toUpperCase() || "?";
  }

  return `${words[0]?.[0] ?? ""}${words[1]?.[0] ?? ""}`.toUpperCase() || "?";
};

const getPluginBadgeTone = (name: string) => {
  const hash = Array.from(name).reduce(
    (sum, char) => sum + char.charCodeAt(0),
    0,
  );

  return pluginBadgeTones[hash % pluginBadgeTones.length];
};

interface PluginTileProps {
  plugin: PluginInventoryEntry;
  showMaintainerEditorTrigger: boolean;
}

const getPluginPathSubtitle = (plugin: PluginInventoryEntry) => {
  return plugin.binary_path ?? plugin.bundle_path;
};

const isPluginInventoryAvailable = (plugin: PluginInventoryEntry) => {
  return plugin.availability === "available";
};

const isEditorHostBlocked = (validation: PluginEditorHostValidation | null) => {
  return validation?.status === "blocked";
};

const formatEditorHostPhase = (phase: PluginEditorHostValidation["phase"]) => {
  return phase.replace(/_/g, " ");
};

const getPluginSortPriority = (plugin: PluginInventoryEntry) => {
  return isPluginInventoryAvailable(plugin) ? 0 : 1;
};

const comparePlugins = (
  left: PluginInventoryEntry,
  right: PluginInventoryEntry,
) => {
  const priorityDifference =
    getPluginSortPriority(left) - getPluginSortPriority(right);

  if (priorityDifference !== 0) {
    return priorityDifference;
  }

  const nameComparison = left.name.localeCompare(right.name, undefined, {
    sensitivity: "base",
  });

  if (nameComparison !== 0) {
    return nameComparison;
  }

  return left.id.localeCompare(right.id);
};

const PluginTile: FC<PluginTileProps> = ({
  plugin,
  showMaintainerEditorTrigger,
}) => {
  const [isOpeningEditor, setIsOpeningEditor] = useState(false);
  const [openError, setOpenError] = useState<string | null>(null);
  const isAvailable = plugin.availability === "available";
  const editorHostValidation = plugin.editor_host_validation;
  const editorHostBlocked = isEditorHostBlocked(editorHostValidation);
  const canOpenNativeEditor = isAvailable && !editorHostBlocked;
  const statusLabel = isAvailable ? "Available" : "Unavailable";
  const statusIcon =
    statusLabel === "Available" ? mdiCheckCircleOutline : mdiAlertCircleOutline;
  const statusChipClassName = classNames(
    "self-center shrink-0 flex items-center gap-1.5 rounded-full border pl-1.5 pr-2.5 py-1 text-[11px] uppercase tracking-[0.18em]",
    {
      "border-emerald-700/70 bg-emerald-500/10 text-emerald-100":
        statusLabel === "Available",
      "border-amber-700/70 bg-amber-500/10 text-amber-100":
        statusLabel !== "Available",
    },
  );
  const statusIconClassName =
    statusLabel === "Available" ? "text-emerald-200" : "text-amber-200";
  const subtitle = getPluginPathSubtitle(plugin);
  const badgeLabel = getPluginBadgeLabel(plugin.name);
  const badgeToneClassName = getPluginBadgeTone(plugin.name);
  const editorHostSummary = editorHostValidation?.message ?? null;
  const editorHostCapabilitySummary =
    editorHostValidation?.capability_evidence?.message ?? null;
  const editorHostStatusLabel = editorHostValidation
    ? editorHostValidation.status === "attached"
      ? "Ready"
      : editorHostValidation.status === "blocked"
        ? "Blocked"
        : `Failed at ${formatEditorHostPhase(editorHostValidation.phase)}`
    : "Not run";

  const handleOpenNativeEditor = async () => {
    if (!canOpenNativeEditor || isOpeningEditor) {
      return;
    }

    setIsOpeningEditor(true);
    setOpenError(null);

    try {
      await actions.pluginInventory.openEditorPoc(plugin.id);
    } catch (error) {
      setOpenError(
        error instanceof Error
          ? error.message
          : "Failed to open the native editor host.",
      );
    } finally {
      setIsOpeningEditor(false);
    }
  };

  return (
    <div className="flex flex-col gap-3 bg-neutral-900 px-4 py-3">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0 flex flex-1 items-start gap-3">
          <div
            aria-hidden="true"
            data-testid={`plugin-badge-${plugin.id}`}
            className={classNames(
              "flex h-10 w-10 shrink-0 items-center justify-center rounded-sm border text-xs font-semibold uppercase tracking-[0.16em]",
              badgeToneClassName,
            )}
          >
            {badgeLabel}
          </div>

          <div className="min-w-0 flex-1">
            <div className="text-white text-sm truncate font-semibold">
              <TokenizedText>{plugin.name}</TokenizedText>
            </div>
            <div className="mt-0.5 text-[11px] truncate text-neutral-500">
              {subtitle}
            </div>
          </div>
        </div>

        <div
          className={statusChipClassName}
          aria-label={`Status: ${statusLabel}`}
        >
          <Icon className={statusIconClassName} path={statusIcon} size={0.7} />
          <span>{statusLabel}</span>
        </div>
      </div>

      {showMaintainerEditorTrigger && isAvailable ? (
        <div className="flex items-center justify-between gap-3 border border-sky-900/60 bg-neutral-950/70 px-3 py-2">
          <div className="min-w-0 flex-1">
            <div className="text-[11px] uppercase tracking-[0.18em] text-sky-200/80">
              Maintainer preview
            </div>
            <div className="truncate text-xs text-neutral-400">
              {editorHostBlocked
                ? "Blocked for the bounded native editor host by recorded compatibility evidence."
                : "Open the bounded native editor host for this inventory target."}
            </div>
            <div className="mt-1 text-[11px] text-neutral-500">
              Native editor status: {editorHostStatusLabel}
            </div>
          </div>

          {canOpenNativeEditor ? (
            <button
              type="button"
              onClick={handleOpenNativeEditor}
              disabled={isOpeningEditor}
              className={classNames(
                "flex shrink-0 items-center gap-2 border px-3 py-2 text-xs uppercase tracking-[0.14em]",
                {
                  "border-sky-700/70 bg-sky-500/10 text-sky-100 hover:bg-sky-500/20":
                    !isOpeningEditor,
                  "cursor-default border-sky-900/70 bg-sky-500/5 text-sky-100/50":
                    isOpeningEditor,
                },
              )}
              aria-label={`Open native editor host for ${plugin.name}`}
            >
              <Icon path={mdiOpenInNew} size={0.7} />
              <span>
                {isOpeningEditor ? "Opening..." : "Open native editor"}
              </span>
            </button>
          ) : null}
        </div>
      ) : null}

      {showMaintainerEditorTrigger && isAvailable && editorHostSummary ? (
        <div className="border border-neutral-800 bg-neutral-950/70 px-3 py-2 text-xs text-neutral-300">
          {editorHostSummary}
          {editorHostCapabilitySummary ? (
            <div className="mt-1 text-[11px] text-neutral-500">
              Capability evidence: {editorHostCapabilitySummary}
            </div>
          ) : null}
        </div>
      ) : null}

      {openError ? (
        <div
          role="alert"
          className="border border-amber-800/70 bg-amber-500/10 px-3 py-2 text-xs text-amber-100"
        >
          Native editor host failed: {openError}
        </div>
      ) : null}
    </div>
  );
};

type PluginsToolbarProps = {
  searchTerm: string;
  onSearchTermChange: (term: string) => void;
  showActiveOnly: boolean;
  onShowActiveOnlyChange: (value: boolean) => void;
};

export const PluginsToolbar: FC<PluginsToolbarProps> = ({
  searchTerm,
  onSearchTermChange,
  showActiveOnly,
  onShowActiveOnlyChange,
}) => {
  const [isRescanning, setIsRescanning] = useState(false);

  const handleRescan = async () => {
    if (isRescanning) {
      return;
    }

    setIsRescanning(true);

    try {
      await actions.pluginInventory.discover();
    } finally {
      setIsRescanning(false);
    }
  };

  return (
    <div className="flex flex-1 gap-0.5">
      <IconButton
        path={mdiRefresh}
        variant="interactive"
        onClick={handleRescan}
        isDisabled={isRescanning}
        title="Refresh plugin inventory"
      />

      <div className="flex bg-neutral-900 items-center text-white text-xs px-2 gap-2">
        <span>Available only</span>
        <Switch
          small
          value={showActiveOnly}
          onChange={onShowActiveOnlyChange}
        />
      </div>

      <Input
        type="text"
        variant="dark"
        leadingIcon={mdiMagnify}
        value={searchTerm}
        onChange={onSearchTermChange}
        placeholder="Search plugins..."
        className="flex-1"
      />
    </div>
  );
};

type PluginProps = {
  searchTerm: string;
  showActiveOnly: boolean;
  showMaintainerEditorTrigger?: boolean;
};

export const Plugins: FC<PluginProps> = ({
  searchTerm,
  showActiveOnly,
  showMaintainerEditorTrigger = false,
}) => {
  const pluginInventory = usePluginInventoryState();

  const filteredPlugins = useMemo(() => {
    return pluginInventory.entries
      .filter((plugin) => {
        if (showActiveOnly && !isPluginInventoryAvailable(plugin)) {
          return false;
        }

        return plugin.name.toLowerCase().includes(searchTerm.toLowerCase());
      })
      .sort(comparePlugins);
  }, [pluginInventory.entries, searchTerm, showActiveOnly]);

  if (pluginInventory.entries.length === 0) {
    return (
      <div className="w-full h-full flex items-center justify-center bg-neutral-900">
        <div className="text-center text-neutral-400">
          <div className="text-md mb-2">No plugins discovered yet</div>
        </div>
      </div>
    );
  }

  if (filteredPlugins.length === 0) {
    return (
      <div className="w-full h-full flex items-center justify-center bg-neutral-900">
        <div className="text-center text-neutral-700">
          <div className="text-md mb-2">No plugins match your filters</div>
        </div>
      </div>
    );
  }

  return (
    <div className="w-full h-full overflow-auto bg-neutral-950">
      <div className="flex min-h-full flex-col gap-0.5">
        {filteredPlugins.map((plugin) => {
          return (
            <PluginTile
              key={plugin.id}
              plugin={plugin}
              showMaintainerEditorTrigger={showMaintainerEditorTrigger}
            />
          );
        })}
        <div
          aria-hidden="true"
          data-testid="plugins-list-filler"
          className="flex-1 bg-neutral-900"
        />
      </div>
    </div>
  );
};
