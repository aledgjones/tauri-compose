import classNames from "classnames";

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

export const Menu = <T,>({
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
          <div className="pb-1" key={item.value}>
            <button
              onClick={() => onSelect(item.value)}
              className={classNames("rounded-lg w-full text-left text-sm p-2", {
                "bg-neutral-800 hover:bg-neutral-700 text-white": !isSelected,
                "bg-sky-500 text-black": isSelected,
              })}
            >
              {item.label}
            </button>
            <div>
              {isSelected &&
                item.sections?.map((section) => (
                  <button
                    key={section.value}
                    onClick={() => {}}
                    className="rounded-lg w-full text-white text-sm text-left pl-6 pr-2 py-2 bg-neutral-800 hover:bg-neutral-700"
                  >
                    {section.label}
                  </button>
                ))}
            </div>
          </div>
        );
      })}
    </div>
  );
};
