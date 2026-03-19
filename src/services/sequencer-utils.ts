import { UniqueId } from "./types";

type ToneClickTarget = {
  type: "Tone";
  id: UniqueId;
  tuplet_id?: UniqueId;
};

type TupletClickTarget = {
  type: "Tuplet";
  id: UniqueId;
};

export type ClickTarget = ToneClickTarget | TupletClickTarget;

/**
 * Determines what was clicked in the sequencer based on a pointer event
 * @param target - The HTML element that was clicked
 * @returns Object indicating what was clicked and the tone ID if applicable, or null if no valid target
 */
export const getSequencerClickTarget = (
  target: HTMLElement,
): ClickTarget | null => {
  const toneElement = target.closest("[data-tone-id]");

  if (toneElement) {
    const toneIdString = toneElement.getAttribute("data-tone-id");
    const tupletIdString = toneElement
      .closest("[data-tuplet-id]")
      ?.getAttribute("data-tuplet-id");

    if (toneIdString) {
      return {
        type: "Tone",
        id: parseInt(toneIdString, 10) as UniqueId,
        ...(tupletIdString
          ? { tuplet_id: parseInt(tupletIdString, 10) as UniqueId }
          : {}),
      };
    }
  }

  const tupletElement = target.closest("[data-tuplet-id]");

  if (tupletElement) {
    const tupletIdString = tupletElement.getAttribute("data-tuplet-id");

    if (tupletIdString) {
      return {
        type: "Tuplet",
        id: parseInt(tupletIdString, 10) as UniqueId,
      };
    }
  }

  return null;
};
