import classNames from "classnames";
import { FC, PointerEvent } from "react";

type Props = {
  value: boolean;
  onChange: (value: boolean) => void;
  className?: string;
  onPointerDown?: (event: PointerEvent<HTMLDivElement>) => void;
};

export const Switch: FC<Props> = ({
  value,
  onChange,
  className,
  onPointerDown,
}) => {
  return (
    <div
      onClick={() => onChange(!value)}
      onPointerDown={onPointerDown}
      className={classNames(
        "relative h-[32px] w-[48px] flex items-center",
        className
      )}
    >
      <div
        className={classNames(
          "h-[28px] w-full rounded-full transition-colors flex items-center px-[2px] absolute top-1/2 -translate-y-1/2",
          {
            "bg-sky-500": value,
            "bg-neutral-700": !value,
          }
        )}
      >
        <div
          className={classNames(
            "rounded-full shadow-md transition-all duration-200",
            {
              "bg-white h-[24px] w-[24px]": value,
              "bg-neutral-400 h-[18px] w-[18px]": !value,
            }
          )}
          style={{ transform: `translateX(${value ? "20px" : "2px"})` }}
        ></div>
      </div>
    </div>
  );
};
