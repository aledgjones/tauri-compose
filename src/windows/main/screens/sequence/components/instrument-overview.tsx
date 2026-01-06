import { FC, useMemo } from "react";
import { useAppState } from "../../../../../services/state";
import { generateRainbowColors } from "../../../../../services/utils";
import { InstrumentOverviewRow } from "./instrument-overview-row";
import { TickBar } from "../../../components/tick-bar";
import { useTicks } from "../../../hooks/use-ticks";
import { KEYBOARD_WIDTH } from "../../../config";
import { MiniaturePianoRoll } from "./miniature-piano-roll";
import { UniqueId } from "../../../../../services/types";
import { DragScroll } from "../../../../../components/drag-scroll";
import { PlaybackLine } from "../../../components/playback-line";

export const InstrumentOverview: FC = () => {
  const state = useAppState();

  // Get the selected flow from UI state, or fall back to the first flow
  const selectedFlowId = state.ui.sequencer.flow;
  const flowId = selectedFlowId || state.score.flows.order[0];
  const flow = flowId ? state.score.flows.by_id[flowId] : null;

  // Get all instruments in the score with their player information
  const instruments = useMemo(() => {
    return state.score.players.order.flatMap((playerId: UniqueId) => {
      const player = state.score.players.by_id[playerId];
      return player.instrument_ids.map((instrumentId: UniqueId) => {
        const instrument = state.score.instruments.by_id[instrumentId];
        return {
          ...instrument,
          playerId, // Add player ID for flow checking
        };
      });
    });
  }, [state.score.players, state.score.instruments]);

  // Check if an instrument is in the selected flow
  const isInstrumentInSelectedFlow = useMemo(() => {
    return (instrument: any) => {
      if (!flow) return true; // Fallback in case no flow exists at all
      return flow.player_ids.includes(instrument.playerId);
    };
  }, [flow]);

  // Generate colors for instruments
  const instrumentColors = useMemo(() => {
    return generateRainbowColors(instruments.length);
  }, [instruments.length]);

  // Get ticks for the flow (call hooks before any early returns)
  const master = flow?.master_id
    ? state.score.tracks.by_id[flow.master_id]
    : null;
  const ticks = useTicks(flow, master);

  if (!flow) {
    return (
      <div className="flex items-center justify-center w-full h-full bg-neutral-900 text-neutral-700">
        You don't have any flows yet...
      </div>
    );
  }

  return (
    <DragScroll className="w-full h-full bg-neutral-950 relative">
      {/* Left column - Instrument labels (sticky, scrollable vertically) */}
      <div
        className="ignore-x sticky left-0 z-20 flex flex-col bg-neutral-950"
        style={{ width: KEYBOARD_WIDTH * 1.5 }}
      >
        {/* Header space for tick bar */}
        <div className="sticky top-0 z-30 h-7 bg-neutral-900 w-full mb-0.5" />

        {/* Scrollable instrument list */}
        <div className="flex-1 overflow-y-auto flex flex-col gap-0.5">
          {instruments.map((instrument: any, index: number) => (
            <InstrumentOverviewRow
              key={instrument.id}
              instrument={instrument}
              color={instrumentColors[index]}
              isDisabled={!isInstrumentInSelectedFlow(instrument)}
            />
          ))}
        </div>
      </div>

      {/* Right column - Miniature piano rolls (scrollable both directions) */}
      <div
        style={{ paddingLeft: KEYBOARD_WIDTH * 1.5 }}
        className="absolute top-0 z-10 min-w-full"
      >
        {/* Tick bar - sticky at top */}
        <TickBar
          ticks={ticks}
          className="sticky top-0 z-30 bg-neutral-900 mb-0.5"
        />

        {/* Scrollable piano roll overviews */}
        <div className="flex flex-col overflow-y-auto gap-0.5 relative">
          {instruments.map((instrument: any, index: number) => (
            <MiniaturePianoRoll
              key={instrument.id}
              instrument={instrument}
              flow={flow}
              ticks={ticks}
              color={instrumentColors[index]}
              isDisabled={!isInstrumentInSelectedFlow(instrument)}
            />
          ))}
          <PlaybackLine
            ticks={ticks}
            height={instruments.length * 64 + (instruments.length - 1) * 2}
          />
        </div>
      </div>
    </DragScroll>
  );
};
