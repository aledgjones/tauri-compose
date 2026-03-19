import { FC, Fragment } from "react";
import { TickConfig } from "../hooks/use-ticks";
import classNames from "classnames";
import { Tuplet } from "../../../services/types";
import { getTupletHostTicks } from "./sequencer-tuplet-utils";
import { lightenColor } from "../../../services/utils";

type TickBarProps = {
  ticks: TickConfig;
  className?: string;
  barNumberClassName?: string;
  tuplets?: Tuplet[];
  annotationColor?: string;
};

export const TickBar: FC<TickBarProps> = ({
  ticks,
  className,
  barNumberClassName = "leading-7",
  tuplets = [],
  annotationColor = "#9ca3af",
}) => {
  const accentColor = lightenColor(annotationColor, 0.3);

  return (
    <div
      className={classNames("h-7", className)}
      style={{ width: ticks.tickWidth * ticks.length }}
      data-testid="sequencer-tick-bar"
    >
      {ticks.entries.map((entry) => {
        return (
          <Fragment key={entry.x}>
            <div
              style={{ left: `${entry.x}px` }}
              className={classNames("absolute bottom-0 w-px bg-neutral-500", {
                "h-full": entry.type === "bar",
                "h-[50%]": entry.type === "beat",
                "h-[25%]": entry.type === "sub",
              })}
            >
              {entry.barNumber && (
                <p
                  className={classNames(
                    "text-white h-full pl-3 text-xs",
                    barNumberClassName,
                  )}
                >
                  {entry.barNumber}
                </p>
              )}
            </div>
          </Fragment>
        );
      })}

      {tuplets.map((tuplet) => {
        const width = getTupletHostTicks(tuplet) * ticks.tickWidth;

        return (
          <div
            key={tuplet.id}
            className="absolute bottom-0 pointer-events-none"
            style={{ left: tuplet.tick * ticks.tickWidth, width }}
            data-testid={`sequencer-tick-bar-tuplet-${tuplet.id}`}
          >
            <div
              className="absolute bottom-0 left-0 w-full border-b-2"
              style={{ borderColor: accentColor }}
            />
            <div
              className="absolute bottom-1 left-1/2 inline-flex -translate-x-1/2 items-center justify-center font-mono tabular-nums text-[10px] font-semibold"
              style={{ color: accentColor }}
              data-testid={`sequencer-tuplet-label-${tuplet.id}`}
            >
              <span>{tuplet.note_count}</span>
              <span className="px-[0.08em]">:</span>
              <span>{tuplet.in_space_of}</span>
            </div>
          </div>
        );
      })}
    </div>
  );
};
