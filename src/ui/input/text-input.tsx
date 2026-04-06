import { FC, useState } from "react";
import { TextInputProps } from ".";
import classNames from "classnames";
import { Icon } from "../../ui/icon";

export const TextInput: FC<TextInputProps> = ({
  value,
  onChange,
  useLocalDraftWhileFocused = true,
  className,
  label,
  ariaLabel,
  placeholder,
  readOnly,
  disabled,
  onKeyDown,
  onBlur,
  onEnter,
  units,
  leadingIcon,
  variant = "default",
}) => {
  const [isFocused, setIsFocused] = useState(false);
  const [draftValue, setDraftValue] = useState(value);
  const displayValue =
    useLocalDraftWhileFocused && isFocused ? draftValue : value;

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
          aria-label={ariaLabel}
          placeholder={placeholder}
          readOnly={readOnly}
          disabled={disabled}
          type="text"
          value={displayValue}
          onChange={(e) => {
            const nextValue = e.target.value;
            setDraftValue(nextValue);
            onChange(nextValue);
          }}
          onFocus={() => {
            setDraftValue(value);
            setIsFocused(true);
          }}
          onBlur={() => {
            setIsFocused(false);
            onBlur?.();
          }}
          className={classNames(
            "block w-full px-2 py-1.5 text-sm focus:outline-hidden border-2 border-transparent focus:border-sky-500 text-white",
            {
              "bg-neutral-800 hover:bg-neutral-700 focus:bg-neutral-700":
                variant === "default",
              "bg-neutral-900 hover:bg-neutral-800 focus:bg-neutral-800":
                variant === "dark",
            },
            {
              "cursor-not-allowed opacity-50 hover:bg-neutral-900 focus:bg-neutral-900":
                disabled && variant === "dark",
              "cursor-not-allowed opacity-50 hover:bg-neutral-800 focus:bg-neutral-800":
                disabled && variant === "default",
            },
            { "pr-2": units },
            { "pl-9": leadingIcon },
          )}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              onEnter?.();
            }

            onKeyDown?.(event);
          }}
        />
        {leadingIcon && (
          <Icon
            path={leadingIcon}
            className="absolute left-2 top-1/2 transform -translate-y-1/2 text-white pointer-events-none"
          />
        )}
        {units && (
          <span className="absolute right-2 top-1/2 transform -translate-y-1/2 text-sm text-neutral-500 pointer-events-none">
            {units}
          </span>
        )}
      </div>
    </div>
  );
};
