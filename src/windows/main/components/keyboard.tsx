import { FC } from "react";
import { KeyType, Slots } from "../hooks/use-slots";
import classNames from "classnames";

interface Props {
  slots: Slots;
  className?: string;
}

export const Keyboard: FC<Props> = ({ className, slots }) => {
  const whiteKeys = slots.pattern
    .filter((slot) => slot.type === KeyType.White)
    .slice()
    .reverse();
  const blackKeys = slots.pattern.filter((slot) => slot.type === KeyType.Black);

  return (
    <div
      className={classNames("relative", className)}
      style={{ height: slots.height }}
    >
      {whiteKeys.map((key, i) => {
        return (
          <div
            key={i}
            className="absolute bg-white rounded-r-sm left-0 w-full flex items-center justify-end pr-2"
            style={{ height: key.height - 1, top: key.top }}
          >
            {i % 7 === 0 && <span>C{i / 7}</span>}
          </div>
        );
      })}
      {blackKeys.map((key, i) => {
        return (
          <div
            key={i}
            className="z-10 absolute bg-neutral-950 rounded-r-sm left-0 w-[75%]"
            style={{ height: key.height, top: key.top }}
          />
        );
      })}
    </div>
  );
};
