import classNames from "classnames";
import { FC, useCallback, useRef, useEffect } from "react";
import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";

type Props = { className?: string };

export const Renderer: FC<Props> = ({ className }) => {
  const divRef = useRef<HTMLDivElement>(null);
  const state = useAppState();

  // Track the renderer bounds and send to native renderer
  useEffect(() => {
    if (!divRef.current) return;

    const updateBounds = () => {
      if (!divRef.current) return;

      const rect = divRef.current.getBoundingClientRect();
      const pixelRatio = window.devicePixelRatio || 1.0;

      // Convert CSS coordinates to physical pixels
      const bounds = {
        x: rect.left * pixelRatio,
        y: rect.top * pixelRatio,
        width: rect.width * pixelRatio,
        height: rect.height * pixelRatio,
      };

      actions.ui.renderer.setBounds(bounds);
    };

    // Use setTimeout to ensure the div is fully mounted
    const timer = setTimeout(() => {
      updateBounds();
    }, 0);

    // Watch for resize changes
    const resizeObserver = new ResizeObserver(updateBounds);
    resizeObserver.observe(divRef.current);

    return () => {
      clearTimeout(timer);
      resizeObserver.disconnect();
      actions.ui.renderer.setBounds({ x: 0, y: 0, width: 0, height: 0 });
    };
  }, []);

  const handlePointerDown = useCallback(
    (e: any) => {
      if (!divRef.current) {
        return;
      }

      let isDragging = true;

      const startX = e.clientX;
      const startY = e.clientY;
      const initialPanX = state.ui.renderer.pan_x;
      const initialPanY = state.ui.renderer.pan_y;

      const handlePointerMove = (e: any) => {
        if (!isDragging || !divRef.current) {
          return;
        }

        e.preventDefault();

        const deltaX = e.clientX - startX;
        const deltaY = e.clientY - startY;

        // Account for device pixel ratio - mouse coordinates are in CSS pixels
        // but canvas rendering happens in physical pixels
        const pixelRatio = window.devicePixelRatio || 1.0;
        const adjustedDeltaX = deltaX * pixelRatio;
        const adjustedDeltaY = deltaY * pixelRatio;

        const newPanX = initialPanX + adjustedDeltaX;
        const newPanY = initialPanY + adjustedDeltaY;

        // Update the backend pan state
        actions.ui.renderer.setPan(newPanX, newPanY);
      };

      const handlePointerUp = () => {
        document.removeEventListener("pointermove", handlePointerMove);
        document.removeEventListener("pointerup", handlePointerUp);
        isDragging = false;
      };

      document.addEventListener("pointermove", handlePointerMove);
      document.addEventListener("pointerup", handlePointerUp);
    },
    [state.ui.renderer.pan_x, state.ui.renderer.pan_y],
  );

  return (
    <div
      ref={divRef}
      className={classNames("bg-transparent touch-none", className)}
      data-testid="renderer"
      onPointerDown={handlePointerDown}
    />
  );
};
