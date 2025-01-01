import MdiIcon from "@mdi/react";
import { IconProps } from "@mdi/react/dist/IconProps";
import classNames from "classnames";
import { FunctionComponent } from "preact";

type Variant = "standard" | "filled";
type Props = {
  variant: Variant;
  onClick: () => void;
  isDisabled?: boolean;
};

const getTheme = (variant: Variant) => {
  switch (variant) {
    case "filled":
      return "flex items-center justify-center bg-neutral-900 hover:bg-neutral-800 h-[36px] w-[36px] rounded-lg";
    default:
      return "flex items-center justify-center h-[36px] w-[36px] rounded-lg";
  }
};

export const IconButton: FunctionComponent<IconProps & Props> = ({
  variant,
  className,
  size = 1,
  onClick,
  isDisabled = false,
  ...props
}) => {
  return (
    <button
      className={classNames(
        getTheme(variant),
        {
          "text-white": !isDisabled,
          "pointer-events-none text-neutral-600": isDisabled,
        },
        className
      )}
    >
      <MdiIcon size={size} {...props} />
    </button>
  );
};
