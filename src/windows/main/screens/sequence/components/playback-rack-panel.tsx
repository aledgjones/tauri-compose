import classNames from "classnames";
import { FC, useMemo } from "react";
import {
  useAppState,
  usePluginInventoryState,
} from "../../../../../services/state";
import {
  Instrument,
  ResolvedPlaybackRoute,
} from "../../../../../services/types";
import { TokenizedText } from "../../../../../ui/text";
import { PlaybackReadinessBadge } from "./playback-readiness";
import {
  getPlaybackRackSlotLabel,
  getPlaybackReadinessPresentation,
} from "./playback-readiness-presentation";

type PlaybackRackPanelProps = {
  instrument: Instrument | null;
  route: ResolvedPlaybackRoute | null;
};

export const PlaybackRackPanel: FC<PlaybackRackPanelProps> = ({
  instrument,
  route,
}) => {
  const state = useAppState();
  const pluginInventory = usePluginInventoryState();
  const rack = state.score.playback?.rack ?? null;

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
  const slotIds = rack?.order ?? [];

  if (
    slotIds.length === 0 &&
    !(focusedSlotMissing && instrumentLabel && presentation)
  ) {
    return (
      <div
        data-rack-panel-root
        tabIndex={-1}
        className="flex h-full flex-col justify-center bg-neutral-950 px-6 text-center"
      >
        <div className="text-[11px] uppercase tracking-[0.18em] text-neutral-500">
          Current rack
        </div>
        <div className="mt-3 text-sm text-neutral-200">
          Current rack has no score slots yet.
        </div>
        <div className="mt-2 text-sm text-neutral-400">
          Slot preparation stays in the Sequence rack workflow. Plugin Manager
          remains the app-scoped inventory browser.
        </div>
      </div>
    );
  }

  return (
    <div
      data-rack-panel-root
      tabIndex={-1}
      className="flex h-full flex-col overflow-hidden bg-neutral-950 text-white"
    >
      <div className="border-b border-neutral-800 px-4 py-3">
        <div className="text-[11px] uppercase tracking-[0.18em] text-neutral-500">
          Current rack
        </div>
        <div className="mt-2 flex flex-wrap items-start justify-between gap-3">
          <div className="min-w-0 flex-1">
            <div className="text-sm font-semibold text-neutral-100">
              {slotIds.length} score-context slot
              {slotIds.length === 1 ? "" : "s"}
            </div>
            <div className="mt-1 text-xs text-neutral-400">
              Every slot in the current score rack stays visible here, including
              empty slots.
            </div>
          </div>
          {instrumentLabel && presentation ? (
            <div className="min-w-0 text-left md:text-right">
              <div className="text-[11px] uppercase tracking-[0.18em] text-neutral-500">
                Focused instrument
              </div>
              <div className="mt-1 truncate text-sm text-neutral-100">
                <TokenizedText>{instrumentLabel}</TokenizedText>
              </div>
              <div className="mt-2 flex md:justify-end">
                <PlaybackReadinessBadge presentation={presentation} />
              </div>
            </div>
          ) : null}
        </div>
      </div>

      <div className="grid flex-1 gap-0.5 overflow-auto bg-neutral-900 p-0.5 md:grid-cols-2 xl:grid-cols-3">
        {slotIds.map((slotId) => {
          const slot = rack?.by_id[slotId] ?? null;
          const slotLabel = getPlaybackRackSlotLabel(
            slotId,
            rack,
            pluginInventory.entries,
            slot?.target?.plugin_id ?? null,
          );
          const isFocused = focusedSlotId === slotId;

          return (
            <div
              key={slotId}
              data-rack-slot-id={slotId}
              tabIndex={-1}
              className={classNames(
                "flex min-h-36 flex-col justify-between bg-neutral-950 px-4 py-4",
                {
                  "ring-1 ring-sky-500/70": isFocused,
                },
              )}
            >
              <div>
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

                <div className="mt-3 text-xs text-neutral-400">
                  {slot?.target
                    ? "This slot currently targets a plugin in the score rack."
                    : "This score rack slot exists but does not currently target a plugin."}
                </div>
              </div>

              {isFocused && instrumentLabel && presentation ? (
                <div className="mt-4 rounded border border-sky-900/70 bg-sky-500/10 px-3 py-3">
                  <div className="text-[11px] uppercase tracking-[0.18em] text-sky-100/80">
                    Focused instrument route
                  </div>
                  <div className="mt-1 truncate text-sm text-sky-50">
                    <TokenizedText>{instrumentLabel}</TokenizedText>
                  </div>
                  <div className="mt-2 text-xs text-sky-100/90">
                    {presentation.target_summary}
                  </div>
                  {presentation.address_summary ? (
                    <div className="mt-1 text-xs text-sky-100/70">
                      {presentation.address_summary}
                    </div>
                  ) : null}
                </div>
              ) : null}
            </div>
          );
        })}

        {focusedSlotMissing && instrumentLabel && presentation ? (
          <div className="flex flex-col justify-between bg-neutral-950 px-4 py-4 md:col-span-2 xl:col-span-3">
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
      </div>
    </div>
  );
};
