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
    <div className={classNames("flex flex-col gap-0.5", className)}>
      {items.map((item) => {
        const isSelected = item.value === selection;

        return (
          <div className="gap-0.5 flex flex-col" key={item.value}>
            <button
              onClick={() => onSelect(item.value)}
              className={classNames(
                "border-l-4 w-full text-left text-sm p-2 pl-3 text-white",
                {
                  "bg-neutral-900 hover:bg-neutral-800 border-transparent":
                    !isSelected,
                  "bg-neutral-800 border-sky-500": isSelected,
                },
              )}
            >
              {item.label}
            </button>
            {isSelected && item.sections && (
              <>
                {item.sections.map((section) => (
                  <button
                    key={section.value}
                    onClick={() => {}}
                    className="border-l-4 border-sky-950 bg-neutral-950 w-full text-white text-sm text-left pl-3 pr-2 py-2"
                  >
                    {section.label}
                  </button>
                ))}
              </>
            )}
          </div>
        );
      })}
    </div>
  );
};
