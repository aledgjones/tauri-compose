import { useEffect, useMemo } from "react";
import { useAppState } from "../../../services/state";
import { SelectionType } from "../../../services/types";

export const useKeyboardShortcuts = () => {
  const state = useAppState();

  const selection = useMemo(() => {
    const selection = state.ui.sequencer.selection;
    if (!selection || selection.selection_type !== SelectionType.Tone) {
      return;
    }

    const track = state.score.tracks.by_id[selection.track_id];
    if (!track) {
      return;
    }

    const tone = track.entries.by_id[selection.id];
    if (!tone || tone.type !== "Tone") {
      return;
    }

    return tone;
  }, [state]);

  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      if (!selection) {
        return;
      }

      switch (e.key) {
        case "≠": {
          if (!e.metaKey && !e.ctrlKey) {
            e.preventDefault();

            console.log("Increase pitch by step");

            // const newPitch = Pitch.fromMidiNumber(
            //   selection.pitch.toMidiNumber() + 1,
            // );
            // await actions.score.tone.setPitch(
            //   state.ui.sequencer.track!,
            //   selection.id,
            //   newPitch,
            // );
          }
          break;
        }
        case "\u2013": {
          if (!e.metaKey && !e.ctrlKey && e.altKey) {
            e.preventDefault();

            console.log("Decrease pitch by step");
          }
          break;
        }
      }

      //   // Accidental changes
      //   if (
      //     e.key === "0" &&
      //     !e.metaKey &&
      //     !e.ctrlKey &&
      //     !e.shiftKey &&
      //     !e.altKey
      //   ) {
      //     e.preventDefault();
      //     await actions.score.tone.setPitch(
      //       trackId,
      //       toneId,
      //       currentPitchInt,
      //       Accidental.Natural,
      //     );
      //     return;
      //   }
      //   if (
      //     e.key === "-" &&
      //     !e.metaKey &&
      //     !e.ctrlKey &&
      //     !e.shiftKey &&
      //     !e.altKey
      //   ) {
      //     e.preventDefault();
      //     await actions.score.tone.setPitch(
      //       trackId,
      //       toneId,
      //       currentPitchInt,
      //       Accidental.Flat,
      //     );
      //     return;
      //   }
      //   if (
      //     e.key === "=" &&
      //     !e.metaKey &&
      //     !e.ctrlKey &&
      //     !e.shiftKey &&
      //     !e.altKey
      //   ) {
      //     e.preventDefault();
      //     await actions.score.tone.setPitch(
      //       trackId,
      //       toneId,
      //       currentPitchInt,
      //       Accidental.Sharp,
      //     );
      //     return;
      //   }
      //   // Pitch movement by step (option + arrow)
      //   if (e.altKey && !e.shiftKey && !e.metaKey && !e.ctrlKey) {
      //     if (e.key === "ArrowUp") {
      //       e.preventDefault();
      //       await actions.score.tone.setPitch(
      //         trackId,
      //         toneId,
      //         currentPitchInt + 1,
      //       );
      //       return;
      //     }
      //     if (e.key === "ArrowDown") {
      //       e.preventDefault();
      //       await actions.score.tone.setPitch(
      //         trackId,
      //         toneId,
      //         currentPitchInt - 1,
      //       );
      //       return;
      //     }
      //   }
      //   // Pitch movement by octave (shift + option + arrow)
      //   if (e.altKey && e.shiftKey && !e.metaKey && !e.ctrlKey) {
      //     if (e.key === "ArrowUp") {
      //       e.preventDefault();
      //       await actions.score.tone.setPitch(
      //         trackId,
      //         toneId,
      //         currentPitchInt + 12,
      //       );
      //       return;
      //     }
      //     if (e.key === "ArrowDown") {
      //       e.preventDefault();
      //       await actions.score.tone.setPitch(
      //         trackId,
      //         toneId,
      //         currentPitchInt - 12,
      //       );
      //       return;
      //     }
      //   }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [selection]);
};
