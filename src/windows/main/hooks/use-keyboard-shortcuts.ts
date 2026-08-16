import { useEffect, useMemo, useRef } from "react";
import { useAppState, useTransportState } from "../../../services/state";
import { Accidental, Articulation } from "../../../services/types";
import { actions } from "../../../services/actions";
import { getSelectedTone } from "./use-selected-tone-info";
import {
  auditionInstrumentPitch,
  canAuditionInstrument,
} from "../utils/audition";

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
  const transport = useTransportState();
  const hasObservedInitialSelectionRef = useRef(false);
  const lastAuditionedSelectionKeyRef = useRef<string | null>(null);
  const selection = state.ui.sequencer.selection;
  const selectionFlowId = selection?.flow_id ?? null;

  const selectionTick = useMemo(() => {
    if (selection === null) {
      return null;
    }

    const resolveTupletHostTick = (
      trackId: number,
      tupletId: number | null | undefined,
      localTick: number,
    ) => {
      if (tupletId === null || tupletId === undefined) {
        return localTick;
      }

      const track = state.score.tracks.by_id[trackId];
      const tupletEntry = track?.entries.by_id[tupletId];
      if (tupletEntry?.type !== "Tuplet") {
        return null;
      }

      return tupletEntry.tick + localTick;
    };

    if (selection.type === "Tone") {
      const track = state.score.tracks.by_id[selection.track_id];
      const tone = track
        ? getSelectedTone(track, selection.tone_id, selection.tuplet_id)
        : null;

      if (!tone) {
        return null;
      }

      return resolveTupletHostTick(
        selection.track_id,
        selection.tuplet_id,
        tone.tick,
      );
    }

    if (selection.type === "Rest") {
      return resolveTupletHostTick(
        selection.track_id,
        selection.tuplet_id,
        selection.tick,
      );
    }

    return selection.tick;
  }, [selection, state.score.tracks.by_id]);

  const toneSelectionInfo = useMemo(() => {
    if (selection === null || selection.type !== "Tone") {
      return null;
    }

    const track = state.score.tracks.by_id[selection.track_id];
    if (!track) {
      return null;
    }

    const tone = getSelectedTone(track, selection.tone_id, selection.tuplet_id);
    if (!tone) {
      return null;
    }

    return {
      tone,
      toneId: selection.tone_id,
      instrumentId: selection.instrument_id,
      trackId: selection.track_id,
      flowId: selection.flow_id,
      tupletId: selection.tuplet_id ?? null,
    };
  }, [selection, state.score.tracks.by_id]);

  const selectedToneAudition = useMemo(() => {
    if (!toneSelectionInfo) {
      return null;
    }

    return {
      selectionKey: [
        toneSelectionInfo.flowId,
        toneSelectionInfo.trackId,
        toneSelectionInfo.tupletId ?? "root",
        toneSelectionInfo.toneId,
      ].join(":"),
      instrumentId: toneSelectionInfo.instrumentId,
      pitch: toneSelectionInfo.tone.pitch.int,
    };
  }, [toneSelectionInfo]);

  const canAutoAuditionSelectedTone = useMemo(() => {
    return Boolean(
      selectedToneAudition &&
      canAuditionInstrument(state, selectedToneAudition.instrumentId),
    );
  }, [selectedToneAudition, state]);

  const deletionInfo = useMemo(() => {
    if (selection === null) {
      return null;
    }

    switch (selection.type) {
      case "Bar": {
        return {
          deleteSelection: async () => {
            const tick = await actions.score.flow.deleteBar(
              selection.flow_id,
              selection.tick,
            );
            await actions.ui.sequencer.selectBar(selection.flow_id, tick);
          },
        };
      }
      case "InsertBarAfter":
      case "DeleteBar":
        return null;
      case "Tone": {
        if (!toneSelectionInfo) {
          return null;
        }

        return {
          deleteSelection: () =>
            actions.score.tone.remove(
              toneSelectionInfo.trackId,
              toneSelectionInfo.tupletId,
              toneSelectionInfo.toneId,
            ),
        };
      }
      case "Tempo": {
        return {
          deleteSelection: () =>
            actions.score.tempo.clearAtTick(selection.flow_id, selection.tick),
        };
      }
      case "TimeSignature": {
        return {
          deleteSelection: () =>
            actions.score.timeSignature.clearAtTick(
              selection.flow_id,
              selection.tick,
            ),
        };
      }
      case "KeySignature": {
        return {
          deleteSelection: () =>
            actions.score.keySignature.clearAtTick(
              selection.flow_id,
              selection.tick,
            ),
        };
      }
      case "Clef": {
        return {
          deleteSelection: () =>
            actions.score.clef.clear(
              selection.flow_id,
              selection.tick,
              selection.stave_id,
            ),
        };
      }
      case "PlayingTechnique": {
        return {
          deleteSelection: () =>
            actions.score.playingTechnique.remove(
              selection.flow_id,
              selection.stave_id,
              selection.playing_technique_id,
            ),
        };
      }
      case "Tuplet": {
        return {
          deleteSelection: () =>
            actions.score.tuplet.remove(
              selection.track_id,
              selection.tuplet_id,
            ),
        };
      }
      case "Rest":
        return null;
    }
  }, [selection, toneSelectionInfo]);

  useEffect(() => {
    const currentSelectionKey = selectedToneAudition?.selectionKey ?? null;

    if (!hasObservedInitialSelectionRef.current) {
      hasObservedInitialSelectionRef.current = true;

      if (currentSelectionKey !== null && canAutoAuditionSelectedTone) {
        lastAuditionedSelectionKeyRef.current = currentSelectionKey;
      } else {
        lastAuditionedSelectionKeyRef.current = null;
      }

      return;
    }

    if (currentSelectionKey === null) {
      lastAuditionedSelectionKeyRef.current = null;
      return;
    }

    if (lastAuditionedSelectionKeyRef.current === currentSelectionKey) {
      return;
    }

    if (!selectedToneAudition || !canAutoAuditionSelectedTone) {
      return;
    }

    lastAuditionedSelectionKeyRef.current = currentSelectionKey;

    void auditionInstrumentPitch(
      state,
      selectedToneAudition.instrumentId,
      selectedToneAudition.pitch,
    );
  }, [canAutoAuditionSelectedTone, selectedToneAudition, state]);

  useEffect(() => {
    const auditionTone = async (instrumentId: number, pitch: number) => {
      await auditionInstrumentPitch(state, instrumentId, pitch);
    };

    const toneShortcuts: Array<{
      key: string;
      modifiers: Modifiers;
      action: () => Promise<number | null | void>;
      shouldAudition?: boolean;
    }> = [];

    if (toneSelectionInfo) {
      const { trackId, toneId, tupletId } = toneSelectionInfo;

      const shiftPitchByStep = (steps: number) =>
        actions.score.tone.shiftPitchByStep(trackId, tupletId, toneId, steps);

      const shiftPitchByOctave = (octaves: number) =>
        actions.score.tone.shiftPitchByOctave(
          trackId,
          tupletId,
          toneId,
          octaves,
        );

      const setAccidental = (accidental: Accidental) =>
        actions.score.tone.setAccidental(trackId, tupletId, toneId, accidental);

      const setArticulation = (articulation: Articulation) =>
        actions.score.tone.setArticulation(
          trackId,
          tupletId,
          toneId,
          articulation,
        );

      toneShortcuts.push(
        {
          key: "ArrowUp",
          modifiers: { command: false, control: false, option: true },
          action: () => shiftPitchByStep(1),
          shouldAudition: true,
        },
        {
          key: "ArrowUp",
          modifiers: { command: true, control: false, option: true },
          action: () => shiftPitchByOctave(1),
          shouldAudition: true,
        },
        {
          key: "ArrowDown",
          modifiers: { command: false, control: false, option: true },
          action: () => shiftPitchByStep(-1),
          shouldAudition: true,
        },
        {
          key: "ArrowDown",
          modifiers: { command: true, control: false, option: true },
          action: () => shiftPitchByOctave(-1),
          shouldAudition: true,
        },
        {
          key: "=",
          modifiers: { command: false, control: false, option: false },
          action: () => setAccidental(Accidental.Sharp),
          shouldAudition: true,
        },
        {
          key: "-",
          modifiers: { command: false, control: false, option: false },
          action: () => setAccidental(Accidental.Flat),
          shouldAudition: true,
        },
        {
          key: "0",
          modifiers: { command: false, control: false, option: false },
          action: () => setAccidental(Accidental.Natural),
          shouldAudition: true,
        },
        {
          key: "]",
          modifiers: { command: false, control: false, option: false },
          action: () => setArticulation(Articulation.Staccato),
        },
        {
          key: "\\",
          modifiers: { command: false, control: false, option: false },
          action: () => setArticulation(Articulation.Tenuto),
        },
        {
          key: "[",
          modifiers: { command: false, control: false, option: false },
          action: () => setArticulation(Articulation.Staccatissimo),
        },
        {
          key: "'",
          modifiers: { command: false, control: false, option: false },
          action: () => setArticulation(Articulation.StaccatoTenuto),
        },
        {
          key: "}",
          modifiers: { command: false, control: false, option: false },
          action: () => setArticulation(Articulation.Accent),
        },
        {
          key: "|",
          modifiers: { command: false, control: false, option: false },
          action: () => setArticulation(Articulation.Marcato),
        },
        {
          key: "{",
          modifiers: { command: false, control: false, option: false },
          action: () => setArticulation(Articulation.Stressed),
        },
        {
          key: '"',
          modifiers: { command: false, control: false, option: false },
          action: () => setArticulation(Articulation.Unstressed),
        },
      );
    }

    const handleKeyDown = async (e: KeyboardEvent) => {
      if (e.defaultPrevented || isEditableTarget(e.target)) {
        return;
      }

      if (
        e.key === " " &&
        matchesModifiers(e, { command: false, control: false, option: false })
      ) {
        e.preventDefault();

        if (transport.playing) {
          await actions.transport.pause();
        } else {
          await actions.transport.play();
        }

        return;
      }

      if (
        (e.key === "p" || e.key === "P") &&
        matchesModifiers(e, { command: false, control: false, option: false })
      ) {
        e.preventDefault();

        if (selectionTick !== null) {
          if (
            selectionFlowId !== null &&
            state.ui.flow_id !== selectionFlowId
          ) {
            await actions.ui.setActiveFlowId(selectionFlowId);
          }
          await actions.transport.seek(selectionTick);
          await actions.transport.play();
          return;
        }

        if (transport.playing) {
          await actions.transport.pause();
        } else {
          await actions.transport.play();
        }

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

        return;
      }

      if (toneShortcuts.length === 0) {
        return;
      }

      for (const shortcut of toneShortcuts) {
        if (e.key === shortcut.key && matchesModifiers(e, shortcut.modifiers)) {
          e.preventDefault();
          const result = await shortcut.action();

          if (
            shortcut.shouldAudition &&
            typeof result === "number" &&
            toneSelectionInfo
          ) {
            await auditionTone(toneSelectionInfo.instrumentId, result);
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
    selectionFlowId,
    selectionTick,
    state,
    toneSelectionInfo,
    transport.playing,
  ]);
};
