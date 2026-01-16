import { useMemo } from "react";

import { repeatArray } from "../utils/repeat-array";

export const SLOT_HEIGHT = 21;

export enum KeyType {
  Black,
  White,
}

type Pattern = Array<{ type: KeyType; height: number; top: number }>;

export type Slots = {
  pattern: Pattern;
  slotHeight: number;
  height: number;
};

export const SLOT_PATTERN = (zoom: number) => [
  { type: KeyType.White, height: 36 * zoom },
  { type: KeyType.Black, height: 21 * zoom },
  { type: KeyType.White, height: 37.5 * zoom },
  { type: KeyType.Black, height: 21 * zoom },
  { type: KeyType.White, height: 37.5 * zoom },
  { type: KeyType.Black, height: 21 * zoom },
  { type: KeyType.White, height: 36 * zoom },
  { type: KeyType.White, height: 35 * zoom },
  { type: KeyType.Black, height: 21 * zoom },
  { type: KeyType.White, height: 35 * zoom },
  { type: KeyType.Black, height: 21 * zoom },
  { type: KeyType.White, height: 35 * zoom },
];

export const useSlotPattern = (octaves: number, zoomY = 1): Slots => {
  return useMemo(() => {
    const pattern = repeatArray(SLOT_PATTERN(zoomY), octaves);
    const slotHeight = SLOT_HEIGHT * zoomY;
    const height = slotHeight * pattern.length;

    return {
      pattern: pattern.map((slot, i) => {
        if (slot.type === KeyType.White) {
          // Calculate top position based on accumulated white key heights
          const whiteKeysBefore = pattern
            .slice(0, i)
            .filter((s) => s.type === KeyType.White);
          const top = whiteKeysBefore.reduce((acc, s) => acc + s.height, 0);
          return { ...slot, top };
        } else {
          return { ...slot, top: slot.height * i };
        }
      }),
      slotHeight,
      height,
    };
  }, [zoomY, octaves]);
};
