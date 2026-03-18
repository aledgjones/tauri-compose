import classNames from "classnames";
import { FC, useRef, useEffect, useState } from "react";
import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";
import { pxPerMm } from "../../../utils/mm";
import { useKeyboardShortcuts } from "../hooks/use-keyboard-shortcuts";
import { hitTest } from "../../../services/hit-test";
import { Popover, PopoverState } from "../../../ui/popover";

type Props = { className?: string };

export const Renderer: FC<Props> = ({ className }) => {
  const divRef = useRef<HTMLDivElement>(null);
  const state = useAppState();
  const [popover, setPopover] = useState<PopoverState>(null);

  // Tone keyboard shortcuts
  useKeyboardShortcuts();

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

    const onPointerDown = async (e: PointerEvent) => {
      if (!$element) return;

      // Convert document coordinates to element-relative coordinates
      const rect = $element.getBoundingClientRect();
      const relativeX = e.clientX - rect.left;
      const relativeY = e.clientY - rect.top;

      // Check for hit target first
      const hitResult = await hitTest(
        relativeX,
        relativeY,
        panXRef.current,
        panYRef.current,
        zoomRef.current,
      );

      if (hitResult) {
        e.preventDefault();

        switch (hitResult.type) {
          case "Tone": {
            await actions.ui.sequencer.selectTone(
              hitResult.tone_id,
              hitResult.track_id,
              hitResult.stave_id,
              hitResult.flow_id,
              hitResult.instrument_id,
              hitResult.tuplet_id,
            );
            return;
          }
          case "Rest": {
            await actions.ui.sequencer.selectRest(
              hitResult.tick,
              hitResult.track_id,
              hitResult.stave_id,
              hitResult.flow_id,
              hitResult.instrument_id,
              hitResult.tuplet_id,
            );
            return;
          }
          case "Tempo": {
            await actions.ui.sequencer.selectTempo(
              hitResult.tempo_id,
              hitResult.flow_id,
              hitResult.tick,
            );
            return;
          }
          case "TimeSignature": {
            await actions.ui.sequencer.selectTimeSignature(
              hitResult.flow_id,
              hitResult.time_signature_id,
              hitResult.tick,
            );
            return;
          }
          case "KeySignature": {
            await actions.ui.sequencer.selectKeySignature(
              hitResult.flow_id,
              hitResult.key_signature_id,
              hitResult.tick,
            );
            return;
          }
          case "Clef": {
            await actions.ui.sequencer.selectClef(
              hitResult.flow_id,
              hitResult.stave_id,
              hitResult.clef_id,
              hitResult.tick,
            );
            return;
          }
          case "Tuplet": {
            await actions.ui.sequencer.selectTuplet(
              hitResult.flow_id,
              hitResult.instrument_id,
              hitResult.stave_id,
              hitResult.track_id,
              hitResult.tuplet_id,
              hitResult.tick,
            );
            return;
          }
        }
      }

      // No hit - proceed with panning
      let isDragging = true;
      let hasMoved = false;

      let panStartX = panXRef.current;
      let panStartY = panYRef.current;
      let startX = e.clientX;
      let startY = e.clientY;

      const handlePointerMove = async (e: PointerEvent) => {
        if (!isDragging || !$element) return;

        const deltaX = e.clientX - startX;
        const deltaY = e.clientY - startY;

        if (!hasMoved && (Math.abs(deltaX) > 3 || Math.abs(deltaY) > 3)) {
          hasMoved = true;
        }

        const newPanX = panStartX + deltaX;
        const newPanY = panStartY + deltaY;

        const [clampedX, clampedY] = await actions.ui.renderer.setPan(
          newPanX,
          newPanY,
        );

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
        if (!hasMoved && popover === null) {
          actions.ui.sequencer.clearSelection();
        }

        document.removeEventListener("pointermove", handlePointerMove);
        document.removeEventListener("pointerup", handlePointerUp);
        isDragging = false;
      };

      document.addEventListener("pointermove", handlePointerMove);
      document.addEventListener("pointerup", handlePointerUp);
    };

    const onWheel = async (e: WheelEvent) => {
      if (popover) {
        setPopover(null);
      }

      if (e.metaKey || e.ctrlKey) {
        const zoomDelta = Math.round(-e.deltaY);
        const newZoom = zoomRef.current + zoomDelta;

        const [clampedZoom, newPanX, newPanY] =
          await actions.ui.renderer.setZoom(newZoom);

        zoomRef.current = clampedZoom;
        panXRef.current = newPanX;
        panYRef.current = newPanY;
      } else {
        const newPanX = panXRef.current - e.deltaX;
        const newPanY = panYRef.current - e.deltaY;

        const [clampedX, clampedY] = await actions.ui.renderer.setPan(
          newPanX,
          newPanY,
        );

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
  }, [
    popover,
    state.preferences.audition,
    state.score.tracks.by_id,
    state.ui.sequencer.selection,
  ]);

  return (
    <div
      className={classNames("relative bg-transparent", className)}
      data-testid="renderer"
    >
      <div ref={divRef} className="absolute top-0 left-0 h-full w-full" />
      <Popover popover={popover} setPopover={setPopover} />
    </div>
  );
};
