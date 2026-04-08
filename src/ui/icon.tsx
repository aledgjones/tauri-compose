import { Icon as MdiIcon } from "@mdi/react";
import { IconProps } from "@mdi/react/dist/IconProps";
import { FC } from "react";

export const Icon: FC<IconProps> = ({ className, size = 1, ...props }) => {
  return (
    <div className={className}>
      <MdiIcon size={size} {...props} />
    </div>
  );
};
