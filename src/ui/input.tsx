import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { useState } from "preact/hooks";

interface InputProps {
  value: string;
  onChange?: (value: string) => void;
  className?: string;
  label: string;
  readOnly?: boolean;
}

export const Input: FunctionalComponent<InputProps> = ({ value, onChange, className, label, readOnly }) => {
  const [isFocused, setIsFocused] = useState(false);

  return (
    <div className={className}>
      <label
        className={classNames("text-xs", {
          "text-sky-500": isFocused,
          "text-neutral-400": !isFocused,
        })}
      >
        {label}
      </label>
      <input
        readOnly={readOnly}
        type="text"
        value={value}
        onInput={(e: any) => onChange?.(e.target.value)}
        onFocus={() => setIsFocused(true)}
        onBlur={() => setIsFocused(false)}
        className=" block w-full bg-neutral-800 px-2 py-[6px] text-sm focus:outline-none border-transparent border-2 border-neutral-800 focus:border-sky-500 text-white"
      />
    </div>
  );
};
