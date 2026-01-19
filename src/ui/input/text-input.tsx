import { FC, useState, useEffect } from "react";
import { TextInputProps } from ".";
import classNames from "classnames";

export const TextInput: FC<TextInputProps> = ({
  value,
  onChange,
  className,
  label,
  placeholder,
  readOnly,
  onKeyDown,
  onBlur,
  onEnter,
  units,
}) => {
  const [isFocused, setIsFocused] = useState(false);
  const [internalValue, setInternalValue] = useState(value);

  useEffect(() => {
    setInternalValue(value);
  }, [value]);

  return (
    <div className={className}>
      {label && (
        <label
          className={classNames("block mb-0.5 text-xs", {
            "text-sky-500": isFocused,
            "text-neutral-400": !isFocused,
          })}
        >
          {label}
        </label>
      )}
      <div className="relative">
        <input
          placeholder={placeholder}
          readOnly={readOnly}
          type="text"
          value={internalValue}
          onChange={(e) => {
            setInternalValue(e.target.value);
            onChange(e.target.value);
          }}
          onFocus={() => setIsFocused(true)}
          onBlur={() => {
            setIsFocused(false);
            onBlur?.();
          }}
          className={classNames(
            "block w-full bg-neutral-800 px-2 py-1.5 text-sm focus:outline-hidden border-2 border-transparent focus:border-sky-500 text-white hover:bg-neutral-700 focus:bg-neutral-700",
            { "pr-2": units },
          )}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              onEnter?.();
            }

            onKeyDown?.(event);
          }}
        />
        {units && (
          <span className="absolute right-2 top-1/2 transform -translate-y-1/2 text-sm text-neutral-500 pointer-events-none">
            {units}
          </span>
        )}
      </div>
    </div>
  );
};
