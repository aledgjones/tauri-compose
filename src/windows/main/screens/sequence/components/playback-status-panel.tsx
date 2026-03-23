import { FC, useMemo } from "react";
import {
  usePluginInventoryState,
  useSamplerState,
} from "../../../../../services/state";
import {
  Instrument,
  ResolvedPlaybackAssignment,
} from "../../../../../services/types";
import { TokenizedText } from "../../../../../ui/text";
import { PlaybackReadinessBadge } from "./playback-readiness";
import { getPlaybackReadinessPresentation } from "./playback-readiness-presentation";

type PlaybackStatusPanelProps = {
  instrument: Instrument | null;
  assignment: ResolvedPlaybackAssignment | null;
};

export const PlaybackStatusPanel: FC<PlaybackStatusPanelProps> = ({
  instrument,
  assignment,
}) => {
  const pluginInventory = usePluginInventoryState();
  const sampler = useSamplerState();

  const presentation = useMemo(() => {
    if (!instrument) {
      return null;
    }

    return getPlaybackReadinessPresentation(
      assignment,
      instrument,
      pluginInventory.entries,
      sampler.plugins,
    );
  }, [assignment, instrument, pluginInventory.entries, sampler.plugins]);

  const handleFocusAssignment = () => {
    if (!instrument) {
      return;
    }

    const assignmentButton = document.querySelector<HTMLButtonElement>(
      `[data-assignment-select-for="${instrument.id}"] button`,
    );

    assignmentButton?.scrollIntoView({ block: "center" });
    assignmentButton?.focus();
  };

  if (!instrument || !presentation) {
    return (
      <div className="flex h-full items-center justify-center bg-neutral-950 px-6 text-center text-sm text-neutral-500">
        Select an instrument in Sequence to inspect playback readiness.
      </div>
    );
  }

  const instrumentLabel = instrument.count
    ? `${instrument.long_name} ${instrument.count}`
    : instrument.long_name;

  return (
    <div className="flex h-full flex-col bg-neutral-950 text-white">
      <div className="border-b border-neutral-800 px-4 py-3">
        <div className="text-[11px] uppercase tracking-[0.18em] text-neutral-500">
          Selected instrument
        </div>
        <div className="mt-2 flex items-start justify-between gap-3">
          <div className="min-w-0 flex-1">
            <div className="truncate text-sm font-semibold">
              <TokenizedText>{instrumentLabel}</TokenizedText>
            </div>
            <div className="mt-1 text-xs text-neutral-400">
              {presentation.summary}
            </div>
          </div>
          <PlaybackReadinessBadge presentation={presentation} />
        </div>
      </div>

      <div className="grid flex-1 gap-0.5 bg-neutral-900 p-0.5 md:grid-cols-[minmax(0,1.15fr)_minmax(0,0.85fr)]">
        <div className="bg-neutral-950 px-4 py-4">
          <div className="text-[11px] uppercase tracking-[0.18em] text-neutral-500">
            Status
          </div>
          <div className="mt-2 text-sm text-neutral-100">
            {presentation.explanation}
          </div>
        </div>

        <div className="bg-neutral-950 px-4 py-4">
          <div className="text-[11px] uppercase tracking-[0.18em] text-neutral-500">
            Context
          </div>
          {presentation.plugin_label ? (
            <div className="mt-2">
              <div className="text-xs text-neutral-500">
                {presentation.plugin_caption}
              </div>
              <div className="mt-1 truncate text-sm text-neutral-100">
                <TokenizedText>{presentation.plugin_label}</TokenizedText>
              </div>
            </div>
          ) : (
            <div className="mt-2 text-sm text-neutral-400">
              No current plugin context.
            </div>
          )}
        </div>

        {presentation.action_label ? (
          <div className="bg-neutral-950 px-4 py-4 md:col-span-2">
            <div className="text-[11px] uppercase tracking-[0.18em] text-neutral-500">
              Next step
            </div>
            <div className="mt-2 flex flex-col gap-3 text-sm text-neutral-300 md:flex-row md:items-center md:justify-between">
              <div>
                Use the existing Sequence row assignment control for the primary
                recovery path.
              </div>
              <button
                type="button"
                onClick={handleFocusAssignment}
                className="border border-sky-700/70 bg-sky-500/10 px-3 py-2 text-xs uppercase tracking-[0.14em] text-sky-100 hover:bg-sky-500/20"
                aria-label={`Focus row assignment for ${instrumentLabel}`}
              >
                {presentation.action_label}
              </button>
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
};
