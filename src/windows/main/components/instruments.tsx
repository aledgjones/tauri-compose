import { FC, useState } from "react";
import { IconButton } from "../../../ui/icon-button";
import { mdiRefresh, mdiPower, mdiMagnify } from "@mdi/js";
import { useSamplerState } from "../../../services/state";
import { TokenizedText } from "../../../ui/text";
import { actions } from "../../../services/actions";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Input } from "../../../ui/input";

type Plugin = {
  id: string;
  name: string;
  author: string;
  version: string;
  format: string;
  cover_image?: string;
};

const InstrumentTile: FC<{ plugin: Plugin }> = ({ plugin }) => {
  const isLoaded = false; // TODO: implement plugin loading state

  // Convert file path to asset URL if cover image exists
  const backgroundImage = plugin.cover_image
    ? `url(${convertFileSrc(plugin.cover_image)})`
    : undefined;

  return (
    <div
      key={plugin.id}
      className="flex flex-col bg-neutral-900 h-full relative"
      style={{
        aspectRatio: "24 / 40",
        backgroundImage,
        backgroundSize: "cover",
        backgroundPosition: "center",
        backgroundRepeat: "no-repeat",
      }}
    >
      <div className="absolute top-0 left-0 right-0 p-2 pl-4 flex items-center gap-1">
        <div className="mt-1 bg-neutral-950/50 text-neutral-300 text-xs px-2 py-1 text-center min-w-0 truncate">
          v{plugin.version}
        </div>

        <div className="flex-1" />

        <IconButton
          path={mdiPower}
          variant="filled-dark"
          isSelected={isLoaded}
          onClick={() => {
            // TODO: implement load/disable logic
          }}
          title={isLoaded ? "Disable plugin" : "Load plugin"}
        />
      </div>

      <div className="text-center absolute bottom-0 left-0 right-0 p-4 bg-linear-0 from-black/50 to-black/0">
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

type InstrumentsToolbarProps = {
  searchTerm: string;
  onSearchTermChange: (term: string) => void;
};

export const InstrumentsToolbar: FC<InstrumentsToolbarProps> = ({
  searchTerm,
  onSearchTermChange,
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

  return (
    <div className="flex flex-1 gap-0.5">
      <IconButton
        path={mdiRefresh}
        variant="interactive"
        onClick={handleRescan}
        isDisabled={isRescanning}
        title={isRescanning ? "Rescanning..." : "Refresh plugin list"}
      />
      <Input
        type="text"
        variant="dark"
        leadingIcon={mdiMagnify}
        value={searchTerm}
        onChange={onSearchTermChange}
        placeholder="Search instruments..."
        className="w-80"
      />

      <div className="bg-neutral-900 flex-1" />
    </div>
  );
};

type InstrumentsProps = {
  searchTerm: string;
};

export const Instruments: FC<InstrumentsProps> = ({ searchTerm }) => {
  const sampler = useSamplerState();

  const filteredPlugins = sampler.plugins.filter((plugin) => {
    const matchesSearch = plugin.name
      .toLowerCase()
      .includes(searchTerm.toLowerCase());
    return matchesSearch;
  });

  if (sampler.plugins.length === 0) {
    return (
      <div className="w-full h-full flex items-center justify-center bg-neutral-900">
        <div className="text-center text-neutral-400">
          <div className="text-md mb-2">No instruments found</div>
        </div>
      </div>
    );
  }

  if (filteredPlugins.length === 0 && searchTerm.length > 0) {
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
          return <InstrumentTile key={plugin.id} plugin={plugin} />;
        })}

        <div className="bg-neutral-900 flex-1 h-full" />
      </div>
    </div>
  );
};
