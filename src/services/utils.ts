import prettyMilliseconds from "pretty-ms";
import React, { useEffect, useState } from "react";
import { AppState, LayoutType, UniqueId } from "./types";

const THIRTY_SECONDS = 30 * 1000;
const ONE_WEEK = 7 * 24 * 60 * 60 * 1000;

const convertTimestampToHuman = (when: number) => {
  const calc = Date.now() - when;

  if (calc < THIRTY_SECONDS) {
    return "Just now";
  } else if (calc > ONE_WEEK) {
    return new Date(when).toLocaleDateString();
  } else {
    const str = prettyMilliseconds(calc, {
      compact: true,
      separateMilliseconds: true,
      unitCount: 1,
      verbose: true,
    });
    return str + " ago";
  }
};

export const useHumanizedTime = (when: number) => {
  const [display, setDisplay] = useState(convertTimestampToHuman(when));

  useEffect(() => {
    const interval = setInterval(() => {
      setDisplay(convertTimestampToHuman(when));
    }, 1000);
    return () => {
      clearInterval(interval);
    };
  }, [when]);

  return display;
};

export const useAsync = <T>(
  asyncFn: () => Promise<T>,
  deps: React.DependencyList = [],
) => {
  useEffect(() => {
    const runAsync = async () => {
      try {
        await asyncFn();
      } catch (err) {
        console.log(err);
      }
    };

    runAsync();
  }, [asyncFn, ...deps]);
};

export const getLayoutName = (state: AppState, layoutId: UniqueId) => {
  const layout = state.score.layouts.by_id[layoutId];

  if (layout.title) {
    return layout.title;
  }

  if (layout.layout_type === LayoutType.FullScore) {
    return "Full Score";
  }

  const instrumentNames = state.score.players.order
    .filter((playerId) => layout.player_ids.includes(playerId))
    .flatMap((playerId) => {
      const player = state.score.players.by_id[playerId];
      return player.instrument_ids.map((instrumentId) => {
        const instrument = state.score.instruments.by_id[instrumentId];
        return (
          instrument.long_name +
          (instrument.count ? ` ${instrument.count}` : "")
        );
      });
    });

  if (instrumentNames.length === 0) {
    return "Empty Part";
  }

  if (instrumentNames.length === 1) {
    return instrumentNames[0];
  }

  const last = instrumentNames.pop();
  return `${instrumentNames.join(", ")} & ${last}`;
};

export const parseTimeSignature = (input: string) => {
  const validBeatTypes = [1, 2, 4, 8, 16, 32, 64, 128];
  const match = input.match(/^(\d{1,2})\/(\d{1,3})$/);

  if (!match) return null;

  const beats = parseInt(match[1], 10);
  const beatType = parseInt(match[2], 10);

  if (!validBeatTypes.includes(beatType)) return null;

  return { beats, beatType };
};
