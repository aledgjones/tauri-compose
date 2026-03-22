import MdiIcon from "@mdi/react";
import { IconProps } from "@mdi/react/dist/IconProps";
import classNames from "classnames";
import { FC, PointerEvent } from "react";
import { getTheme, Variant } from "./shared";

type Props = {
  variant: Variant;
  onClick: () => void;
  onPointerDown?: (e: PointerEvent<HTMLButtonElement>) => void;
  isDisabled?: boolean;
  isSelected?: boolean;
};

// NOTE: this file must *only* export the component so Fast Refresh stays reliable.
export const IconButton: FC<IconProps & Props> = ({
  variant,
  className,
  size = 1,
  onClick,
  onPointerDown,
  isDisabled = false,
  isSelected,
  ...props
}) => {
  return (
    <button
      type="button"
      onClick={onClick}
      onPointerDown={onPointerDown}
      disabled={isDisabled}
      aria-disabled={isDisabled}
      className={classNames(
        "flex items-center justify-center h-9 w-9",
        getTheme(variant, isSelected),
        { "pointer-events-none": isDisabled },
        className,
      )}
    >
      <div className={classNames({ "opacity-25": isDisabled })}>
        <MdiIcon size={size} {...props} />
      </div>
    </button>
  );
};
