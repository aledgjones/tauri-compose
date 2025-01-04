import classNames from "classnames";
import { FunctionalComponent } from "preact";

type Props = {
  title: string;
  className?: string;
};

export const Panel: FunctionalComponent<Props> = ({ title, className }) => {
  return (
    <div
      className={classNames(
        "rounded-lg bg-neutral-900 overflow-hidden h-full",
        className
      )}
    >
      <div className="h-[48px] px-4 flex items-center">
        <p className="text-white text-sm">{title}</p>
      </div>
    </div>
  );
};
