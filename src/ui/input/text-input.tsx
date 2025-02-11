import { FC, useState } from "react";
import { TextInputProps } from ".";
import classNames from "classnames";

export const TextInput: FC<TextInputProps> = ({
  value,
  onChange,
  className,
  label,
  readOnly,
  onKeyDown,
  onBlur,
  onEnter,
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
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onFocus={() => setIsFocused(true)}
        onBlur={() => {
          setIsFocused(false);
          onBlur?.();
        }}
        className="block w-full bg-neutral-800 px-2 py-[6px] text-sm focus:outline-hidden border-2 border-neutral-800 focus:border-sky-500 text-white"
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            onEnter?.();
          }

          onKeyDown?.(event);
        }}
      />
    </div>
  );
};
