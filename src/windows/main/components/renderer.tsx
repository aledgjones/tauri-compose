import classNames from "classnames";
import { FC, useCallback, useRef, useEffect } from "react";
import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";
import { pxPerMm } from "../../../utils/mm";

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

      const bounds = {
        x: rect.left,
        y: rect.top,
        width: rect.width,
        height: rect.height,
      };

      actions.ui.renderer.setBounds(pxPerMm, bounds);
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
      actions.ui.renderer.setBounds(pxPerMm, {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
      });
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

        // Work in CSS pixels - matches mouse coordinates naturally
        const newPanX = initialPanX + deltaX;
        const newPanY = initialPanY + deltaY;

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

  // Handle mouse wheel for panning and zooming
  useEffect(() => {
    const handleWheel = (e: WheelEvent) => {
      e.preventDefault();

      // Handle zoom (with modifier key)
      if (e.metaKey || e.ctrlKey) {
        // TODO: Implement zooming from center point
        // const zoomDelta = -e.deltaY;
        // const nextZoom = Math.max(
        //   Math.floor(state.ui.renderer.zoom + zoomDelta),
        //   50,
        // );
        // console.log(zoomDelta, nextZoom);
        // actions.ui.renderer.setZoom(nextZoom);
      } else {
        actions.ui.renderer.setPan(
          state.ui.renderer.pan_x - e.deltaX,
          state.ui.renderer.pan_y - e.deltaY,
        );
      }
    };

    divRef.current?.addEventListener("wheel", handleWheel, { passive: false });

    return () => {
      divRef.current?.removeEventListener("wheel", handleWheel);
    };
  }, [state.ui.renderer.pan_x, state.ui.renderer.pan_y]);

  return (
    <div
      ref={divRef}
      className={classNames("bg-transparent touch-none", className)}
      data-testid="renderer"
      onPointerDown={handlePointerDown}
    />
  );
};
