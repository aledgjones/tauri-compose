import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { CSSProperties, useEffect, useState } from "preact/compat";
import { actions } from "../../../services/actions";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { AppState } from "../../../services/types";

type Props = { className?: string; style?: CSSProperties };

export const Renderer: FunctionalComponent<Props> = ({ className, style }) => {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;
    let imageUrl: string | null = null;
    let animationFrameId: number | null = null;

    const run = async () => {
      if (imageUrl) {
        URL.revokeObjectURL(imageUrl);
        imageUrl = null;
      }

      const result = await actions.render();
      const blob = new Blob([result], { type: "image/webp" });
      imageUrl = URL.createObjectURL(blob);
      setSrc(imageUrl);
    };

    const handleStateUpdate = () => {
      if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
      }

      animationFrameId = requestAnimationFrame(() => {
        run();
        animationFrameId = null;
      });
    };

    (async () => {
      unlisten = await listen<AppState>("solocompose://state", handleStateUpdate);
    })();

    return () => {
      unlisten?.();
      if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
      }
    };
  }, []);

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
