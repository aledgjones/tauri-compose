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
      return "bg-neutral-900 hover:bg-neutral-800";
    default:
      return "";
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
      onClick={onClick}
      className={classNames(
        "flex items-center justify-center h-[36px] w-[36px]",
        getTheme(variant),
        "rounded-lg",
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
