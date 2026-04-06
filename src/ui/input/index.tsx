import { FC, KeyboardEvent } from "react";
import { NumberInput } from "./number-input";
import { TextInput } from "./text-input";

interface BaseInputProps {
  className?: string;
  label?: string;
  ariaLabel?: string;
  readOnly?: boolean;
  disabled?: boolean;
  placeholder?: string;
  leadingIcon?: string;
  variant?: "default" | "dark";
}

export interface TextInputProps extends BaseInputProps {
  type: "text";
  value: string;
  onChange: (value: string) => void;
  useLocalDraftWhileFocused?: boolean;
  onKeyDown?: (event: KeyboardEvent<HTMLInputElement>) => void;
  onBlur?: () => void;
  onEnter?: () => void;
  units?: string;
}

export interface NumberInputProps extends BaseInputProps {
  type: "number";
  value?: number;
  onChange: (value: number) => void;
  defaultValue?: number;
  step: number;
  decimalPlaces: number;
  min?: number;
  max?: number;
  units?: string;
}

type InputProps = TextInputProps | NumberInputProps;

export const Input: FC<InputProps> = (props) => {
  if (props.type === "number") {
    return <NumberInput {...props} />;
  }

  return <TextInput {...props} />;
};
