import MdiIcon from "@mdi/react";
import { IconProps } from "@mdi/react/dist/IconProps";
import classNames from "classnames";
import { FunctionComponent } from "preact";

type Variant = "standard" | "filled";
type Props = {
  variant: Variant;
  onClick: () => void;
};

const getTheme = (variant: Variant) => {
  switch (variant) {
    case "filled":
      return "flex items-center justify-center hover:bg-neutral-800 h-[36px] w-[36px] text-white rounded-lg";
    default:
      return "flex items-center justify-center h-[36px] w-[36px] text-white rounded-lg";
  }
};

export const IconButton: FunctionComponent<IconProps & Props> = ({
  variant,
  className,
  size = 1,
  onClick,
  ...props
}) => {
  return (
    <button className={classNames(getTheme(variant), className)}>
      <MdiIcon size={size} {...props} />
    </button>
  );
};
