import classNames from "classnames";
import { Key } from "react";

export type TabsOption<T> = {
  value: T;
  label: string;
};

type TabsProps<T> = {
  value: T;
  options: TabsOption<T>[];
  onChange: (value: T) => void;
  className?: string;
};

export const Tabs = <T extends Key>({
  value,
  options,
  onChange,
  className,
}: TabsProps<T>) => {
  return (
    <div className={className}>
      <div className="relative whitespace-nowrap">
        <ul className="relative z-10 flex gap-0.5">
          {options.map((option) => {
            const isSelected = value === option.value;

            return (
              <li key={option.value}>
                <button
                  onClick={() => onChange(option.value)}
                  className={classNames("py-2 px-4 text-sm", {
                    "text-black bg-sky-500": isSelected,
                    "text-white bg-neutral-900 hover:bg-neutral-800":
                      !isSelected,
                  })}
                >
                  {option.label}
                </button>
              </li>
            );
          })}
        </ul>
      </div>
    </div>
  );
};
