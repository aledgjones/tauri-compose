import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { CSSProperties, useEffect, useState } from "preact/compat";
import { useAppState } from "../../../services/state";
import { actions } from "../../../services/actions";

type Props = { className?: string; style?: CSSProperties };

export const Renderer: FunctionalComponent<Props> = ({ className, style }) => {
  const state = useAppState();
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    let imageUrl: string | null = null;

    const run = async () => {
      const result = await actions.render();
      console.log(result);
      const blob = new Blob([result], { type: "image/webp" });
      imageUrl = URL.createObjectURL(blob);
      setSrc(imageUrl);
    };

    run();

    return () => {
      if (imageUrl) {
        URL.revokeObjectURL(imageUrl);
      }
    };
  }, [state]);

  return (
    <div
      style={{ ...style }}
      className={classNames(
        "overflow-auto bg-fixed bg-center bg-cover from-sky-700 to-sky-900 bg-gradient-to-b",
        className
      )}
    >
      <div className="p-20 flex gap-5 text-xs">{src && <img src={src} />}</div>
    </div>
  );
};
