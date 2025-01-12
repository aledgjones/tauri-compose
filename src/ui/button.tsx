import classNames from "classnames";
import { FunctionalComponent } from "preact";

type ButtonProps = {
  onClick: () => void;
  className?: string;
  label?: string;
  isDisabled?: boolean;
};

export const Button: FunctionalComponent<ButtonProps> = ({
  className,
  children,
  onClick,
  isDisabled,
}) => {
  return (
    <button
      onClick={onClick}
      className={classNames(
        "h-[36px] flex items-center text-left  text-white text-sm px-3 bg-neutral-900 hover:bg-neutral-800",
        className
      )}
    >
      <div className={classNames({ "opacity-25": isDisabled })}>{children}</div>
    </button>
  );
};
