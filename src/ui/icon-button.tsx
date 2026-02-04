import MdiIcon from "@mdi/react";
import { IconProps } from "@mdi/react/dist/IconProps";
import classNames from "classnames";
import { FC } from "react";

type Variant =
  | "standard"
  | "interactive"
  | "filled"
  | "filled-dark"
  | "outline";
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
      return "bg-neutral-800 text-white hover:bg-neutral-700";
    case "filled-dark":
      return "bg-neutral-950 text-white hover:bg-neutral-800";
    case "interactive":
      return "text-white bg-neutral-900 hover:bg-neutral-800";
    case "outline":
      return "border border-neutral-600 text-white hover:bg-neutral-800";
    default:
      return "";
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
