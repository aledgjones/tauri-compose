import { useEffect, useMemo } from "react";
import { useAppState } from "../../../services/state";
import { Accidental, Articulation } from "../../../services/types";
import { actions } from "../../../services/actions";

type Modifiers = { command: boolean; control: boolean; option: boolean };

const NON_TEXT_INPUT_TYPES = new Set([
  "button",
  "checkbox",
  "color",
  "file",
  "hidden",
  "image",
  "radio",
  "range",
  "reset",
  "submit",
]);

const matchesModifiers = (e: KeyboardEvent, pattern: Modifiers) => {
  return (
    e.metaKey === pattern.command &&
    e.ctrlKey === pattern.control &&
    e.altKey === pattern.option
  );
};

const isEditableTarget = (target: EventTarget | null) => {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  if (target.isContentEditable || target.closest('[contenteditable="true"]')) {
    return true;
  }

  if (target instanceof HTMLTextAreaElement) {
    return true;
  }

  if (target instanceof HTMLInputElement) {
    return !NON_TEXT_INPUT_TYPES.has(target.type);
  }

  return (
    target.getAttribute("role") === "textbox" ||
    target.closest('[role="textbox"]') !== null
  );
};

export const useKeyboardShortcuts = () => {
  const state = useAppState();
  const selection = state.ui.sequencer.selection;

  const toneSelectionInfo = useMemo(() => {
    if (selection === null || selection.type !== "Tone") {
      return null;
    }

    const track = state.score.tracks.by_id[selection.track_id];
    if (!track) {
      return null;
    }

    const entry = track.entries.by_id[selection.tone_id];
    if (!entry || entry.type !== "Tone") {
      return null;
    }

    return {
      tone: entry,
      toneId: selection.tone_id,
      instrumentId: selection.instrument_id,
      trackId: selection.track_id,
      flowId: selection.flow_id,
    };
  }, [selection, state.score.tracks.by_id]);

  const deletionInfo = useMemo(() => {
    if (selection === null) {
      return null;
    }

    switch (selection.type) {
      case "Tone": {
        if (!toneSelectionInfo) {
          return null;
        }

        return {
          clearSelectionAfterDelete: true,
          deleteSelection: () =>
            actions.score.tone.remove(
              toneSelectionInfo.trackId,
              toneSelectionInfo.toneId,
            ),
        };
      }
      case "Tempo": {
        return {
          clearSelectionAfterDelete: true,
          deleteSelection: () =>
            actions.score.tempo.clearAtTick(selection.flow_id, selection.tick),
        };
      }
      case "TimeSignature": {
        return {
          clearSelectionAfterDelete: selection.tick !== 0,
          deleteSelection: () =>
            actions.score.timeSignature.clearAtTick(
              selection.flow_id,
              selection.tick,
            ),
        };
      }
      case "KeySignature": {
        return {
          clearSelectionAfterDelete: true,
          deleteSelection: () =>
            actions.score.keySignature.clearAtTick(
              selection.flow_id,
              selection.tick,
            ),
        };
      }
      case "Clef": {
        return {
          clearSelectionAfterDelete: selection.tick !== 0,
          deleteSelection: () =>
            actions.score.clef.clear(
              selection.flow_id,
              selection.tick,
              selection.stave_id,
            ),
        };
      }
      case "Rest":
        return null;
    }
  }, [selection, toneSelectionInfo]);

  useEffect(() => {
    if (!selection) {
      return;
    }

    const activeToneSelectionInfo = toneSelectionInfo;

    const auditionTone = (pitch: number) => {
      if (!activeToneSelectionInfo) {
        return;
      }

      if (!state.preferences.audition) {
        return;
      }

      const instrument =
        state.score.instruments.by_id[activeToneSelectionInfo.instrumentId];
      if (!instrument?.assigned_plugin_id) {
        return;
      }

      actions.sampler.audition(instrument.assigned_plugin_id, pitch);
    };

    const toneShortcuts: Array<{
      key: string;
      modifiers: Modifiers;
      action: () => Promise<number | null | void>;
      shouldAudition?: boolean;
    }> = activeToneSelectionInfo
      ? [
          {
            key: "ArrowUp",
            modifiers: { command: false, control: false, option: true },
            action: () =>
              actions.score.tone.shiftPitchByStep(
                activeToneSelectionInfo.flowId,
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                1,
              ),
            shouldAudition: true,
          },
          {
            key: "ArrowUp",
            modifiers: { command: true, control: false, option: true },
            action: () =>
              actions.score.tone.shiftPitchByOctave(
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                1,
              ),
            shouldAudition: true,
          },
          {
            key: "ArrowDown",
            modifiers: { command: false, control: false, option: true },
            action: () =>
              actions.score.tone.shiftPitchByStep(
                activeToneSelectionInfo.flowId,
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                -1,
              ),
            shouldAudition: true,
          },
          {
            key: "ArrowDown",
            modifiers: { command: true, control: false, option: true },
            action: () =>
              actions.score.tone.shiftPitchByOctave(
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                -1,
              ),
            shouldAudition: true,
          },
          {
            key: "=",
            modifiers: { command: false, control: false, option: false },
            action: () =>
              actions.score.tone.setAccidental(
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                Accidental.Sharp,
              ),
            shouldAudition: true,
          },
          {
            key: "-",
            modifiers: { command: false, control: false, option: false },
            action: () =>
              actions.score.tone.setAccidental(
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                Accidental.Flat,
              ),
            shouldAudition: true,
          },
          {
            key: "0",
            modifiers: { command: false, control: false, option: false },
            action: () =>
              actions.score.tone.setAccidental(
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                Accidental.Natural,
              ),
            shouldAudition: true,
          },
          {
            key: "]",
            modifiers: { command: false, control: false, option: false },
            action: () => {
              return actions.score.tone.setArticulation(
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                Articulation.Staccato,
              );
            },
          },
          {
            key: "\\",
            modifiers: { command: false, control: false, option: false },
            action: () => {
              return actions.score.tone.setArticulation(
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                Articulation.Tenuto,
              );
            },
          },
          {
            key: "[",
            modifiers: { command: false, control: false, option: false },
            action: () => {
              return actions.score.tone.setArticulation(
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                Articulation.Staccatissimo,
              );
            },
          },
          {
            key: "'",
            modifiers: { command: false, control: false, option: false },
            action: () => {
              return actions.score.tone.setArticulation(
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                Articulation.StaccatoTenuto,
              );
            },
          },
          {
            key: "}",
            modifiers: { command: false, control: false, option: false },
            action: () => {
              return actions.score.tone.setArticulation(
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                Articulation.Accent,
              );
            },
          },
          {
            key: "|",
            modifiers: { command: false, control: false, option: false },
            action: () => {
              return actions.score.tone.setArticulation(
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                Articulation.Marcato,
              );
            },
          },
          {
            key: "{",
            modifiers: { command: false, control: false, option: false },
            action: () => {
              return actions.score.tone.setArticulation(
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                Articulation.Stressed,
              );
            },
          },
          {
            key: '"',
            modifiers: { command: false, control: false, option: false },
            action: () => {
              return actions.score.tone.setArticulation(
                activeToneSelectionInfo.trackId,
                activeToneSelectionInfo.toneId,
                Articulation.Unstressed,
              );
            },
          },
        ]
      : [];

    const handleKeyDown = async (e: KeyboardEvent) => {
      if (e.defaultPrevented || isEditableTarget(e.target)) {
        return;
      }

      if (
        e.key === "Escape" &&
        matchesModifiers(e, { command: false, control: false, option: false })
      ) {
        e.preventDefault();
        await actions.ui.sequencer.clearSelection();
        return;
      }

      if (
        deletionInfo &&
        (e.key === "Backspace" || e.key === "Delete") &&
        matchesModifiers(e, { command: false, control: false, option: false })
      ) {
        e.preventDefault();
        await deletionInfo.deleteSelection();

        if (deletionInfo.clearSelectionAfterDelete) {
          await actions.ui.sequencer.clearSelection();
        }

        return;
      }

      if (!activeToneSelectionInfo) {
        return;
      }

      for (const shortcut of toneShortcuts) {
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
  }, [
    deletionInfo,
    selection,
    state.preferences.audition,
    state.score.instruments.by_id,
    toneSelectionInfo,
  ]);
};
