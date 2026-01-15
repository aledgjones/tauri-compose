import { useEffect, useMemo } from "react";
import { useAppState } from "../../../services/state";
import { Accidental, SelectionType } from "../../../services/types";
import { actions } from "../../../services/actions";

type Modifiers = { command: boolean; control: boolean; option: boolean };

const matchesModifiers = (e: KeyboardEvent, pattern: Modifiers) => {
  return (
    e.metaKey === pattern.command &&
    e.ctrlKey === pattern.control &&
    e.altKey === pattern.option
  );
};

export const useKeyboardShortcuts = () => {
  const state = useAppState();

  const selectionInfo = useMemo(() => {
    const selection = state.ui.sequencer.selection;
    if (!selection || selection.selection_type !== SelectionType.Tone) {
      return null;
    }

    const track = state.score.tracks.by_id[selection.track_id];
    if (!track) {
      return null;
    }

    const tone = track.entries.by_id[selection.id];
    if (!tone || tone.type !== "Tone") {
      return null;
    }

    return {
      tone,
      toneId: selection.id,
      trackId: selection.track_id,
      flowId: selection.flow_id,
    };
  }, [state]);

  useEffect(() => {
    if (!selectionInfo) {
      return;
    }

    const shortcuts: Array<{
      key: string;
      modifiers: Modifiers;
      action: () => Promise<void>;
    }> = [
      {
        key: "ArrowUp",
        modifiers: { command: false, control: false, option: true },
        action: () =>
          actions.score.tone.shiftPitchByStep(
            selectionInfo.flowId,
            selectionInfo.trackId,
            selectionInfo.toneId,
            1,
          ),
      },
      {
        key: "ArrowUp",
        modifiers: { command: true, control: false, option: true },
        action: () =>
          actions.score.tone.shiftPitchByOctave(
            selectionInfo.trackId,
            selectionInfo.toneId,
            1,
          ),
      },
      {
        key: "ArrowDown",
        modifiers: { command: false, control: false, option: true },
        action: () =>
          actions.score.tone.shiftPitchByStep(
            selectionInfo.flowId,
            selectionInfo.trackId,
            selectionInfo.toneId,
            -1,
          ),
      },
      {
        key: "ArrowDown",
        modifiers: { command: true, control: false, option: true },
        action: () =>
          actions.score.tone.shiftPitchByOctave(
            selectionInfo.trackId,
            selectionInfo.toneId,
            -1,
          ),
      },
      {
        key: "=",
        modifiers: { command: false, control: false, option: false },
        action: () =>
          actions.score.tone.setAccidental(
            selectionInfo.trackId,
            selectionInfo.toneId,
            Accidental.Sharp,
          ),
      },
      {
        key: "-",
        modifiers: { command: false, control: false, option: false },
        action: () =>
          actions.score.tone.setAccidental(
            selectionInfo.trackId,
            selectionInfo.toneId,
            Accidental.Flat,
          ),
      },
      {
        key: "0",
        modifiers: { command: false, control: false, option: false },
        action: () =>
          actions.score.tone.setAccidental(
            selectionInfo.trackId,
            selectionInfo.toneId,
            Accidental.Natural,
          ),
      },
    ];

    const handleKeyDown = async (e: KeyboardEvent) => {
      for (const shortcut of shortcuts) {
        if (e.key === shortcut.key && matchesModifiers(e, shortcut.modifiers)) {
          e.preventDefault();
          await shortcut.action();
          return; // Stop after first match
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [selectionInfo]);
};
