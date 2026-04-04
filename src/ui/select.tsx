import { mdiChevronDown } from "@mdi/js";
import { Icon as MdiIcon } from "@mdi/react";
import classNames from "classnames";
import { Key, ReactNode, useEffect, useRef, useState } from "react";
import { TokenizedText } from "./text";

export type SelectOption<T> = {
  value: T;
  label: string;
  disabled?: boolean;
  endAdornment?: ReactNode;
};

type SelectProps<T> = {
  value: T | null;
  options: SelectOption<T>[];
  onChange: (value: T) => void;
  direction?: "up" | "down";
  className?: string;
  variant: "standard" | "filled";
  label?: string;
  placeholder?: string;
  disabled?: boolean;
  icon?: string;
  iconSize?: number;
  endAdornment?: ReactNode;
};

export const Select = <T extends Key>({
  value,
  options,
  onChange,
  direction = "down",
  className,
  variant,
  label,
  placeholder,
  disabled,
  icon,
  iconSize = 1,
  endAdornment,
}: SelectProps<T>) => {
  const [isOpen, setIsOpen] = useState(false);
  const selectRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        selectRef.current &&
        !selectRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  const handleOptionClick = (optionValue: T, optionDisabled?: boolean) => {
    if (optionDisabled) {
      return;
    }

    onChange(optionValue);
    setIsOpen(false);
  };

  const isStandard = variant === "standard";

  const selection = options.find((option) => option.value === value);

  return (
    <div className={classNames("relative", className)} ref={selectRef}>
      {label && (
        <label
          className={classNames("block mb-0.5 text-xs", {
            "text-sky-500": isOpen,
            "text-neutral-400": !isOpen,
          })}
        >
          {label}
        </label>
      )}
      <button
        disabled={disabled}
        onClick={() => setIsOpen((prev) => !prev)}
        className={classNames(
          "h-9 flex items-center w-full min-w-0 text-left text-white text-sm px-3 border-2",
          {
            "bg-neutral-900 hover:bg-neutral-800 border-transparent":
              !isOpen && isStandard,
            "bg-neutral-800 border-transparent": isOpen && isStandard,
            "bg-neutral-800 hover:bg-neutral-700 border-transparent":
              !isOpen && !isStandard,
            "bg-neutral-700 border-sky-500": isOpen && !isStandard,
          },
        )}
      >
        {icon && (
          <div className="flex items-center pr-2 pointer-events-none">
            <MdiIcon path={icon} size={iconSize} className="text-neutral-300" />
          </div>
        )}
        <span
          className={classNames("flex-1 min-w-0 whitespace-nowrap", {
            "text-white": !disabled,
            "text-neutral-500 italic": disabled,
          })}
        >
          <span className="block overflow-hidden text-ellipsis whitespace-nowrap">
            <TokenizedText>
              {selection?.label || placeholder || "Select"}
            </TokenizedText>
          </span>
        </span>

        {endAdornment ? (
          <div className="ml-2 flex shrink-0 items-center">{endAdornment}</div>
        ) : null}

        <div className="flex items-center pointer-events-none pl-3">
          <MdiIcon
            path={mdiChevronDown}
            size={0.75}
            className={classNames(
              "text-neutral-300 transform transition-transform",
              { "rotate-180": isOpen },
            )}
          />
        </div>
      </button>

      {isOpen && (
        <ul
          className={classNames(
            "whitespace-nowrap py-1 min-w-full absolute z-50 text-white text-sm shadow-lg overflow-auto max-h-60.5",
            {
              "bottom-full mb-0.5": direction === "up",
              "top-full mt-0.5": direction === "down",
              "bg-neutral-800": isStandard,
              "bg-neutral-700": !isStandard,
            },
          )}
        >
          {options.map((option) => {
            return (
              <li key={option.value}>
                <button
                  disabled={option.disabled}
                  onClick={() =>
                    handleOptionClick(option.value, option.disabled)
                  }
                  role="option"
                  aria-selected={option.value === value}
                  className={classNames(
                    "flex w-full items-center gap-2 px-3 py-2 text-left",
                    {
                      "cursor-pointer hover:bg-neutral-700":
                        isStandard && !option.disabled,
                      "cursor-pointer hover:bg-neutral-600":
                        !isStandard && !option.disabled,
                      "cursor-not-allowed text-neutral-500": option.disabled,
                    },
                  )}
                >
                  <span className="min-w-0 flex-1 overflow-hidden text-ellipsis whitespace-nowrap">
                    <TokenizedText>{option.label}</TokenizedText>
                  </span>
                  {option.endAdornment ? (
                    <span className="shrink-0">{option.endAdornment}</span>
                  ) : null}
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
};
