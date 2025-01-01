import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { useState } from "preact/hooks";

interface InputProps {
  value: string;
  onChange: (value: string) => void;
  className?: string;
  label: string;
}

export const Input: FunctionalComponent<InputProps> = ({
  value,
  onChange,
  className,
  label,
}) => {
  const [isFocused, setIsFocused] = useState(false);

  return (
    <div className={classNames("relative", className)}>
      <label
        className={classNames("text-xs", {
          "text-neutral-500": !isFocused,
          "text-sky-500": isFocused,
        })}
      >
        {label}
      </label>
      <input
        type="text"
        value={value}
        onInput={(e: any) => onChange(e.target.value)}
        onFocus={() => setIsFocused(true)}
        onBlur={() => setIsFocused(false)}
        className="block w-full bg-neutral-800 rounded px-3 py-[6px] text-sm focus:outline-none border-transparent border-2 border-neutral-800 focus:border-sky-500 text-white"
      />
    </div>
  );
};
