import MdiIcon from "@mdi/react";
import { IconProps } from "@mdi/react/dist/IconProps";
import classNames from "classnames";
import { FunctionComponent } from "preact";

type Props = {
  isDisabled?: boolean;
};

export const Icon: FunctionComponent<IconProps & Props> = ({
  className,
  size = 1,
  isDisabled = false,
  ...props
}) => {
  return (
    <div
      className={classNames(
        {
          "text-white": !isDisabled,
          "pointer-events-none text-neutral-600": isDisabled,
        },
        className
      )}
    >
      <MdiIcon size={size} {...props} />
    </div>
  );
};
