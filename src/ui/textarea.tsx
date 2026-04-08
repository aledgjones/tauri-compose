import classNames from "classnames";
import { FC, InputEvent, useRef, useState } from "react";

interface TextareaProps {
  value: string;
  onChange?: (value: string) => void;
  className?: string;
  label: string;
  readOnly?: boolean;
  lines: number;
}

export const Textarea: FC<TextareaProps> = ({
  value,
  onChange,
  className,
  label,
  readOnly,
  lines,
}) => {
  const [isFocused, setIsFocused] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [draftValue, setDraftValue] = useState(value);
  const displayValue = isFocused ? draftValue : value;

  return (
    <div className={className}>
      <label
        className={classNames("text-xs", {
          "text-neutral-400": !isFocused,
          "text-sky-500": isFocused,
        })}
      >
        {label}
      </label>
      <textarea
        ref={textareaRef}
        value={displayValue}
        contentEditable={!readOnly}
        onInput={(e: InputEvent<HTMLTextAreaElement>) => {
          const target = e.target as HTMLTextAreaElement;
          setDraftValue(target.value);
          target.style.height = "1px";
          target.style.height = target.scrollHeight + 4 + "px";
          onChange?.(target.value);
        }}
        onFocus={() => {
          setDraftValue(value);
          setIsFocused(true);
        }}
        onBlur={() => setIsFocused(false)}
        className="block w-full h-9 bg-neutral-800 hover:bg-neutral-700 focus:bg-neutral-700 px-2 py-1.5 text-sm focus:outline-hidden border-2 border-neutral-800 focus:border-sky-500 text-white resize-none"
        style={{ minHeight: `${lines * 20 + 16}px` }}
      />
    </div>
  );
};
