import { FC, useMemo, useState } from "react";
import { IconButton } from "../../../ui/buttons/icon-button";
import { mdiRefresh, mdiMagnify, mdiOpenInNew } from "@mdi/js";
import { useSamplerState } from "../../../services/state";
import { TokenizedText } from "../../../ui/text";
import { actions } from "../../../services/actions";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Input } from "../../../ui/input";
import { Switch } from "../../../ui/switch";
import { PluginInfo } from "../../../services/types";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

interface PluginTileProps {
  plugin: PluginInfo;
  isLoaded: boolean;
}

const PluginTile: FC<PluginTileProps> = ({ plugin, isLoaded }) => {
  // Convert file path to asset URL if cover image exists
  const backgroundImage = plugin.cover_image
    ? `url(${convertFileSrc(plugin.cover_image)})`
    : undefined;

  const handleOpenSettings = async () => {
    const sanitizedId = plugin.id.replace(/\./g, "-");
    const windowLabel = `plugin-settings-${sanitizedId}`;

    const existingWindow = await WebviewWindow.getByLabel(windowLabel);
    if (existingWindow) {
      await existingWindow.setFocus();
      return;
    }

    new WebviewWindow(windowLabel, {
      url: `/plugin-settings.html?id=${encodeURIComponent(plugin.id)}`,
      title: "",
      width: 550,
      height: 650,
      resizable: false,
      titleBarStyle: "overlay",
      theme: "dark",
      center: true,
      focus: true,
    });
  };

  return (
    <div
      className="flex flex-col bg-neutral-900 h-full relative bg-no-repeat bg-cover bg-center"
      style={{ aspectRatio: "24 / 40", backgroundImage }}
    >
      <div className="absolute top-0 left-0 right-0 p-2 pl-4 flex items-center gap-1">
        <div className="mt-1 bg-neutral-950/50 text-neutral-300 text-xs px-2 py-1 text-center min-w-0 truncate">
          v{plugin.version}
        </div>

        <div className="flex-1" />

        <IconButton
          path={mdiOpenInNew}
          variant="filled"
          className="bg-transparent"
          isDisabled={!isLoaded}
          size={0.75}
          onClick={handleOpenSettings}
        />
      </div>

      <div className="text-center absolute bottom-0 left-0 right-0 p-4 pt-20 bg-linear-0 from-black/75 to-black/0">
        <div className="text-white text-sm truncate font-semibold">
          <TokenizedText>{plugin.name}</TokenizedText>
        </div>

        <div className="text-neutral-400 text-xs truncate">
          by {plugin.author}
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
      await actions.sampler.rescanPlugins();
    } finally {
      setIsRescanning(false);
    }
  };

  const handleOpenNativeHostPoc = async () => {
    try {
      await actions.sampler.openPluginEditorPoc();
    } catch (error) {
      console.error("Failed to open macOS native host POC:", error);
    }
  };

  return (
    <div className="flex flex-1 gap-0.5">
      <IconButton
        path={mdiRefresh}
        variant="interactive"
        onClick={handleRescan}
        isDisabled={isRescanning}
        title={isRescanning ? "Rescanning..." : "Refresh plugin list"}
      />

      <div className="flex bg-neutral-900 items-center text-white text-xs px-2 gap-2">
        <span>Active only</span>
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

      <button
        type="button"
        onClick={handleOpenNativeHostPoc}
        className="bg-neutral-900 px-3 text-[11px] font-semibold uppercase tracking-[0.2em] text-sky-400 transition-colors hover:bg-neutral-800"
        title="Open the macOS native plugin host proof of concept"
        aria-label="Open the macOS native plugin host proof of concept"
      >
        Native Host POC
      </button>

      <div className="bg-neutral-900 flex-1" />
    </div>
  );
};

type PluginProps = {
  searchTerm: string;
  showActiveOnly: boolean;
};

export const Plugins: FC<PluginProps> = ({ searchTerm, showActiveOnly }) => {
  const sampler = useSamplerState();

  const filteredPlugins = useMemo(() => {
    return sampler.plugins
      .filter((plugin) => {
        const isLoaded = sampler.loaded_plugin_ids.includes(plugin.id);

        if (showActiveOnly && !isLoaded) {
          return false;
        }

        return plugin.name.toLowerCase().includes(searchTerm.toLowerCase());
      })
      .sort((a, b) => {
        return a.id.localeCompare(b.id);
      });
  }, [sampler.loaded_plugin_ids, sampler.plugins, searchTerm, showActiveOnly]);

  if (sampler.plugins.length === 0) {
    return (
      <div className="w-full h-full flex items-center justify-center bg-neutral-900">
        <div className="text-center text-neutral-400">
          <div className="text-md mb-2">No instruments found</div>
        </div>
      </div>
    );
  }

  if (filteredPlugins.length === 0) {
    return (
      <div className="w-full h-full flex items-center justify-center bg-neutral-900">
        <div className="text-center text-neutral-700">
          <div className="text-md mb-2">No instruments match your filters</div>
        </div>
      </div>
    );
  }

  return (
    <div className="w-full h-full overflow-x-auto overflow-y-hidden">
      <div className="flex h-full gap-0.5">
        {filteredPlugins.map((plugin) => {
          const isLoaded = sampler.loaded_plugin_ids.includes(plugin.id);

          return (
            <PluginTile key={plugin.id} plugin={plugin} isLoaded={isLoaded} />
          );
        })}

        <div className="bg-neutral-900 flex-1 h-full" />
      </div>
    </div>
  );
};
