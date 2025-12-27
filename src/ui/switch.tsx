import classNames from "classnames";
import { FC, PointerEvent } from "react";

type Props = {
  value: boolean;
  label?: string;
  description?: string;
  onChange: (value: boolean) => void;
  className?: string;
  onPointerDown?: (event: PointerEvent<HTMLDivElement>) => void;
};

export const Switch: FC<Props> = ({
  value,
  onChange,
  className,
  onPointerDown,
  label,
  description,
}) => {
  return (
    <div className="flex items-center">
      <div className="flex-1 mr-4">
        {label && <label className="text-white text-sm">{label}</label>}
        {description && (
          <p className="text-xs text-neutral-500 mb-2">{description}</p>
        )}
      </div>
      <div
        onClick={() => onChange(!value)}
        onPointerDown={onPointerDown}
        className={classNames("relative h-8 w-12 flex items-center", className)}
      >
        <div
          className={classNames(
            "h-7 w-full rounded-full transition-colors flex items-center px-0.5 absolute top-1/2 -translate-y-1/2",
            {
              "bg-sky-500": value,
              "bg-neutral-700": !value,
            },
          )}
        >
          <div
            className={classNames(
              "rounded-full shadow-md transition-all duration-200",
              {
                "bg-white h-6 w-6": value,
                "bg-neutral-400 h-4.5 w-4.5": !value,
              },
            )}
            style={{ transform: `translateX(${value ? "20px" : "2px"})` }}
          ></div>
        </div>
      </div>
    </div>
  );
};
