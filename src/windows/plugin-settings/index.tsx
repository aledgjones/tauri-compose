import { FC, useMemo, useState } from "react";
import { useSamplerState } from "../../services/state";
import { convertFileSrc } from "@tauri-apps/api/core";
import { actions } from "../../services/actions";
import { ArticulationInfo } from "../../services/types";
import classNames from "classnames";

const isNoteInRanges = (note: number, ranges: Array<[number, number]>) => {
  return ranges.some(([min, max]) => note >= min && note <= max);
};

type MiniKeyboardProps = {
  range: Array<[number, number]>;
  onNoteClick: (note: number) => void;
};

const MiniKeyboard: FC<MiniKeyboardProps> = ({ range, onNoteClick }) => {
  const startNote = 17;
  const endNote = 112;

  const isBlackKey = (note: number) => {
    const pitchClass = note % 12;
    return [1, 3, 6, 8, 10].includes(pitchClass);
  };

  const whiteKeys: number[] = [];
  const blackKeys: number[] = [];

  for (let note = startNote; note <= endNote; note++) {
    if (isBlackKey(note)) {
      blackKeys.push(note);
    } else {
      whiteKeys.push(note);
    }
  }

  const whiteKeyWidth = 100 / whiteKeys.length;
  const blackKeyWidth = whiteKeyWidth * 0.6;

  return (
    <div className="relative h-16 flex">
      {/* White keys */}
      {whiteKeys.map((note) => {
        const inRange = isNoteInRanges(note, range);

        return (
          <button
            key={note}
            disabled={!inRange}
            onPointerDown={() => onNoteClick(note)}
            className={classNames(
              "border transition-colors border-y-0 rounded-b-xs",
              {
                "bg-white hover:bg-neutral-200 border-neutral-950": inRange,
                "bg-neutral-600 cursor-not-allowed border-neutral-800":
                  !inRange,
              },
            )}
            style={{ width: `${whiteKeyWidth}%` }}
          />
        );
      })}

      {/* Black keys */}
      <div className="absolute inset-0 pointer-events-none">
        {blackKeys.map((note) => {
          const inRange = isNoteInRanges(note, range);
          const whiteKeyIndex = whiteKeys.findIndex((wk) => wk > note) - 1;
          const leftPercent = ((whiteKeyIndex + 0.7) / whiteKeys.length) * 100;

          return (
            <button
              key={note}
              disabled={!inRange}
              onPointerDown={() => onNoteClick(note)}
              className={classNames(
                "absolute pointer-events-auto h-[60%] border-2 border-t-0 rounded-b-xs",
                {
                  "bg-neutral-950 hover:bg-neutral-700 border-neutral-950":
                    inRange,
                  "bg-neutral-800 cursor-not-allowed border-neutral-800":
                    !inRange,
                },
              )}
              style={{ left: `${leftPercent}%`, width: `${blackKeyWidth}%` }}
            />
          );
        })}
      </div>
    </div>
  );
};

type InstrumentTileProps = {
  articulation: ArticulationInfo;
  isSelected: boolean;
  onSelect: () => void;
};

const ArticulationItem: FC<InstrumentTileProps> = ({
  articulation,
  isSelected,
  onSelect,
}) => {
  return (
    <button
      onClick={onSelect}
      className={classNames(
        "border p-3 rounded cursor-pointer transition-colors min-w-23 shrink-0 h-12 text-center",
        {
          "bg-sky-500/10 border-sky-500": isSelected,
          "bg-neutral-800/20 border-neutral-800/50 hover:bg-neutral-800/50":
            !isSelected,
        },
      )}
    >
      <div className="text-white text-[11px] uppercase whitespace-nowrap">
        {articulation.name}
      </div>
    </button>
  );
};

export const Main: FC = () => {
  const samplerState = useSamplerState();
  const [selectionId, setSelectionId] = useState<string | null>(null);

  const pluginId = useMemo(() => {
    const params = new URLSearchParams(window.location.search);
    return params.get("id");
  }, []);

  const plugin = useMemo(() => {
    return samplerState.plugins.find((p) => p.id === pluginId);
  }, [samplerState.plugins, pluginId]);

  const selection = useMemo(() => {
    if (!plugin) {
      return null;
    }

    if (selectionId === null) {
      return plugin.articulations[0] || null;
    }

    return plugin.articulations.find((a) => a.id === selectionId) || null;
  }, [plugin, selectionId]);

  const coverImageUrl = plugin?.cover_image
    ? convertFileSrc(plugin.cover_image)
    : undefined;

  const handleNoteClick = async (note: number) => {
    if (!selection?.id || !pluginId) return;

    if (!isNoteInRanges(note, selection.range)) {
      return;
    }

    try {
      await actions.sampler.audition(pluginId, note, 100, selection.id);
    } catch (error) {
      console.error(`Failed to play note ${note}:`, error);
    }
  };

  if (!plugin) {
    return (
      <div className="flex items-center justify-center h-screen bg-neutral-950">
        <div className="text-neutral-500 text-sm">Plugin not found</div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-screen bg-neutral-950">
      <div
        data-tauri-drag-region
        className="shrink-0 flex-1 bg-neutral-900 bg-no-repeat bg-cover bg-center relative flex flex-col justify-end"
        style={{ backgroundImage: `url(${coverImageUrl})` }}
      >
        <div className="absolute bottom-0 left-0 right-0 w-full pointer-events-none">
          <div className="h-60 w-full bg-linear-to-b from-transparent to-neutral-950" />
          <div className="bg-neutral-950 h-8" />
        </div>

        <div className="relative z-10 mb-4 px-6">
          <div className="text-neutral-100 text-2xl font-bold">
            {plugin.name}
          </div>

          <div className="flex items-center gap-4 mt-1">
            <div className="text-neutral-400 text-sm">by {plugin.author}</div>
            <div className="w-1 h-1 bg-neutral-600 rounded-full" />
            <div className="text-neutral-400 text-sm">v{plugin.version}</div>
          </div>
        </div>

        <div className="flex gap-2 z-10 overflow-auto px-6 pb-6">
          {plugin.articulations.map((articulation) => (
            <ArticulationItem
              key={articulation.id}
              articulation={articulation}
              isSelected={selection?.id === articulation.id}
              onSelect={() => setSelectionId(articulation.id)}
            />
          ))}
        </div>
      </div>

      <div className="px-4 pb-0.5 bg-neutral-800">
        <MiniKeyboard
          range={selection?.range || []}
          onNoteClick={handleNoteClick}
        />
      </div>
    </div>
  );
};
