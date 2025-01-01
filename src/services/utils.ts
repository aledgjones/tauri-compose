import humanizeDuration from "humanize-duration";
import { useEffect, useState } from "preact/hooks";

export const useHumanizedTime = (timestamp: number): string => {
  const [humanizedTime, setHumanizedTime] = useState("");

  useEffect(() => {
    const updateHumanizedTime = () => {
      const now = Date.now();
      const durationInMs = timestamp * 1000 - now;
      const oneWeekInMs = 7 * 24 * 60 * 60 * 1000;

      if (durationInMs < oneWeekInMs) {
        setHumanizedTime(humanizeDuration(durationInMs, { largest: 2 }));
      } else {
        const date = new Date(timestamp * 1000);
        setHumanizedTime(date.toLocaleDateString());
      }
    };

    updateHumanizedTime();
    const intervalId = setInterval(updateHumanizedTime, 1000);

    return () => clearInterval(intervalId);
  }, [timestamp]);

  return humanizedTime;
};
