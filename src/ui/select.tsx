import { mdiChevronDown } from "@mdi/js";
import Icon from "@mdi/react";
import classNames from "classnames";
import { useEffect, useRef, useState } from "preact/hooks";

export type SelectOption<T> = {
  value: T;
  label: string;
};

type SelectProps<T> = {
  value: T;
  options: SelectOption<T>[];
  onChange: (value: T) => void;
  direction?: "up" | "down";
  className?: string;
};

export const Select = <T,>({
  value,
  options,
  onChange,
  direction = "down",
  className,
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

  const handleOptionClick = (optionValue: T) => {
    onChange(optionValue);
    setIsOpen(false);
  };

  return (
    <div
      className={classNames("relative inline-block", className)}
      ref={selectRef}
    >
      <button
        onClick={() => setIsOpen((prev) => !prev)}
        className={classNames("pr-10 text-white text-sm px-4 py-2 rounded-lg", {
          "bg-neutral-900 hover:bg-neutral-800": !isOpen,
          "bg-neutral-800": isOpen,
        })}
      >
        {options.find((option) => option.value === value)?.label || "Select"}
        <div className="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
          <Icon
            path={mdiChevronDown}
            size={0.75}
            className={classNames(
              "text-neutral-300 transform transition-transform",
              { "rotate-180": isOpen }
            )}
          />
        </div>
      </button>

      {isOpen && (
        <ul
          className={classNames(
            "min-w-48 absolute z-10 bg-neutral-800 text-white text-sm rounded-lg shadow-lg mt-1 overflow-hidden",
            {
              "bottom-full mb-2": direction === "up",
              "top-full mt-2": direction === "down",
            }
          )}
        >
          {options.map((option) => {
            return (
              <li
                key={option.value}
                onClick={() => handleOptionClick(option.value)}
                className="px-4 py-2 hover:bg-neutral-700"
              >
                {option.label}
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
};
