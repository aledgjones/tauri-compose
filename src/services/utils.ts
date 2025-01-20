import prettyMilliseconds from "pretty-ms";
import { useEffect, useState } from "react";

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
