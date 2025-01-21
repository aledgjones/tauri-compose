import classNames from "classnames";
import { Key, useEffect, useRef, useState } from "react";

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

type Offset = { left: number; width: number };

export const Tabs = <T extends Key>({
  value,
  options,
  onChange,
  className,
}: TabsProps<T>) => {
  const [tabOffsets, setTabOffsets] = useState<Offset[]>([]);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const tabRefs = useRef<HTMLButtonElement[]>([]);

  useEffect(() => {
    const container = containerRef.current?.getBoundingClientRect();

    const offsets = tabRefs.current.map((ref) => {
      if (ref) {
        const rect = ref.getBoundingClientRect();
        return { left: rect.left - (container?.left || 0), width: rect.width };
      }
      return { left: 0, width: 0 };
    });
    setTabOffsets(offsets);
  }, [options]);

  const selectedIndex = options.findIndex((option) => option.value === value);

  return (
    <div className={className}>
      <div
        ref={containerRef}
        className="relative whitespace-nowrap bg-neutral-900"
      >
        <ul className="relative z-10 flex">
          {options.map((option, index) => {
            const isSelected = value === option.value;

            return (
              <li key={option.value}>
                <button
                  ref={(el) => {
                    if (el) {
                      tabRefs.current[index] = el;
                    }
                  }}
                  onClick={() => onChange(option.value)}
                  className={classNames(
                    "py-2 px-4 text-sm transition-[color] duration-300 ",
                    {
                      "text-black": isSelected,
                      "text-white hover:bg-neutral-800": !isSelected,
                    }
                  )}
                >
                  {option.label}
                </button>
              </li>
            );
          })}
        </ul>
        {tabOffsets.length > 0 && (
          <div
            className="absolute top-0 left-0 h-full bg-sky-500 transition-all duration-300 "
            style={{
              width: tabOffsets[selectedIndex]?.width || 0,
              left: tabOffsets[selectedIndex]?.left || 0,
            }}
          />
        )}
      </div>
    </div>
  );
};
