import { FC, useMemo } from "react";
import { useAppState } from "../../../../../services/state";
import { Select, SelectOption } from "../../../../../ui/select";
import { NoteDuration, Tool, UniqueId } from "../../../../../services/types";
import { actions } from "../../../../../services/actions";
import {
  mdiBoxCutter,
  mdiCursorDefault,
  mdiEraser,
  mdiFormatHorizontalAlignCenter,
  mdiPen,
  mdiPiano,
  mdiTuneVertical,
} from "@mdi/js";
import { IconButton } from "../../../../../ui/icon-button";

const ALPHABET = ["A", "B"];
const TRACK_LABEL = ["Up-stem", "Down-stem"];
const SNAP_OPTIONS = [
  {
    label: "Quarter",
    value: NoteDuration.Quarter,
    icon: mdiFormatHorizontalAlignCenter,
  },
  {
    label: "Eighth",
    value: NoteDuration.Eighth,
    icon: mdiFormatHorizontalAlignCenter,
  },
  {
    label: "Sixteenth",
    value: NoteDuration.Sixteenth,
    icon: mdiFormatHorizontalAlignCenter,
  },
  {
    label: "Thirty-Second",
    value: NoteDuration.ThirtySecond,
    icon: mdiFormatHorizontalAlignCenter,
  },
];
const VIEW_OPTIONS = [
  { label: "Sequencer", value: 0 },
  { label: "Mixer", value: 0 },
];
const TOOL_OPTIONS = [
  { label: "Select", value: Tool.Select, icon: mdiCursorDefault },
  { label: "Write", value: Tool.Write, icon: mdiPen },
  { label: "Slice", value: Tool.Slice, icon: mdiBoxCutter },
  { label: "Erase", value: Tool.Erase, icon: mdiEraser },
];

export const BottomPanel: FC = () => {
  const state = useAppState();

  const instruments: SelectOption<UniqueId>[] = useMemo(() => {
    return state.score.players.order.flatMap((playerId) => {
      const player = state.score.players.by_id[playerId];
      return player.instrument_ids.map((instrumentId) => {
        const instrument = state.score.instruments.by_id[instrumentId];
        return {
          value: instrumentId,
          label:
            instrument.long_name +
            (instrument.count ? ` ${instrument.count}` : ""),
        };
      });
    });
  }, [state]);

  const flowId = useMemo(() => {
    return state.score.flows.order[0] || null;
  }, [state]);

  const instrumentId = useMemo(() => {
    if (instruments.length == 0) {
      return null;
    }

    const id = state.ui.sequencer.instrument;

    // check the selection still exists
    if (
      id &&
      instruments.some((option: SelectOption<UniqueId>) => option.value === id)
    ) {
      return id;
    }

    // fallback to the first instrument
    return instruments[0].value;
  }, [state]);

  const tracks = useMemo(() => {
    if (!flowId || !instrumentId) {
      return [];
    }

    const flow = state.score.flows.by_id[flowId];
    const instrument = state.score.instruments.by_id[instrumentId];

    if (!flow || !instrument) {
      return [];
    }

    const stave_ids = instrument.stave_ids;

    return stave_ids.flatMap((staveId, i) => {
      const stave = flow.staves.by_id[staveId];

      if (!stave) {
        return [];
      }

      return stave.track_ids.map((trackId, ii) => {
        const staffLabel =
          stave_ids.length > 1 ? `Staff (${ALPHABET[i]}), ` : ``;
        const trackLabel = `${TRACK_LABEL[ii % 2]} voice ${
          Math.floor(ii / 2) + 1
        }`;

        return {
          value: trackId,
          label: [staffLabel, trackLabel].join(""),
        };
      });
    });
  }, [state]);

  const trackId = useMemo(() => {
    if (tracks.length == 0) {
      return null;
    }

    const id = state.ui.sequencer.track;

    // check the selection still exists
    if (
      id &&
      tracks.some((option: SelectOption<UniqueId>) => option.value === id)
    ) {
      return id;
    }

    // fallback to the first instrument
    return tracks[0].value;
  }, [state]);

  const tool = state.ui.sequencer.tool;

  return (
    <div className="bg-neutral-900 flex flex-col">
      <div className="flex p-2 h-[52px] items-center">
        <div className="flex flex-1">
          <IconButton
            path={mdiCursorDefault}
            variant="filled"
            isSelected={tool === Tool.Select}
            onClick={() => actions.ui.sequencer.setTool(Tool.Select)}
            color={tool === Tool.Select ? "#000" : "#fff"}
          />
          <IconButton
            path={mdiPen}
            variant="filled"
            isSelected={tool === Tool.Write}
            onClick={() => actions.ui.sequencer.setTool(Tool.Write)}
            color={tool === Tool.Write ? "#000" : "#fff"}
          />
          <IconButton
            path={mdiBoxCutter}
            variant="filled"
            isSelected={tool === Tool.Slice}
            onClick={() => actions.ui.sequencer.setTool(Tool.Slice)}
            color={tool === Tool.Slice ? "#000" : "#fff"}
          />
          <IconButton
            className="mr-2"
            path={mdiEraser}
            variant="filled"
            isSelected={tool === Tool.Erase}
            onClick={() => actions.ui.sequencer.setTool(Tool.Erase)}
            color={tool === Tool.Erase ? "#000" : "#fff"}
          />
          <Select
            icon={mdiFormatHorizontalAlignCenter}
            variant="filled"
            value={state.ui.sequencer.snap}
            options={SNAP_OPTIONS}
            onChange={actions.ui.sequencer.setSnap}
          />
        </div>
        <div className="flex flex-1 justify-center">
          {instrumentId && instruments.length > 0 && (
            <Select
              className="mr-1"
              variant="filled"
              value={instrumentId}
              options={instruments}
              onChange={actions.ui.sequencer.setInstrument}
            />
          )}
          {trackId && tracks.length > 0 && (
            <Select
              variant="filled"
              value={trackId}
              options={tracks}
              onChange={actions.ui.sequencer.setTrack}
            />
          )}
        </div>
        <div className="flex flex-1 justify-end">
          <IconButton
            path={mdiTuneVertical}
            variant="filled"
            isSelected={true}
            onClick={() => {}}
            color="#000"
          />
        </div>
      </div>
      {!flowId && (
        <div className="flex flex-1 items-center justify-center">{/*  */}</div>
      )}
    </div>
  );
};
