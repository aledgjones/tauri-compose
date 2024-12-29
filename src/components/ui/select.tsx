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
};

export const Select = <T,>({
  value,
  options,
  onChange,
  direction = "down",
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
    <div className="relative inline-block" ref={selectRef}>
      <button
        onClick={() => setIsOpen((prev) => !prev)}
        className="pr-10 bg-neutral-800 text-white text-sm px-4 py-2 rounded-lg focus:outline-none"
      >
        {options.find((option) => option.value === value)?.label || "Select"}
        <div className="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
          <svg
            className={classNames(
              "w-4 h-4 text-neutral-300 transform transition-transform",
              { "rotate-180": isOpen }
            )}
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" />
          </svg>
        </div>
      </button>

      {isOpen && (
        <ul
          className={classNames(
            "min-w-48 absolute z-10 bg-neutral-700 text-white text-sm rounded-lg shadow-lg mt-1 overflow-hidden",
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
                className="px-4 py-2 hover:bg-neutral-600"
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
