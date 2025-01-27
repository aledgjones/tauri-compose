import MdiIcon from "@mdi/react";
import { IconProps } from "@mdi/react/dist/IconProps";
import classNames from "classnames";
import { FC } from "react";

type Variant = "standard" | "filled";
type Props = {
  variant: Variant;
  onClick: () => void;
  isDisabled?: boolean;
  isSelected?: boolean;
};

const getTheme = (variant: Variant, isSelected?: boolean) => {
  if (isSelected) {
    return "bg-sky-500 text-black";
  }

  switch (variant) {
    case "filled":
      return "bg-neutral-800 hover:bg-neutral-700";
    default:
      return "hover:bg-neutral-800";
  }
};

export const IconButton: FC<IconProps & Props> = ({
  variant,
  className,
  size = 1,
  onClick,
  isDisabled = false,
  isSelected,
  ...props
}) => {
  return (
    <button
      onClick={onClick}
      className={classNames(
        "flex items-center justify-center h-[36px] w-[36px]",
        getTheme(variant, isSelected),
        { "pointer-events-none": isDisabled },
        className
      )}
    >
      <div className={classNames({ "opacity-25": isDisabled })}>
        <MdiIcon size={size} {...props} />
      </div>
    </button>
  );
};
