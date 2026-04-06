import {
  FC,
  KeyboardEvent,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";
import { NumberInputProps } from ".";
import { TextInput } from "./text-input";
import Big from "big.js";

export const NumberInput: FC<NumberInputProps> = ({
  value,
  onChange,
  defaultValue,
  className,
  label,
  ariaLabel,
  readOnly,
  disabled,
  step,
  decimalPlaces,
  units,
  leadingIcon,
  variant,
}) => {
  const toValue = useCallback((value: string | undefined) => {
    if (value === undefined || value === "") {
      return undefined;
    } else {
      return parseFloat(value);
    }
  }, []);

  const toPrecision = useCallback(
    (value: number | string | undefined) => {
      if (value === undefined || value === "") {
        return "";
      } else {
        return new Big(value).toFixed(decimalPlaces);
      }
    },
    [decimalPlaces],
  );

  // Derive the display value from the value prop
  const derivedDisplay = useMemo(
    () => toPrecision(value),
    [value, toPrecision],
  );

  // Local state for user input (only when actively editing)
  const [display, setDisplay] = useState<string>(derivedDisplay);

  const fallbackValue = useMemo(() => {
    if (defaultValue !== undefined) {
      return defaultValue;
    }

    if (value !== undefined) {
      return value;
    }

    return 0;
  }, [defaultValue, value]);

  // Sync display with derived value when value prop changes
  useEffect(() => {
    setDisplay(derivedDisplay);
  }, [derivedDisplay]);

  // apply valid number, or revert
  const handleBlur = useCallback(() => {
    try {
      const val = toPrecision(display);
      const parsed = toValue(val) ?? fallbackValue;
      const formatted = toPrecision(parsed);
      setDisplay(formatted);
      onChange(parsed);
    } catch {
      setDisplay(derivedDisplay);
    }
  }, [toPrecision, display, toValue, onChange, derivedDisplay, fallbackValue]);

  const onIncrease = useCallback(() => {
    if (value === undefined) {
      return;
    }

    const val = new Big(value).plus(step).toFixed(decimalPlaces);
    const parsed = parseFloat(val);
    onChange(parsed);
  }, [value, step, decimalPlaces, onChange]);

  const onDecrease = useCallback(() => {
    if (value === undefined) {
      return;
    }

    const val = new Big(value).minus(step).toFixed(decimalPlaces);
    const parsed = parseFloat(val);
    onChange(parsed);
  }, [value, step, decimalPlaces, onChange]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLInputElement>) => {
      switch (e.key) {
        case "ArrowUp":
          e.preventDefault();
          onIncrease();
          break;
        case "ArrowDown":
          e.preventDefault();
          onDecrease();
          break;
        default:
          break;
      }
    },
    [onIncrease, onDecrease],
  );

  return (
    <TextInput
      type="text"
      value={display}
      onChange={setDisplay}
      useLocalDraftWhileFocused={false}
      className={className}
      label={label}
      ariaLabel={ariaLabel}
      readOnly={readOnly}
      disabled={disabled}
      onKeyDown={handleKeyDown}
      onBlur={handleBlur}
      units={units}
      leadingIcon={leadingIcon}
      variant={variant}
    />
  );
};
