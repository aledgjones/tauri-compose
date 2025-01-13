import { FunctionalComponent } from "preact";
import { mdiCheck } from "@mdi/js";
import { Icon } from "./icon";
import classNames from "classnames";

type Props = {
  value: boolean;
  onChange: (value: boolean) => void;
  className?: string;
  onPointerDown?: (event: PointerEvent) => void;
};

export const Checkbox: FunctionalComponent<Props> = ({
  value,
  onChange,
  className,
  onPointerDown,
}) => {
  const handleChange = (event: Event) => {
    onChange((event.target as HTMLInputElement).checked);
  };

  return (
    <div
      onClick={handleChange}
      onPointerDown={onPointerDown}
      className={`relative h-[36px] w-[36px] flex items-center justify-center ${className}`}
    >
      <div
        className={classNames("border-2 h-5 w-5", {
          "bg-sky-500 border-sky-500": value,
          "border-neutral-600": !value,
        })}
      >
        {value && (
          <Icon
            className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
            path={mdiCheck}
            size={0.8}
            color="white"
          />
        )}
      </div>
    </div>
  );
};
