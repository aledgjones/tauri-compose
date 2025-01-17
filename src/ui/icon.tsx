import MdiIcon from "@mdi/react";
import { IconProps } from "@mdi/react/dist/IconProps";
import { FunctionComponent } from "preact";

export const Icon: FunctionComponent<IconProps> = ({ className, size = 1, ...props }) => {
  return (
    <div className={className}>
      <MdiIcon size={size} {...props} />
    </div>
  );
};
