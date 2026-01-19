import { FC, KeyboardEvent } from "react";
import { NumberInput } from "./number-input";
import { TextInput } from "./text-input";

interface BaseInputProps {
  className?: string;
  label?: string;
  readOnly?: boolean;
  placeholder?: string;
}

export interface TextInputProps extends BaseInputProps {
  type: "text";
  value: string;
  onChange: (value: string) => void;
  onKeyDown?: (event: KeyboardEvent<HTMLInputElement>) => void;
  onBlur?: () => void;
  onEnter?: () => void;
  units?: string;
}

export interface NumberInputProps extends BaseInputProps {
  type: "number";
  value: number;
  onChange: (value: number) => void;
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
