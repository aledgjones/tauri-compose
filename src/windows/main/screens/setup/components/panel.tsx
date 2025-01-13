import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { IconButton } from "../../../../../ui/icon-button";

type IconConfig = {
  key: string;
  onClick: () => void;
  path: string;
};

type Props = {
  title: string;
  className?: string;
  icons?: IconConfig[];
};

export const Panel: FunctionalComponent<Props> = ({
  title,
  className,
  icons,
  children,
}) => {
  return (
    <div
      className={classNames(
        " bg-neutral-900 overflow-hidden h-full flex flex-col",
        className
      )}
    >
      <div className="py-2 pl-4 pr-2 flex items-center">
        <p className="text-white text-sm flex-1">{title}</p>
        {icons?.map((config) => {
          return (
            <IconButton
              key={config.key}
              className="text-white"
              onClick={config.onClick}
              variant="standard"
              path={config.path}
            />
          );
        })}
      </div>
      {children}
    </div>
  );
};
