import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { CSSProperties } from "preact/compat";

import bg from "../../../assets/images/bg.jpg";

type Props = { className?: string; style?: CSSProperties };

export const Renderer: FunctionalComponent<Props> = ({ className, style }) => {
  return (
    <div
      style={{ ...style, backgroundImage: `url(${bg})` }}
      className={classNames(
        "overflow-auto rounded-lg bg-fixed bg-center bg-cover",
        className
      )}
    >
      <div className="p-10 inline-block">
        <img
          draggable={false}
          className="max-w-none"
          src="solocompose://render.webp"
        />
      </div>
    </div>
  );
};
