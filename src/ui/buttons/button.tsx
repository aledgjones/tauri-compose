import classNames from "classnames";
import { FC, PropsWithChildren } from "react";

type ButtonProps = PropsWithChildren<{
  onClick: () => void;
  className?: string;
  label?: string;
  isDisabled?: boolean;
}>;

export const Button: FC<ButtonProps> = ({
  className,
  children,
  onClick,
  isDisabled,
}) => {
  return (
    <button
      onClick={onClick}
      className={classNames(
        "h-9 flex items-center text-left text-white text-sm px-3 bg-neutral-900 hover:bg-neutral-800",
        className,
      )}
    >
      <div className={classNames({ "opacity-25": isDisabled })}>{children}</div>
    </button>
  );
};
