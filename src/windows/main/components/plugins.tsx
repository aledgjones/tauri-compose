import { FC, useMemo, useState } from "react";
import { IconButton } from "../../../ui/buttons/icon-button";
import { mdiRefresh, mdiMagnify } from "@mdi/js";
import { usePluginInventoryState } from "../../../services/state";
import { TokenizedText } from "../../../ui/text";
import { actions } from "../../../services/actions";
import { Input } from "../../../ui/input";
import { Switch } from "../../../ui/switch";
import { PluginInventoryEntry } from "../../../services/types";

interface PluginTileProps {
  plugin: PluginInventoryEntry;
}

const PluginTile: FC<PluginTileProps> = ({ plugin }) => {
  const isAvailable = plugin.availability === "available";
  const statusLabel = isAvailable ? "Available" : "Problematic";
  const detail = isAvailable
    ? plugin.binary_path
    : (plugin.validation_failure?.message ?? "Validation failed");

  return (
    <div className="flex min-w-84 max-w-84 flex-col justify-between bg-neutral-900 p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="text-white text-sm truncate font-semibold">
            <TokenizedText>{plugin.name}</TokenizedText>
          </div>
          <div className="mt-1 text-[11px] text-neutral-500 truncate">
            {plugin.bundle_path}
          </div>
        </div>

        <div
          className="shrink-0 px-2 py-1 text-[11px] uppercase tracking-[0.18em] text-neutral-200"
          aria-label={`Status: ${statusLabel}`}
        >
          {statusLabel}
        </div>
      </div>

      <div className="mt-4 space-y-2 text-xs text-neutral-300">
        <div>
          <div className="text-neutral-500">Root</div>
          <div className="truncate">{plugin.source_root}</div>
        </div>

        <div>
          <div className="text-neutral-500">
            {isAvailable ? "Binary" : "Issue"}
          </div>
          <div className="line-clamp-3">{detail}</div>
        </div>
      </div>
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
        title={isRescanning ? "Refreshing..." : "Refresh plugin inventory"}
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
        className="w-80"
      />

      <div className="bg-neutral-900 flex-1" />
    </div>
  );
};

type PluginProps = {
  searchTerm: string;
  showActiveOnly: boolean;
};

export const Plugins: FC<PluginProps> = ({ searchTerm, showActiveOnly }) => {
  const pluginInventory = usePluginInventoryState();

  const filteredPlugins = useMemo(() => {
    return pluginInventory.entries
      .filter((plugin) => {
        const isAvailable = plugin.availability === "available";

        if (showActiveOnly && !isAvailable) {
          return false;
        }

        return plugin.name.toLowerCase().includes(searchTerm.toLowerCase());
      })
      .sort((a, b) => {
        return a.id.localeCompare(b.id);
      });
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
      <div className="grid min-h-full grid-cols-[repeat(auto-fill,minmax(21rem,1fr))] gap-0.5">
        {filteredPlugins.map((plugin) => {
          return <PluginTile key={plugin.id} plugin={plugin} />;
        })}
      </div>
    </div>
  );
};
