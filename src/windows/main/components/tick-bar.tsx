import { FC, Fragment } from "react";
import { TickConfig } from "../hooks/use-ticks";
import classNames from "classnames";

type TickBarProps = {
  ticks: TickConfig;
  className?: string;
};

export const TickBar: FC<TickBarProps> = ({ ticks, className }) => {
  return (
    <div
      className={classNames("h-[28px]", className)}
      style={{ width: ticks.tickWidth * ticks.length }}
    >
      {ticks.entries.map((entry) => {
        return (
          <Fragment key={entry.x}>
            <div
              style={{ left: `${entry.x}px` }}
              className={classNames(
                "absolute bottom-0 w-[1px] bg-neutral-500",
                {
                  "h-full": entry.type === "bar",
                  "h-[50%]": entry.type === "beat",
                  "h-[25%]": entry.type === "sub",
                }
              )}
            >
              {entry.barNumber && (
                <p className="text-white h-full pl-3 text-xs leading-[28px]">
                  {entry.barNumber}
                </p>
              )}
            </div>
          </Fragment>
        );
      })}
    </div>
  );
};
