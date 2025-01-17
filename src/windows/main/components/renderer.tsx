import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { CSSProperties } from "preact/compat";
import { useAppState } from "../../../services/state";

type Props = { className?: string; style?: CSSProperties };

export const Renderer: FunctionalComponent<Props> = ({ className, style }) => {
  const state = useAppState();

  return (
    <div
      style={{ ...style }}
      className={classNames(
        "overflow-auto bg-fixed bg-center bg-cover from-sky-700 to-sky-900 bg-gradient-to-b",
        className
      )}
    >
      <div className="p-20 flex gap-5 text-xs">
        {state.score.flows.order.map(() => {
          return <img src="solocompose://render" />;
        })}
      </div>
    </div>
  );
};
