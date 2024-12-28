import classNames from "classnames";
import { useEffect, useRef, useState } from "preact/hooks";

export type Option<T> = {
  value: T;
  label: string;
};

type Props<T> = {
  value: T;
  options: Option<T>[];
  onChange: (value: T) => void;
};

type Offset = { left: number; width: number };

export const Tabs = <T,>({ value, options, onChange }: Props<T>) => {
  const [tabOffsets, setTabOffsets] = useState<Offset[]>([]);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const tabRefs = useRef<(HTMLButtonElement | null)[]>([]);

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
    <div
      ref={containerRef}
      className="relative inline-block bg-stone-800 rounded-lg whitespace-nowrap"
    >
      <ul className="relative z-10">
        {options.map((option, index) => {
          const isSelected = value === option.value;

          return (
            <li key={option.value} className="inline-block">
              <button
                ref={(el) => (tabRefs.current[index] = el)}
                onClick={() => onChange(option.value)}
                className={classNames(
                  "py-2 px-4 text-sm transition-colors duration-300",
                  {
                    "text-white": isSelected,
                    "text-stone-400": !isSelected,
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
          className="absolute top-0 left-0 h-full bg-stone-600 transition-all duration-300 rounded-lg"
          style={{
            width: tabOffsets[selectedIndex]?.width || 0,
            left: tabOffsets[selectedIndex]?.left || 0,
          }}
        />
      )}
    </div>
  );
};
