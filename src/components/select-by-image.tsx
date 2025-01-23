import classNames from "classnames";
import { Key } from "react";

export type SelectByImageOption<T> = {
  value: T;
  label: string;
  image: string;
};

type SelectByImageProps<T> = {
  value: T;
  options: SelectByImageOption<T>[];
  onChange: (value: T) => void;
  className?: string;
  label?: string;
};

export const SelectByImage = <T extends Key>({
  value,
  options,
  onChange,
  className,
  label,
}: SelectByImageProps<T>) => {
  return (
    <div className={classNames("bg-neutral-800 p-2", className)}>
      <p className="text-white text-sm mb-2">{label}</p>
      <div className="flex gap-2">
        {options.map((option) => {
          const isSelected = value === option.value;
          return (
            <button
              className="w-[150px]"
              onClick={() => onChange(option.value)}
            >
              <img
                src={option.image}
                className={classNames("w-full mb-2 border-4", {
                  "border-white": !isSelected,
                  "border-sky-500": isSelected,
                })}
              />
              <p className="text-white text-xs">{option.label}</p>
            </button>
          );
        })}
      </div>
    </div>
  );
};
