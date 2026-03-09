import classNames from "classnames";
import { FC, PointerEvent } from "react";

type Props = {
  value: boolean;
  label?: string;
  description?: string;
  onChange: (value: boolean) => void;
  className?: string;
  onPointerDown?: (event: PointerEvent<HTMLButtonElement>) => void;
  small?: boolean;
  disabled?: boolean;
};

export const Switch: FC<Props> = ({
  value,
  onChange,
  className,
  onPointerDown,
  label,
  description,
  small = false,
  disabled = false,
}) => {
  return (
    <div className="flex items-center">
      {(label || description) && (
        <div className="flex-1 mr-4">
          {label && <label className="text-white text-sm">{label}</label>}
          {description && (
            <p className="text-xs text-neutral-500 mb-2">{description}</p>
          )}
        </div>
      )}
      <button
        onClick={() => {
          if (!disabled) {
            onChange(!value);
          }
        }}
        onPointerDown={onPointerDown}
        type="button"
        role="switch"
        aria-checked={value}
        aria-disabled={disabled}
        aria-label={label}
        disabled={disabled}
        className={classNames(
          "relative flex items-center border-none bg-transparent",
          {
            "cursor-pointer": !disabled,
            "cursor-not-allowed opacity-40": disabled,
          },
          small ? "h-6 w-9" : "h-8 w-12",
          className,
        )}
      >
        <div
          className={classNames(
            "w-full rounded-full transition-colors flex items-center px-0.5 absolute top-1/2 -translate-y-1/2",
            {
              "bg-sky-500": value,
              "bg-neutral-700": !value,
              "h-5": small,
              "h-7": !small,
            },
          )}
        >
          <div
            className={classNames(
              "rounded-full shadow-md transition-all duration-200",
              {
                "bg-white": value,
                "bg-neutral-400": !value,
                "h-4 w-4": small && value,
                "h-3 w-3": small && !value,
                "h-6 w-6": !small && value,
                "h-4.5 w-4.5": !small && !value,
              },
            )}
            style={{
              transform: `translateX(${value ? (small ? "16px" : "20px") : small ? "2px" : "2px"})`,
            }}
          />
        </div>
      </button>
    </div>
  );
};
