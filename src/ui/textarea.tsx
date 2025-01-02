import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { useRef, useState } from "preact/hooks";

interface TextareaProps {
  value: string;
  onChange?: (value: string) => void;
  className?: string;
  label: string;
  readOnly?: boolean;
}

export const Textarea: FunctionalComponent<TextareaProps> = ({
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
          "text-neutral-400": !isFocused,
          "text-sky-500": isFocused,
        })}
      >
        {label}
      </label>
      <textarea
        value={value}
        contentEditable={!readOnly}
        onInput={(e: any) => {
          e.target.style.height = "1px";
          e.target.style.height = e.target.scrollHeight + 4 + "px";
          onChange?.(e.target.value);
        }}
        onFocus={() => setIsFocused(true)}
        onBlur={() => setIsFocused(false)}
        className="block w-full h-[36px] bg-neutral-800 px-2 py-[6px] text-sm focus:outline-none border-transparent border-2 border-neutral-800 focus:border-sky-500 text-white resize-none"
      />
    </div>
  );
};
