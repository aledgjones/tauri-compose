import classNames from "classnames";
import { Key } from "react";

type MenuSection<T> = {
  label: string;
  value: T;
};

export type MenuItem<T> = {
  label: string;
  value: T;
  sections?: MenuSection<T>[];
};

type Props<T> = {
  items: MenuItem<T>[];
  selection: T;
  onSelect: (value: T) => void;
  className?: string;
};

export const Menu = <T extends Key>({
  items,
  selection,
  onSelect,
  className,
}: Props<T>) => {
  return (
    <div className={classNames("p-2", className)}>
      {items.map((item) => {
        const isSelected = item.value === selection;

        return (
          <div key={item.value}>
            <button
              onClick={() => onSelect(item.value)}
              className={classNames(
                "border-l-4 w-full text-left text-sm p-2 text-white",
                {
                  "bg-neutral-800 hover:bg-neutral-700 border-transparent":
                    !isSelected,
                  "bg-neutral-700 border-sky-500": isSelected,
                },
              )}
            >
              {item.label}
            </button>
            {isSelected && item.sections && (
              <div className="py-1">
                {item.sections.map((section) => (
                  <button
                    key={section.value}
                    onClick={() => {}}
                    className="border-l-4 border-transparent w-full text-white text-sm text-left pl-2 pr-2 py-2"
                  >
                    {section.label}
                  </button>
                ))}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
};
