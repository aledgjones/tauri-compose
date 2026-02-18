import classNames from "classnames";
import { FC } from "react";
import { getTheme, Variant } from "./shared";

type Props = {
  variant?: Variant;
  bravura: string;
  size?: number; // px
  offset?: number;
  title?: string;
  isDisabled?: boolean;
  isSelected?: boolean;
  onClick: () => void;
  className?: string;
};

// Export only the component so Fast Refresh keeps working reliably.
export const BravuraIconButton: FC<Props> = ({
  variant = "interactive",
  bravura,
  size = 36,
  offset = 0,
  title,
  isDisabled = false,
  isSelected,
  onClick,
  className,
}) => {
  return (
    <button
      title={title}
      onClick={onClick}
      className={classNames(
        "flex items-center justify-center h-9 w-9",
        getTheme(variant, isSelected),
        { "pointer-events-none": isDisabled },
        className,
      )}
    >
      <div className={classNames({ "opacity-25": isDisabled })}>
        <span
          className="font-bravura leading-3 align-middle pointer-events-none relative"
          style={{ fontSize: `${size}px`, top: `${offset}px` }}
        >
          {bravura}
        </span>
      </div>
    </button>
  );
};
