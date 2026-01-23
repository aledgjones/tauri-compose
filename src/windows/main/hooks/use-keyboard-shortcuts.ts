import { useEffect, useMemo } from "react";
import { useAppState } from "../../../services/state";
import { Accidental, SequencerSelectionType } from "../../../services/types";
import { actions } from "../../../services/actions";
import { sampler } from "../../../sampler";
import { Expression } from "../../../sampler/types";

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
    if (
      !selection ||
      selection.selection_type !== SequencerSelectionType.Tone
    ) {
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
      instrumentId: selection.instrument_id,
      trackId: selection.track_id,
      flowId: selection.flow_id,
    };
  }, [state]);

  useEffect(() => {
    if (!selectionInfo) {
      return;
    }

    const auditionTone = (pitch: number) => {
      if (!state.preferences.audition) {
        return;
      }

      sampler.play(
        selectionInfo.instrumentId.toString(),
        Expression.Natural,
        pitch,
        sampler.ctx.currentTime,
        0.5,
      );
    };

    const shortcuts: Array<{
      key: string;
      modifiers: Modifiers;
      action: () => Promise<number | null | void>;
      shouldAudition?: boolean;
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
        shouldAudition: true,
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
        shouldAudition: true,
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
        shouldAudition: true,
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
        shouldAudition: true,
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
        shouldAudition: true,
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
        shouldAudition: true,
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
        shouldAudition: true,
      },
      {
        key: "Backspace",
        modifiers: { command: false, control: false, option: false },
        action: () =>
          actions.score.tone.remove(
            selectionInfo.trackId,
            selectionInfo.toneId,
          ),
      },
      {
        key: "Delete",
        modifiers: { command: false, control: false, option: false },
        action: () =>
          actions.score.tone.remove(
            selectionInfo.trackId,
            selectionInfo.toneId,
          ),
      },
    ];

    const handleKeyDown = async (e: KeyboardEvent) => {
      for (const shortcut of shortcuts) {
        if (e.key === shortcut.key && matchesModifiers(e, shortcut.modifiers)) {
          e.preventDefault();
          const result = await shortcut.action();

          if (shortcut.shouldAudition && typeof result === "number") {
            auditionTone(result);
          }

          return; // Stop after first match
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [selectionInfo, state.preferences.audition]);
};
