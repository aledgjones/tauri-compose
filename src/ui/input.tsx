import classNames from "classnames";
import { FC, useState } from "react";

interface InputProps {
  value: string;
  onChange?: (value: string) => void;
  className?: string;
  label: string;
  readOnly?: boolean;
}

export const Input: FC<InputProps> = ({
  value,
  onChange,
  className,
  label,
  readOnly,
}) => {
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
        defaultValue={value}
        onChange={(e: any) => onChange?.(e.target.value)}
        onFocus={() => setIsFocused(true)}
        onBlur={() => setIsFocused(false)}
        className=" block w-full bg-neutral-800 px-2 py-[6px] text-sm focus:outline-hidden border-2 border-neutral-800 focus:border-sky-500 text-white"
      />
    </div>
  );
};
