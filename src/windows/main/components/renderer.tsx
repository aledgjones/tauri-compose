import classNames from "classnames";
import { actions } from "../../../services/actions";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { AppState } from "../../../services/types";
import { useAppState } from "../../../services/state";
import { FC, useEffect, useState } from "react";
import { DragScroll } from "../../../components/drag-scroll";

const getMM = () => {
  const div = document.createElement("div");
  div.style.position = "fixed";
  div.style.width = "1mm";
  document.body.appendChild(div);
  const width = div.clientWidth;
  document.body.removeChild(div);

  return width;
};

export const pxPerMm = getMM();

type Props = { className?: string };

export const Renderer: FC<Props> = ({ className }) => {
  const state = useAppState();
  const [src, setSrc] = useState<string | null>(null);

  const firstFlow = state.score.flows.order[0];

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;
    let imageUrls: Set<string> = new Set();
    let animationFrameId: number | null = null;

    const run = async () => {
      if (!firstFlow) {
        return;
      }

      const result = await actions.render(
        state.ui.layout_id,
        firstFlow,
        pxPerMm
      );

      imageUrls.forEach((url) => {
        imageUrls.delete(url);
        URL.revokeObjectURL(url);
      });

      const blob = new Blob([result], { type: "image/webp" });
      const imageUrl = URL.createObjectURL(blob);
      imageUrls.add(imageUrl);
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
      await run();
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
  }, [firstFlow, state.ui.layout_id]);

  return (
    <DragScroll
      className={classNames(
        "bg-fixed bg-center bg-cover from-sky-700 to-sky-900 bg-linear-to-b",
        className
      )}
    >
      <div className="flex min-h-full items-center justify-center">
        <div className="p-20 inline-block">
          {src && (
            <img
              alt="Your score"
              draggable="false"
              style={{ zoom: "50%" }}
              src={src}
            />
          )}
        </div>
      </div>
    </DragScroll>
  );
};
