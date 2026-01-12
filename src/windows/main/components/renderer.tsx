import classNames from "classnames";
import { FC, useRef, useEffect } from "react";
import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";
import { pxPerMm } from "../../../utils/mm";

type Props = { className?: string };

export const Renderer: FC<Props> = ({ className }) => {
  const divRef = useRef<HTMLDivElement>(null);
  const state = useAppState();

  // Use refs to track current pan/zoom without triggering re-renders
  const panXRef = useRef(state.ui.renderer.pan_x);
  const panYRef = useRef(state.ui.renderer.pan_y);
  const zoomRef = useRef(state.ui.renderer.zoom);

  useEffect(() => {
    panXRef.current = state.ui.renderer.pan_x;
    panYRef.current = state.ui.renderer.pan_y;
    zoomRef.current = state.ui.renderer.zoom;
  }, [
    state.ui.renderer.pan_x,
    state.ui.renderer.pan_y,
    state.ui.renderer.zoom,
  ]);

  // BOUNDS LISTENER
  useEffect(() => {
    if (!divRef.current) return;

    const $element = divRef.current;

    const updateBounds = () => {
      const rect = $element.getBoundingClientRect();

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
    resizeObserver.observe($element);

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

  // PANNING & ZOOMING LISTENERS
  useEffect(() => {
    if (!divRef.current) return;

    const $element = divRef.current;

    const onPointerDown = (e: PointerEvent) => {
      let isDragging = true;

      let panStartX = panXRef.current;
      let panStartY = panYRef.current;
      let startX = e.clientX;
      let startY = e.clientY;

      const handlePointerMove = async (e: PointerEvent) => {
        if (!isDragging || !$element) return;

        const deltaX = e.clientX - startX;
        const deltaY = e.clientY - startY;

        const newPanX = panStartX + deltaX;
        const newPanY = panStartY + deltaY;

        // Call with silent=true - get clamped values back
        const [clampedX, clampedY] = await actions.ui.renderer.setPan(
          newPanX,
          newPanY,
        );

        // Update refs with clamped values
        panXRef.current = clampedX;
        panYRef.current = clampedY;

        // If clamping occurred, adjust the start position so next delta is correct
        if (clampedX !== newPanX || clampedY !== newPanY) {
          panStartX = clampedX;
          panStartY = clampedY;
          startX = e.clientX;
          startY = e.clientY;
        }
      };

      const handlePointerUp = () => {
        document.removeEventListener("pointermove", handlePointerMove);
        document.removeEventListener("pointerup", handlePointerUp);
        isDragging = false;
      };

      document.addEventListener("pointermove", handlePointerMove);
      document.addEventListener("pointerup", handlePointerUp);
    };

    const onWheel = async (e: WheelEvent) => {
      if (e.metaKey || e.ctrlKey) {
        const zoomDelta = Math.round(-e.deltaY * 0.5);
        const newZoom = zoomRef.current + zoomDelta;

        const [clampedZoom, newPanX, newPanY] =
          await actions.ui.renderer.setZoom(newZoom);

        zoomRef.current = clampedZoom;
        panXRef.current = newPanX;
        panYRef.current = newPanY;
      } else {
        const newPanX = panXRef.current - e.deltaX;
        const newPanY = panYRef.current - e.deltaY;

        // Call with silent=true - get clamped values back
        const [clampedX, clampedY] = await actions.ui.renderer.setPan(
          newPanX,
          newPanY,
        );

        // Update refs with clamped values
        panXRef.current = clampedX;
        panYRef.current = clampedY;
      }
    };

    $element.addEventListener("pointerdown", onPointerDown, { passive: true });
    $element.addEventListener("wheel", onWheel, { passive: true });

    return () => {
      $element.removeEventListener("pointerdown", onPointerDown);
      $element.removeEventListener("wheel", onWheel);
    };
  }, []);

  return (
    <div
      ref={divRef}
      className={classNames("bg-transparent touch-none", className)}
      data-testid="renderer"
    />
  );
};
