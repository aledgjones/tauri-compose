import { UniqueId } from "./types";

export interface ClickTarget {
  type: "Tone" | "Tuplet" | null;
  id: UniqueId;
}

/**
 * Determines what was clicked in the sequencer based on a pointer event
 * @param target - The HTML element that was clicked
 * @returns Object indicating what was clicked and the tone ID if applicable, or null if no valid target
 */
export const getSequencerClickTarget = (
  target: HTMLElement,
): ClickTarget | null => {
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

  const toneElement = target.closest("[data-tone-id]");

  if (toneElement) {
    const toneIdString = toneElement.getAttribute("data-tone-id");

    if (toneIdString) {
      return {
        type: "Tone",
        id: parseInt(toneIdString, 10) as UniqueId,
      };
    }
  }

  return null;
};
