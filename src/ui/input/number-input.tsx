import { FC, KeyboardEvent, useCallback, useEffect, useState } from "react";
import { NumberInputProps } from ".";
import { TextInput } from "./text-input";
import Big from "big.js";

export const NumberInput: FC<NumberInputProps> = ({
  value,
  onChange,
  className,
  label,
  readOnly,
  step,
  decimalPlaces,
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
    [decimalPlaces]
  );

  const [display, setDisplay] = useState<string>(toPrecision(value));

  useEffect(() => setDisplay(toPrecision(value)), [value, toPrecision]);

  // apply valid number, or revert
  const handleBlur = useCallback(() => {
    try {
      const val = toPrecision(display);
      const parsed = toValue(val)!;
      setDisplay(val);
      onChange(parsed);
    } catch (e) {
      setDisplay(toPrecision(value));
    }
  }, [toPrecision, display, toValue, onChange, value]);

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
    [onIncrease, onDecrease]
  );

  return (
    <TextInput
      type="text"
      value={display}
      onChange={setDisplay}
      className={className}
      label={label}
      readOnly={readOnly}
      onKeyDown={handleKeyDown}
      onBlur={handleBlur}
    />
  );
};
