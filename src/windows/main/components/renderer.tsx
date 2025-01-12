import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { CSSProperties } from "preact/compat";

type Props = { className?: string; style?: CSSProperties };

export const Renderer: FunctionalComponent<Props> = ({ className, style }) => {
  return (
    <div
      style={{ ...style }}
      className={classNames(
        "overflow-auto bg-fixed bg-center bg-cover from-sky-700 to-sky-900 bg-gradient-to-b",
        className
      )}
    >
      <div className="p-20 inline-block">
        <img
          draggable={false}
          className="max-w-none"
          src="solocompose://render.webp"
        />
      </div>
    </div>
  );
};
