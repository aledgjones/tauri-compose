import classNames from "classnames";
import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";
import { FC, useEffect, useRef, useState } from "react";
import { DragScroll } from "../../../components/drag-scroll";

type Props = { className?: string };

export const Renderer: FC<Props> = ({ className }) => {
  const state = useAppState();
  const [src, setSrc] = useState<string | null>(null);
  const imageUrls = useRef<Set<string>>(new Set());
  const animationFrameId = useRef<number | null>(null);

  useEffect(() => {
    // Cancel any pending animation frame
    if (animationFrameId.current) {
      cancelAnimationFrame(animationFrameId.current);
    }

    // Schedule render for next animation frame
    animationFrameId.current = requestAnimationFrame(async () => {
      const firstFlow = state.score.flows.order[0];

      if (firstFlow === undefined) {
        setSrc(null);
        return;
      }

      const result = await actions.render(state.ui.layout_id, firstFlow);

      imageUrls.current.forEach((url) => {
        imageUrls.current.delete(url);
        URL.revokeObjectURL(url);
      });

      const blob = new Blob([result], { type: "image/webp" });
      const imageUrl = URL.createObjectURL(blob);
      imageUrls.current.add(imageUrl);
      setSrc(imageUrl);

      animationFrameId.current = null;
    });

    return () => {
      if (animationFrameId.current) {
        cancelAnimationFrame(animationFrameId.current);
        animationFrameId.current = null;
      }
    };
  }, [state]);

  return (
    <DragScroll
      className={classNames(
        "bg-fixed bg-center bg-cover from-sky-700 to-sky-900 bg-linear-to-b",
        className,
      )}
    >
      <div className="flex min-h-full">
        <div className="p-20 inline-block">
          {src && (
            <img
              alt="Your score"
              draggable="false"
              style={{ zoom: `${(state.ui.renderer.zoom / 100) * 50}%` }}
              src={src}
            />
          )}
        </div>
      </div>
    </DragScroll>
  );
};
