import MdiIcon from "@mdi/react";
import { IconProps } from "@mdi/react/dist/IconProps";
import { FunctionComponent } from "preact";

type Props = {};

export const Icon: FunctionComponent<IconProps & Props> = ({
  className,
  size = 1,
  ...props
}) => {
  return (
    <div className={className}>
      <MdiIcon size={size} {...props} />
    </div>
  );
};
