import classNames from "classnames";
import { actions } from "../../../services/actions";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { AppState } from "../../../services/types";
import { useAppState } from "../../../services/state";
import { CSSProperties, FC, useEffect, useState } from "react";

type Props = { className?: string; style?: CSSProperties };

export const Renderer: FC<Props> = ({ className, style }) => {
  const state = useAppState();
  const [src, setSrc] = useState<string | null>(null);

  const firstFlow = state.score.flows.order[0];

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;
    let imageUrl: string | null = null;
    let animationFrameId: number | null = null;

    const run = async () => {
      if (imageUrl) {
        URL.revokeObjectURL(imageUrl);
        imageUrl = null;
      }

      if (!firstFlow) {
        return;
      }

      const result = await actions.render(firstFlow);
      const blob = new Blob([result], { type: "image/svg+xml" });
      imageUrl = URL.createObjectURL(blob);
      setSrc(imageUrl);
    };

    const handleStateUpdate = () => {
      if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
      }

      animationFrameId = requestAnimationFrame(async () => {
        await run();
        animationFrameId = null;
      });
    };

    (async () => {
      unlisten = await listen<AppState>(
        "solocompose://state",
        handleStateUpdate
      );
    })();

    return () => {
      unlisten?.();
      if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
      }
    };
  }, [firstFlow]);

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
