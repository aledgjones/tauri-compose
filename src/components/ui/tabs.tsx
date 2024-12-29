import classNames from "classnames";
import { useEffect, useRef, useState } from "preact/hooks";

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

export const Tabs = <T,>({
  value,
  options,
  onChange,
  className,
}: TabsProps<T>) => {
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
      className={classNames(
        "relative inline-block bg-neutral-800 rounded-lg whitespace-nowrap",
        className
      )}
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
                  "py-2 px-4 text-sm transition-[color] duration-300 rounded-lg",
                  {
                    "text-black": isSelected,
                    "text-white": !isSelected,
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
          className="absolute top-0 left-0 h-full bg-sky-500 transition-all duration-300 rounded-lg"
          style={{
            width: tabOffsets[selectedIndex]?.width || 0,
            left: tabOffsets[selectedIndex]?.left || 0,
          }}
        />
      )}
    </div>
  );
};
